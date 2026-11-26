//! An online challenge round: `StateOnlineGame`'s `ChallengeLoadState` (0x82771a28),
//! `ChallengeSkate` (0x82776a78), `WaitingForGameToFinish` (0x82775a78), `SendPlayerResults`
//! (0x82774f08) and `DisplayFinalResults` (0x82772e58).
//!
//! The original ran the challenge script on every console over a lockstep of every pad, so each
//! console saw every skater's output. Here each console simulates only its own skater: every
//! tick it sends that skater's completed output (its "facts", `crate::online::facts`) and runs
//! the challenge system once it holds every player's facts of the tick. Every console runs the
//! same ticks over the same facts with the same `GameReset` seed, so the script reaches the same
//! letters, turns and results everywhere; what the script does to a skater, only that skater's
//! console carries out.

use super::{SelectedLiveError, SelectedLiveSession, error, trace};
use crate::challenge::mission::MissionFrame;
use crate::challenge::players::{OnlineChallenge, PT_HUMAN, PlayersBackEnd, SpotBattleCarry};
use crate::challenge::results::{OnlineResultRow, OnlineResults};
use crate::challenge::system::Output;
use crate::online::catalog;
use crate::online::event_log::EventText;
use crate::online::facts::{self, FactsCoder, FactsScope};
use crate::players_backend_settings::PlayersBackendSettings;
use super::super::career::HeldPresentation;
use skate2_net::protocol::{ClientMessage, FactsNotes, GameResult, MAX_PLAYERS, MAX_PRESENTED, Phase, PlayerStats, Presented, reset};
use std::collections::{BTreeMap, BTreeSet};

/// Co-op fork (M11): how long the challenge AI skaters stand at their leaving owner's last
/// frames before the next owner's facts must carry them (its lead over the lockstep is about
/// 120 ticks, and its spawns may wait on a character's scene).
const AI_HANDOFF_GRACE: u32 = 600;
use std::sync::{Arc, Mutex};

/// Challenge ticks one simulation tick may run while catching up with the other players.
const CATCH_UP_TICKS: usize = 4;
/// Ready challenge ticks a round that ran out waits for before it runs on: the other players'
/// facts arrive with their runs of frames, two or three ticks at a time and early or late by the
/// link's jitter, and the ticks they leave waiting are a jitter buffer that keeps the round, and
/// the scenes it plays, at the simulation's pace instead of stopping at every arrival.
const REFILL_LEAD: u32 = 6;
/// Simulation ticks a refilling round waits with some ticks ready before it runs those. Facts
/// that are coming fill the buffer well inside it; a player who paces the round for a tick or
/// two (a teleport done at once, as Skate 3's 1-Up moves its players) sends no more until the
/// round has run them, so waiting on for a full buffer would hold the round for good.
const REFILL_WAIT: u32 = 12;
/// Ready challenge ticks a simulation tick still runs one at a time: the buffer and a run's
/// arrival on top of it.
const SMOOTH_LEAD: u32 = 12;
/// Past `SMOOTH_LEAD` a simulation tick runs two; past this, `CATCH_UP_TICKS`.
const CATCH_UP_LEAD: u32 = 30;
/// The most facts kept of another player, the furthest ahead going first: half a minute of its
/// ticks, far past any lead the catch-up leaves. Facts for ticks or runs the round never reaches
/// would otherwise pile up.
const KEPT_FACTS: usize = 1800;

/// The challenge ticks a simulation tick runs with `lead` of them ready.
const fn ticks_for_lead(lead: u32) -> usize {
    if lead == 0 {
        0
    } else if lead <= SMOOTH_LEAD {
        1
    } else if lead <= CATCH_UP_LEAD {
        2
    } else {
        CATCH_UP_TICKS
    }
}

/// The challenge ticks a simulation tick runs with `lead` of them ready, and whether the round
/// then waits to refill: once out of ticks, until [`REFILL_LEAD`] are ready.
const fn pace(lead: u32, refilling: bool) -> (usize, bool) {
    if lead == 0 || (refilling && lead < REFILL_LEAD) { (0, true) } else { (ticks_for_lead(lead), false) }
}
/// The places of the final results' lines: the round's players the results hold, in the
/// round's order. FE state 27's data blocks read `mNumPlayers` lines from the first
/// (`Results_GetLeaderboardDataBlock` 0x826b9608, `Results_GetLeaderboardNameBlock`
/// 0x826b9ba0), and the server's results leave out a player who left the round, so the lines
/// close up over them.
fn result_places(players: &[u64], rows: &[PlayerStats]) -> Vec<u64> {
    players.iter().copied().filter(|&user_id| rows.iter().any(|row| row.user_id == user_id)).collect()
}

/// `GM_OnlineChallenge` and `GM_OnlineFreeSkate` as the session's game info has them.
const GM_ONLINE_CHALLENGE: u32 = 5;
const GM_ONLINE_FREE_SKATE: u32 = 4;
/// `eChallengeTypes` S.K.A.T.E. and Own the Spot (the online Spot Battle).
const SKATE: i32 = 4;
const OWN_THE_SPOT: i32 = 9;
/// PC port (user request): the longest a wipeout runs in their online rounds before its respawn:
/// S.K.A.T.E.'s, then Spot Battle's.
const TURN_BAIL_LIMIT: f32 = 5.0;
const SPOT_BATTLE_BAIL_LIMIT: f32 = 4.0;
/// PC port (user request): a player waiting through another's turn whose skater shows farther
/// than this from the spot the challenge put them on (x and z) is not drawn on this console, so
/// skating around through others' turns shows nothing to anyone; their own console still shows
/// them.
const SPOT_LIMIT: f32 = 4.0;
/// Closer than this to the spot, the player has reached it.
const SPOT_REACHED: f32 = 1.5;
/// A spot not reached by then holds the player to it anyway.
const SPOT_GRACE: std::time::Duration = std::time::Duration::from_secs(8);
/// PC port (user request): four seconds of simulation ticks. A S.K.A.T.E. round that waits this
/// long at a challenge tick for one other player's facts tells the server
/// (`ClientMessage::Stalled`), which takes the player out of the challenge once every other
/// player's round waits on them there: whose set trick a player copied decides whose turn comes
/// next, so a console whose game parted from the others' waits on another turn than theirs, and
/// every one of them waits on it.
const STALL_TICKS: u32 = 240;
/// PC port (user request): the log's lines for the host's Infinite Rounds and its end of such a
/// battle.
pub(super) const HOST_ENDED_CHALLENGE: &str = "The host has ended the challenge.";
pub(super) const INFINITE_ROUNDS_ENABLED: &str = "The host has enabled infinite rounds.";
pub(super) const INFINITE_ROUNDS_DISABLED: &str = "The host has disabled infinite rounds.";
/// PC port (user request): the log's line for the host's Round Lock turned off; set, the line
/// names its round ([`round_lock_line`]).
pub(super) const ROUND_LOCK_DISABLED: &str = "The host has turned the round lock off.";

/// The log's line for the host's Round Lock at `round` (0 off).
pub(super) fn round_lock_line(round: u32) -> String {
    if round == 0 { ROUND_LOCK_DISABLED.to_owned() } else { format!("The host has locked every round to round {round}.") }
}
/// While the wait goes on, the round tells the server again every second of simulation ticks:
/// the server counts a report only while it is fresh, so one from a wait that has ended cannot
/// join later ones.
const STALL_REPORT_TICKS: u32 = 60;
/// Co-op fork (M22): the "waiting for players" notice's text, the game's own (`SetupLobbyHUD`'s
/// waiting line); the original's `eSk8FEPopup_Online_WaitForPlayersSmall` is not read here.
const WAITING_FOR_SKATERS: &str = "ID_ONLINE_WAITING_FOR_SKATERS";

/// Where the challenge last teleported a player of a turn-based round (see [`SPOT_LIMIT`]),
/// whether their skater has shown there since, and whether this console now leaves it undrawn.
#[derive(Clone, Copy, Debug)]
struct Spot {
    xz: [f32; 2],
    at: std::time::Instant,
    reached: bool,
    hidden: bool,
}

/// Co-op fork (M23): the tick a difference found at checkpoint `tick` dumps its parts' fields
/// at: the next multiple of [`SYNC_DUMP_GRID`] at least a grid past it, which every console
/// reaches after hearing of the difference unless its round trails by more than that.
const fn sync_dump_tick(tick: u32) -> u32 {
    (tick / SYNC_DUMP_GRID + 2) * SYNC_DUMP_GRID
}

/// Co-op fork (M23): the grid of ticks field dumps fall on.
const SYNC_DUMP_GRID: u32 = 300;

fn distance_xz(root: [f32; 3], xz: [f32; 2]) -> f32 {
    let (dx, dz) = (root[0] - xz[0], root[2] - xz[1]);
    (dx * dx + dz * dz).sqrt()
}

/// The other player whose facts the round's next tick waits for (see [`STALL_TICKS`]) and the
/// simulation ticks the round has waited for them there.
#[derive(Clone, Copy, Debug)]
struct Awaiting {
    user_id: u64,
    tick: u32,
    waited: u32,
}

impl Awaiting {
    /// The server has heard of the wait.
    const fn reported(&self) -> bool {
        self.waited >= STALL_TICKS
    }
}

/// `GetChallengeColumnHeader` 0x82605a50 of a challenge type: S.K.A.T.E.'s letters, Death
/// Race's time, otherwise trick points (a freeskate activity's by its type,
/// [`crate::online::activities::column_header`]).
const fn column_header(challenge_type: i32) -> &'static str {
    match challenge_type {
        4 | 5 => "ID_LEADERBOARD_LETTERS_HEADER",
        19 => "ID_LEADERBOARD_HEADER_TIME",
        _ => "ID_LEADERBOARD_HEADER_TRICK_POINTS",
    }
}

/// Where the local player's round is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    /// `ChallengeLoadState`: the challenge's assets load, then `GameChallengeLoaded`.
    Loading { sent: bool },
    /// `ChallengeSkate`.
    Running,
    /// `WaitingForGameToFinish`: `GameComplete` sent.
    Complete,
    /// `DisplayFinalResults`.
    Results,
}

/// Another player's facts as they arrived, decoded in order: by the run of ticks it sent facts
/// for and the tick, with what it told every console at the tick.
struct PlayerFacts {
    coder: FactsCoder,
    frames: BTreeMap<(u32, u32), TickFacts>,
    left: bool,
    /// The run and tick the player waits at a sync point from, standing still at those facts.
    syncing: Option<(u32, u32)>,
    /// Co-op fork (M10): the coders of the challenge AI skaters the player owns, by their slot,
    /// each decoded as the player codes it; the slots its last facts carried, for the trace.
    owned_coders: BTreeMap<u8, FactsCoder>,
    owned_slots: Vec<u8>,
}

/// One tick of a player's facts: its frame, its notes, and (M10) the frames of the challenge
/// AI skaters it owns, at their slots.
#[derive(Clone)]
struct TickFacts {
    frame: MissionFrame,
    notes: FactsNotes,
    owned: Vec<MissionFrame>,
}

/// One tick of the local skater's facts as every console runs the challenge on them (shared),
/// whole (for what only this console reads of its own skater), the notes sent with them, and
/// (M10) the owned challenge AI skaters' shared frames sent with them.
#[derive(Clone)]
struct LocalTickFacts {
    shared: MissionFrame,
    whole: MissionFrame,
    notes: FactsNotes,
    owned: Vec<MissionFrame>,
}

impl PlayerFacts {
    /// The facts of a player in a round of `scope`, decoded as the player codes them.
    fn new(scope: FactsScope) -> Self {
        Self { coder: FactsCoder::new(scope), frames: BTreeMap::new(), left: false, syncing: None, owned_coders: BTreeMap::new(), owned_slots: Vec::new() }
    }
}

/// `StateOnlineGame`'s sync point (`MESSAGE_ChallengeSyncPoint`, `mInSyncPoint`): the round and
/// the local player's spawn point.
struct SyncPoint {
    round: u32,
    spawn: Option<[f32; 16]>,
    /// `GameSyncPoint` went to the server (`ChallengeSkate`'s `mChallengeLoaded`).
    sent: bool,
}

/// `StateOnlineGame::WaitForStreamer` (`0x82748ae8`), which `SetLoadingState(true, true, true)`
/// puts the loading screen up for: `mWaitForStreamerPos` and `mWaitForStreamerTick`, the updates
/// the area there has stayed loaded.
struct StreamerWait {
    position: [f32; 3],
    ticks: u32,
}

/// What an online game shows in place of the game camera.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OnlineView {
    /// A scene's camera.
    Scene,
    /// The camera of the player with this challenge index.
    Player(i32),
    /// The camera of the player a free skate's Spectate watches (PC port addition).
    Spectate(u64),
}

/// The round in progress.
pub(super) struct Round {
    pub(super) session: u32,
    challenge: u64,
    /// The freeskate activity the round plays (`mActivityKey`), whose objective groups each read
    /// their own player.
    activity: Option<crate::online::activities::Activity>,
    /// PC port: a linked Skate 3 data folder's game that runs Skate 3's own state graph, whose
    /// objective groups each read their own player too (`crate::skate3`).
    own_graph: bool,
    pub(super) stage: Stage,
    /// The players by challenge index (`GameReset`'s table).
    players: Vec<u64>,
    local: i32,
    /// The next tick of the local skater's facts while it sends them, and the next challenge tick
    /// to run.
    produced: u32,
    processed: u32,
    /// The challenge ticks run, and the first tick since the last sync point.
    ticks: u32,
    base: u32,
    /// The sync points the round has gone through.
    syncs: u32,
    /// What the challenge reads of another player's facts.
    scope: FactsScope,
    coder: FactsCoder,
    /// The local skater's facts by tick ([`LocalTickFacts`]).
    local_frames: BTreeMap<u32, LocalTickFacts>,
    /// Co-op fork (M10): the coders of the challenge AI skaters this console owns, by slot, and
    /// the AI slots whose frames the last tick carried (for `cMsgSkaterCreated`/`Destroyed`).
    owned_coders: BTreeMap<u8, FactsCoder>,
    ai_present: BTreeSet<i32>,
    /// The slots the last produced facts carried, for the trace.
    owned_slots: Vec<u8>,
    /// This console's ambient skaters last counted, for the trace (they are no frame of the scripts').
    ambient_seen: usize,
    /// The local skater's last whole frame, for the ticks it sends no facts for.
    own: Option<MissionFrame>,
    others: BTreeMap<u64, PlayerFacts>,
    /// The round ran out of other players' facts and waits for [`REFILL_LEAD`] ticks of them.
    refilling: bool,
    /// Simulation ticks the refill has waited with ticks ready ([`REFILL_WAIT`]).
    refill_waited: u32,
    /// Simulation ticks the running round ran no tick in, for the trace.
    stalls: u32,
    /// The players whose facts pace the next challenge tick, a bit per player index. In a
    /// turn-based challenge, once a turn has started, the turn player and the players the
    /// challenge has teleported without their output finishing the teleport: a turn runs at its
    /// player's pace, whatever the others' speed, and the others stand still at their last facts
    /// and send none. Otherwise every player still in the round.
    contributors: u8,
    /// S.K.A.T.E., Spot Battle, Hall of Meat and 1-Up: one player plays at a time.
    turn_based: bool,
    /// S.K.A.T.E. and Spot Battle: the longest a wipeout lasts (`TURN_BAIL_LIMIT`,
    /// `SPOT_BATTLE_BAIL_LIMIT`; PC port).
    bail_limit: Option<f32>,
    /// Each player's waiting spot in a turn-based round, by player index (PC port).
    spots: [Option<Spot>; MAX_PLAYERS],
    /// S.K.A.T.E.: a round that waits [`STALL_TICKS`] at a tick for one other player's facts
    /// tells the server (PC port).
    stall_reports: bool,
    awaiting: Option<Awaiting>,
    /// How many runs of ticks each player has sent facts for, counted alike on every console.
    stints: [u32; MAX_PLAYERS],
    /// Each player's frame of the last tick it sent facts for, and (M10) the owned skaters'
    /// frames that came with it: a player standing at its last facts has its AI stand too.
    standing: [Option<MissionFrame>; MAX_PLAYERS],
    standing_owned: [Vec<MissionFrame>; MAX_PLAYERS],
    /// Votes pressed (player index, score) and players who left while not sending facts, until a
    /// tick's notes count them. A player who left is held by user id and numbered in the notes
    /// by the table then: one who left during the load is not in the challenge's table, which the
    /// server packs again without them, so a load index would name another player.
    votes: BTreeMap<u8, i32>,
    leaving: BTreeSet<u64>,
    /// The facts the local skater sent: ticks and coded bytes, for the trace.
    facts_sent: (u32, usize),
    /// Players whose `cMsgPlayerDisconnected` went to the challenge.
    disconnected: Vec<i32>,
    /// Co-op fork (M11): a player who owned challenge AI skaters left at this tick (and its
    /// index): its skaters stand at its last frames until another player's facts carry them,
    /// `AI_HANDOFF_GRACE` ticks at most (open item 65).
    ai_handoff: Option<(u32, usize)>,
    /// `mMyGameResult`.
    result: Option<GameResult>,
    stance: Arc<Mutex<[bool; MAX_PLAYERS]>>,
    /// The completed tick the challenge last unpinned the local skater at, and how many times
    /// it has, for diagnostics.
    unpinned: Option<u32>,
    turns: u32,
    /// The challenge's last `cMsgPinPlayer` for the local skater pinned it: what the skater goes
    /// back to when Skate During Turn Wait stops (PC port).
    script_pinned: bool,
    /// The player `Online.SwitchCamera` last made this console follow.
    camera_player: Option<i32>,
    /// The sync point the challenge waits at.
    sync: Option<SyncPoint>,
    /// Sync points the server let go before this console reached them (its timeout).
    released: BTreeSet<u32>,
    /// `ChallengeLoadState`'s Enter ran: the lobby's travel came first.
    load_entered: bool,
    /// Co-op fork: the round is a campaign's, the whole session long: every player paces it,
    /// its objective groups each read their own player, and it never completes.
    pub(super) campaign: bool,
    /// Co-op fork (M22): a campaign round's pace and its "waiting for players" notice
    /// (`crate::coop::pace`).
    pace: crate::coop::pace::Pace,
    stall: crate::coop::pace::StallWatch,
    /// The session frame the pace and the watch last stepped in: a frame the round is not
    /// updated in (a travel) leaves the timer to the game and starts the wait again.
    paced_frame: Option<u32>,
    /// Co-op fork (M9): the outcomes this console's scripts decided and not yet carried by a tick,
    /// numbered; the newest applied of each player's.
    outcomes_pending: Vec<skate2_net::protocol::Outcome>,
    outcome_seq: u32,
    outcomes_applied: [u32; MAX_PLAYERS],
    /// Co-op fork: the prompts this console answered and not yet carried by a tick, numbered;
    /// the next number; and the last number of each player's applied.
    prompts_pending: Vec<skate2_net::protocol::Prompt>,
    prompt_seq: u32,
    prompts_applied: [u32; MAX_PLAYERS],
    /// `mWaitingForStreamer`: the loading screen is up.
    streamer: Option<StreamerWait>,
    /// `SKATE2_DIAGNOSTIC_ONLINE_VOTE` voted on the turn the HUD shows.
    diagnostic_voted: bool,
    /// The trigger volumes the local skater entered since the challenge last unpinned it, with
    /// the completed tick, for diagnostics.
    diagnostic_entered: Vec<(u64, u32)>,
    /// `SKATE2_TRACE_ONLINE_INPUTS`: a running hash of the ticks' frames.
    input_hash: u64,
    /// Co-op fork (M23): the campaign round's sync check (`crate::coop::sync_check`) and the
    /// grid ticks whose field dumps a difference asked for (two, so seats that heard of it a
    /// grid apart still dump at one tick alike) and the parts.
    sync_check: crate::coop::sync_check::SyncCheck,
    sync_dump_at: Option<([u32; 2], u8)>,
    /// The scenes' and speech lines' events this console saw, numbered (`Presented`), until a
    /// tick delivers them, each with whether this console announced it while leading; the
    /// counts seen and delivered (delivered alike on every console).
    presentation_log: Vec<(Presented, bool)>,
    nis_seen: u32,
    speech_seen: BTreeMap<(i32, i32), u32>,
    nis_delivered: u32,
    speech_delivered: BTreeMap<(i32, i32), u32>,
    /// Co-op fork: movie ends, counted as the NIS events are.
    fmv_seen: u32,
    fmv_delivered: u32,
    /// PC port (user request): a Spot Battle the host started with Infinite Rounds on, as the
    /// game had it at the round's load on every console.
    pub(super) infinite_rounds: bool,
    /// PC port (user request): the host's Round Lock of a 1-Up, as the game had it at the
    /// round's load on every console (0 without one).
    pub(super) round_lock: u32,
    /// The host ended it (`ServerMessage::ChallengeEnded`) and no tick has counted that yet:
    /// the local skater's facts say so until one does ([`FactsNotes::ended`]).
    end_pending: bool,
    /// A tick's notes counted the host's end, the same tick on every console.
    pub(super) ended: bool,
    /// The challenge has still to take the end (it takes none before its first turn): asked
    /// again at each tick.
    end_due: bool,
    /// PC port (user request): a Spot Battle, which takes in the players who join it in play.
    spot_battle: bool,
    /// A player waits to join it (`ServerMessage::ChallengeJoining`): the battle's next turn
    /// does not start, and its state goes to the server instead.
    join_wanted: bool,
    /// The state this battle starts from (`ServerMessage::ChallengeCarry`), until it starts: one
    /// the server loaded again for a player who joined the battle before it.
    carry: Option<Vec<u8>>,
}

impl Round {
    /// The round from `GameReset(LoadChallenge)`'s table, loading.
    #[expect(clippy::too_many_arguments, reason = "the reset's table and what the challenge type gives the round")]
    fn loading(
        session: u32,
        challenge: u64,
        activity: Option<crate::online::activities::Activity>,
        players: Vec<u64>,
        local: i32,
        scope: FactsScope,
        turn_based: bool,
        stance: Arc<Mutex<[bool; MAX_PLAYERS]>>,
    ) -> Self {
        Self {
            session,
            challenge,
            activity,
            own_graph: false,
            stage: Stage::Loading { sent: false },
            players,
            local,
            produced: 0,
            processed: 0,
            ticks: 0,
            base: 0,
            syncs: 0,
            scope,
            coder: FactsCoder::new(scope),
            local_frames: BTreeMap::new(),
            owned_coders: BTreeMap::new(),
            ai_present: BTreeSet::new(),
            owned_slots: Vec::new(),
            ambient_seen: 0,
            own: None,
            others: BTreeMap::new(),
            refilling: false,
            refill_waited: 0,
            stalls: 0,
            contributors: u8::MAX,
            turn_based,
            bail_limit: None,
            spots: [None; MAX_PLAYERS],
            stall_reports: false,
            awaiting: None,
            stints: [0; MAX_PLAYERS],
            standing: Default::default(),
            standing_owned: Default::default(),
            votes: BTreeMap::new(),
            leaving: BTreeSet::new(),
            facts_sent: (0, 0),
            disconnected: Vec::new(),
            ai_handoff: None,
            result: None,
            stance,
            unpinned: None,
            turns: 0,
            script_pinned: false,
            camera_player: None,
            sync: None,
            released: BTreeSet::new(),
            load_entered: false,
            campaign: false,
            pace: crate::coop::pace::Pace::default(),
            stall: crate::coop::pace::StallWatch::default(),
            paced_frame: None,
            prompts_pending: Vec::new(),
            prompt_seq: 0,
            prompts_applied: [0; MAX_PLAYERS],
            outcomes_pending: Vec::new(),
            outcome_seq: 0,
            outcomes_applied: [0; MAX_PLAYERS],
            streamer: None,
            diagnostic_voted: false,
            diagnostic_entered: Vec::new(),
            input_hash: 0,
            sync_check: crate::coop::sync_check::SyncCheck::default(),
            sync_dump_at: None,
            presentation_log: Vec::new(),
            nis_seen: 0,
            speech_seen: BTreeMap::new(),
            nis_delivered: 0,
            speech_delivered: BTreeMap::new(),
            fmv_seen: 0,
            fmv_delivered: 0,
            infinite_rounds: false,
            round_lock: 0,
            end_pending: false,
            ended: false,
            end_due: false,
            spot_battle: false,
            join_wanted: false,
            carry: None,
        }
    }

    /// The host ended the challenge: an infinite battle still in play takes it at the tick the
    /// next facts' notes name.
    fn host_ended(&mut self) {
        if self.infinite_rounds && !self.ended && matches!(self.stage, Stage::Loading { .. } | Stage::Running) {
            self.end_pending = true;
        }
    }

    /// `GameReset(Challenge)`'s table: the players who loaded, packed again without those who
    /// left during the load.
    fn start(&mut self, players: Vec<u64>, local: i32) {
        self.players = players;
        self.leaving.retain(|user_id| self.players.contains(user_id));
        self.local = local;
        self.stage = Stage::Running;
    }

    /// A player of the table left: after their last facts they disconnect, and while they send
    /// none, the next facts' notes count them. One the challenge already disconnected (who left
    /// the challenge, then the game) needs no note.
    fn player_left(&mut self, user_id: u64) {
        let Some(index) = self.player_index(user_id) else { return };
        let scope = self.scope;
        self.others.entry(user_id).or_insert_with(|| PlayerFacts::new(scope)).left = true;
        if i32::try_from(index).is_ok_and(|player| !self.disconnected.contains(&player)) {
            self.leaving.insert(user_id);
        }
    }

    /// The challenge index of `user_id`'s skater.
    pub(super) fn player_index(&self, user_id: u64) -> Option<usize> {
        self.players.iter().position(|&player| player == user_id)
    }

    /// The challenge ticks: started and not waiting at a sync point.
    pub(super) fn running(&self) -> bool {
        matches!(self.stage, Stage::Running | Stage::Complete) && self.sync.is_none()
    }

    fn contributes(&self, index: usize) -> bool {
        self.contributors & (1 << index) != 0
    }

    /// Whether the local skater's facts pace the round.
    fn local_contributes(&self) -> bool {
        usize::try_from(self.local).is_ok_and(|local| self.contributes(local))
    }

    /// Co-op fork (M22): the local facts produced and not yet run, while a running campaign round
    /// is paced by the local skater and at least one other player (`UpdateInputRate`'s "sent and
    /// not simulated"); none otherwise.
    fn campaign_pace_lead(&self) -> Option<u32> {
        let others = self.players.len().saturating_sub(self.disconnected.len()) > 1;
        (self.campaign && self.running() && others && self.local_contributes()).then(|| self.produced.saturating_sub(self.processed))
    }

    /// Co-op fork (M22): why the pace has no lead to read, for the trace.
    fn unpaced_reason(&self) -> &'static str {
        if !self.running() {
            "the round not running (a sync point)"
        } else if !self.local_contributes() {
            "the local skater not pacing the round"
        } else {
            "no other player in the round"
        }
    }

    /// Whether the local skater alone paces the round: its own turn, no one else's teleport
    /// still under way.
    fn paced_by_local_alone(&self) -> bool {
        u32::try_from(self.local).ok().and_then(|local| 1_u8.checked_shl(local)).is_some_and(|bit| self.contributors == bit)
    }

    /// The challenge ticks this simulation tick runs with `lead` of them ready ([`pace`]): none
    /// while a round that ran out refills its buffer, unless the local skater alone paces it (no
    /// one's arrivals to buffer) or the ready ticks have waited [`REFILL_WAIT`] for more.
    fn ticks_to_run(&mut self, lead: u32) -> usize {
        let (mut ticks, mut refilling) = pace(lead, self.refilling && !self.paced_by_local_alone());
        if refilling && lead > 0 {
            self.refill_waited += 1;
            if self.refill_waited > REFILL_WAIT {
                (ticks, refilling) = (ticks_for_lead(lead), false);
            }
        }
        if !refilling || lead == 0 {
            self.refill_waited = 0;
        }
        self.refilling = refilling;
        ticks
    }

    /// Whether the local skater plays the round now: its facts pace the challenge, and the other
    /// players see it play (its turn, or a challenge everyone plays at once).
    pub(super) fn local_plays(&self) -> bool {
        self.running() && self.local_contributes()
    }

    /// The challenge's last pin of the local skater (`cMsgPinPlayer`) pinned it.
    pub(super) const fn script_pinned(&self) -> bool {
        self.script_pinned
    }

    /// Whether the round follows the local skater (`Online.SwitchCamera`). A turn-based challenge
    /// switches every console to the turn's player at the same tick, and no challenge everyone
    /// plays at once switches at all, so while the local skater plays, the others watch it.
    pub(super) fn local_followed(&self) -> bool {
        self.camera_player == Some(self.local)
    }

    /// The player `Online.SwitchCamera` last made this console follow.
    pub(super) fn followed(&self) -> Option<i32> {
        self.camera_player
    }

    /// Whether this console leads the round's presentation: it paces the round, and no player
    /// before it does.
    fn presentation_leader(&self) -> bool {
        self.local_contributes() && u32::try_from(self.local).is_ok_and(|local| self.contributors.trailing_zeros() == local)
    }

    /// A scene's or speech line's event this console saw, numbered in its kind's order.
    fn log_presentation(&mut self, event: HeldPresentation) {
        let presented = match event {
            HeldPresentation::Nis(notification) => {
                self.nis_seen += 1;
                Presented::Nis { seq: self.nis_seen, notification }
            }
            HeldPresentation::Speech { character, id } => {
                let seen = self.speech_seen.entry((character, id)).or_default();
                *seen += 1;
                Presented::Speech { character, id, seq: *seen }
            }
            HeldPresentation::Fmv => {
                self.fmv_seen += 1;
                Presented::Fmv { seq: self.fmv_seen }
            }
        };
        if self.campaign {
            crate::coop::trace(format_args!(
                "presentation {presented:?} seen here at tick {} ({})",
                self.processed,
                if self.presentation_leader() { "this console leads: its facts will announce it" } else { "waiting for the leader's facts to deliver it" }
            ));
        }
        self.presentation_log.push((presented, false));
    }

    /// The leader's events no tick has delivered and it has not yet announced; a console that
    /// does not lead announces again whatever is undelivered once it leads.
    fn announce_presentation(&mut self) -> Vec<Presented> {
        let leader = self.presentation_leader();
        let mut announced = Vec::new();
        for (presented, sent) in &mut self.presentation_log {
            if !leader {
                *sent = false;
            } else if !*sent && announced.len() < MAX_PRESENTED {
                *sent = true;
                announced.push(*presented);
            }
        }
        if self.campaign && !announced.is_empty() {
            crate::coop::trace(format_args!("presentation {announced:?} announced with the facts of tick {}", self.produced));
        }
        announced
    }

    /// Whether a tick's presentation event is new to the round; what is delivered leaves the log.
    fn deliver_presented(&mut self, presented: Presented) -> bool {
        let new = match presented {
            Presented::Nis { seq, .. } => {
                let new = seq > self.nis_delivered;
                self.nis_delivered = self.nis_delivered.max(seq);
                new
            }
            Presented::Speech { character, id, seq } => {
                let delivered = self.speech_delivered.entry((character, id)).or_default();
                let new = seq > *delivered;
                *delivered = (*delivered).max(seq);
                new
            }
            Presented::Fmv { seq } => {
                let new = seq > self.fmv_delivered;
                self.fmv_delivered = self.fmv_delivered.max(seq);
                new
            }
        };
        let (nis, speech, fmv) = (self.nis_delivered, &self.speech_delivered, self.fmv_delivered);
        self.presentation_log.retain(|(presented, _)| match *presented {
            Presented::Nis { seq, .. } => seq > nis,
            Presented::Speech { character, id, seq } => speech.get(&(character, id)).is_none_or(|&delivered| seq > delivered),
            Presented::Fmv { seq } => seq > fmv,
        });
        if self.campaign && !new {
            crate::coop::trace(format_args!("presentation {presented:?} announced again at tick {}: already delivered", self.processed));
        }
        new
    }

    /// What the local skater's next facts tell every console.
    fn notes(&self) -> FactsNotes {
        let left = self.leaving.iter().filter_map(|&user_id| self.player_index(user_id)).filter_map(|index| u8::try_from(index).ok()).collect();
        let prompts = self.prompts_pending.iter().take(skate2_net::protocol::MAX_PROMPTS).copied().collect();
        let outcomes = self.outcomes_pending.iter().take(skate2_net::protocol::MAX_OUTCOMES).copied().collect();
        FactsNotes { votes: self.votes.iter().map(|(&player, &score)| (player, score)).collect(), left, prompts, outcomes, ended: self.end_pending, ..FactsNotes::default() }
    }

    /// Co-op fork: a prompt this console answered, to travel with its next facts.
    pub(super) fn queue_prompt(&mut self, kind: u8, value: i32) {
        self.prompt_seq += 1;
        self.prompts_pending.push(skate2_net::protocol::Prompt { seq: self.prompt_seq, kind, value });
    }

    /// Co-op fork (M9): an outcome this console's scripts decided, to travel with its next facts.
    pub(super) fn queue_outcome(&mut self, kind: u8, activity: u64, value: i32) {
        self.outcome_seq += 1;
        self.outcomes_pending.push(skate2_net::protocol::Outcome { seq: self.outcome_seq, kind, activity, value });
    }

    /// The frame a player stands at for a tick it sends no facts for.
    fn standing_frame(&self, index: usize) -> MissionFrame {
        let player = i32::try_from(index).unwrap_or(i32::MAX);
        let frame = self.standing[index].as_ref().map_or_else(|| self.scope.shared(&MissionFrame::default()), facts::standing);
        MissionFrame { player, local: player == self.local, ..frame }
    }

    /// Whether `next_tick` would run `tick` with the round's pacing players as they stand now.
    fn tick_ready(&self, tick: u32) -> bool {
        self.players.iter().enumerate().all(|(index, &user_id)| {
            let player = i32::try_from(index).unwrap_or(i32::MAX);
            if self.disconnected.contains(&player) || !self.contributes(index) {
                return true;
            }
            if player == self.local {
                return self.local_ready(tick);
            }
            self.remote_ready(index, user_id, tick)
        })
    }

    /// Whether the local skater lets `tick` run: its facts of the tick are in, or a later tick's
    /// are (it skipped this one after a stall, [`Self::skip_ticks`]: it stands at it, as the
    /// other consoles see it).
    fn local_ready(&self, tick: u32) -> bool {
        self.local_frames.contains_key(&tick) || self.local_frames.range(tick.saturating_add(1)..).next().is_some()
    }

    /// PC port (user request): this console let `ticks` of its clock go after a stall, so that
    /// the game goes on at its pace rather than run them off in fast motion. While the local
    /// skater paces the round its facts skip those ticks: every console runs them with the
    /// skater standing at its last facts, as for a player whose run begins after a tick.
    pub(super) fn skip_ticks(&mut self, ticks: u32) {
        if self.running() && self.local_contributes() {
            self.produced = self.produced.saturating_add(ticks);
        }
    }

    /// Whether another player pacing the round lets `tick` run: its facts of the tick are in, it
    /// left, it waits at a sync point from the tick on, or a later tick of its run is in.
    fn remote_ready(&self, index: usize, user_id: u64, tick: u32) -> bool {
        let stint = self.stints[index];
        self.others.get(&user_id).is_some_and(|facts| {
            facts.frames.contains_key(&(stint, tick))
                || facts.left
                || facts.syncing.is_some_and(|(at, from)| at == stint && from <= tick)
                || facts.frames.range((stint, tick.saturating_add(1))..=(stint, u32::MAX)).next().is_some()
        })
    }

    /// The first other player, by index, whose facts the next tick waits for; none while it
    /// waits for the local skater's own.
    fn awaited(&self) -> Option<u64> {
        let tick = self.processed;
        let mut awaited = None;
        for (index, &user_id) in self.players.iter().enumerate() {
            let player = i32::try_from(index).unwrap_or(i32::MAX);
            if self.disconnected.contains(&player) || !self.contributes(index) {
                continue;
            }
            if player == self.local {
                if !self.local_ready(tick) {
                    return None;
                }
            } else if awaited.is_none() && !self.remote_ready(index, user_id, tick) {
                awaited = Some(user_id);
            }
        }
        awaited
    }

    /// A simulation tick of the round's wait at its next tick for `awaited` (none: the round ran
    /// a tick, or does not report its waits): once it has waited [`STALL_TICKS`] for the same
    /// player at the same tick, and every [`STALL_REPORT_TICKS`] after while the wait goes on,
    /// the player and tick to tell the server of.
    fn watch_stall(&mut self, awaited: Option<u64>) -> Option<(u64, u32)> {
        let Some(user_id) = awaited else {
            self.awaiting = None;
            return None;
        };
        let tick = self.processed;
        match &mut self.awaiting {
            Some(awaiting) if awaiting.user_id == user_id && awaiting.tick == tick => {
                awaiting.waited += 1;
                awaiting.waited.checked_sub(STALL_TICKS).is_some_and(|over| over % STALL_REPORT_TICKS == 0).then_some((user_id, tick))
            }
            awaiting => {
                *awaiting = Some(Awaiting { user_id, tick, waited: 1 });
                None
            }
        }
    }

    /// How many ticks from the next one are ready, up to `limit`; a tick's pacing can change
    /// before it runs, so this only spaces them.
    fn ready_lead(&self, limit: u32) -> u32 {
        (0..limit).take_while(|&ahead| self.tick_ready(self.processed.saturating_add(ahead))).fold(0, |lead, _| lead + 1)
    }

    /// The tick's frames of every player still in the round, once the facts of every player
    /// pacing it are in; the players who disconnect at it; the local skater's whole frame.
    #[expect(clippy::type_complexity, reason = "one tick's deliveries, taken apart by the caller")]
    fn next_tick(
        &mut self,
    ) -> Option<(Vec<MissionFrame>, Vec<i32>, Option<MissionFrame>, Vec<Presented>, Vec<(i32, skate2_net::protocol::Prompt)>, Vec<(i32, skate2_net::protocol::Outcome)>)> {
        let tick = self.processed;
        let mut frames = Vec::with_capacity(self.players.len());
        let mut notes = Vec::new();
        let mut disconnects = Vec::new();
        let mut own = self.own.clone();
        for (index, &user_id) in self.players.iter().enumerate() {
            let player = i32::try_from(index).unwrap_or(i32::MAX);
            if self.disconnected.contains(&player) {
                continue;
            }
            if !self.contributes(index) {
                frames.push(self.standing_frame(index));
                frames.extend(self.standing_owned[index].iter().cloned());
                continue;
            }
            if player == self.local {
                if let Some(local) = self.local_frames.get(&tick).cloned() {
                    own = Some(local.whole);
                    frames.push(local.shared);
                    self.standing_owned[index].clone_from(&local.owned);
                    frames.extend(local.owned);
                    notes.push((player, local.notes));
                } else if self.local_ready(tick) {
                    frames.push(self.standing_frame(index));
                    frames.extend(self.standing_owned[index].iter().cloned());
                } else {
                    return None;
                }
                continue;
            }
            let stint = self.stints[index];
            let scope = self.scope;
            let facts = self.others.entry(user_id).or_insert_with(|| PlayerFacts::new(scope));
            if let Some(facts) = facts.frames.get(&(stint, tick)) {
                frames.push(facts.frame.clone());
                frames.extend(facts.owned.iter().cloned());
                notes.push((player, facts.notes.clone()));
                let owned = facts.owned.clone();
                self.standing_owned[index] = owned;
            } else if facts.left {
                disconnects.push(player);
                if !self.standing_owned[index].is_empty() && self.ai_handoff.is_none() {
                    trace(format_args!("player {player} left at tick {tick} with {} owned AI skaters: they stand at its last frames until the next owner's facts carry them, {AI_HANDOFF_GRACE} ticks at most", self.standing_owned[index].len()));
                    self.ai_handoff = Some((tick, index));
                }
            } else if facts.syncing.is_some_and(|(at, from)| at == stint && from <= tick) {
                frames.push(self.standing_frame(index));
                frames.extend(self.standing_owned[index].iter().cloned());
            } else if facts.frames.range((stint, tick.saturating_add(1))..=(stint, u32::MAX)).next().is_some() {
                // A player's facts arrive in order, so a later tick of the run without this one
                // means the player began the run after it: it stands at this tick.
                trace(format_args!("player {player} sent no facts for tick {tick} of run {stint}; it stands at it"));
                frames.push(self.standing_frame(index));
                frames.extend(self.standing_owned[index].iter().cloned());
            } else {
                return None;
            }
        }
        // Co-op fork: each player's prompts, applied once each in order; the presser's own
        // pending ones are done with once its own facts carried them.
        let mut prompts = Vec::new();
        for (player, note) in &notes {
            let Some(slot) = usize::try_from(*player).ok().filter(|&slot| slot < MAX_PLAYERS) else { continue };
            for &prompt in &note.prompts {
                if prompt.seq > self.prompts_applied[slot] {
                    self.prompts_applied[slot] = prompt.seq;
                    prompts.push((*player, prompt));
                }
            }
            if *player == self.local {
                let applied = self.prompts_applied[slot];
                self.prompts_pending.retain(|prompt| prompt.seq > applied);
            }
        }
        // Co-op fork (M9): each player's outcomes, applied once each in order, the decider's own
        // pending ones done with once its own facts carried them.
        let mut outcomes = Vec::new();
        for (player, note) in &notes {
            let Some(slot) = usize::try_from(*player).ok().filter(|&slot| slot < MAX_PLAYERS) else { continue };
            for &outcome in &note.outcomes {
                if outcome.seq > self.outcomes_applied[slot] {
                    self.outcomes_applied[slot] = outcome.seq;
                    outcomes.push((*player, outcome));
                }
            }
            if *player == self.local {
                let applied = self.outcomes_applied[slot];
                self.outcomes_pending.retain(|outcome| outcome.seq > applied);
            }
        }
        if self.campaign {
            self.compare_checks(&notes);
        }
        let notes: Vec<FactsNotes> = notes.into_iter().map(|(_, note)| note).collect();
        for note in &notes {
            for &(voter, score) in &note.votes {
                if let Some(frame) = frames.iter_mut().find(|frame| frame.player == i32::from(voter)) {
                    if frame.vote != Some(score) {
                        trace(format_args!("tick {tick}: player {voter} votes {score}"));
                    }
                    frame.vote = Some(score);
                }
                if self.votes.get(&voter) == Some(&score) {
                    self.votes.remove(&voter);
                }
            }
            for &left in &note.left {
                let player = i32::from(left);
                if !self.contributes(usize::from(left)) && !self.disconnected.contains(&player) && !disconnects.contains(&player) {
                    disconnects.push(player);
                }
            }
            if note.ended && self.infinite_rounds && !self.ended {
                trace(format_args!("tick {tick}: the host ended the challenge"));
                self.ended = true;
                self.end_due = true;
            }
        }
        if self.ended {
            self.end_pending = false;
        }
        frames.retain(|frame| !disconnects.contains(&frame.player));
        for frame in &frames {
            if let Some(index) = usize::try_from(frame.player).ok().filter(|&index| index < MAX_PLAYERS && self.contributes(index)) {
                self.standing[index] = Some(frame.clone());
            }
        }
        self.local_frames.remove(&tick);
        for facts in self.others.values_mut() {
            facts.frames.retain(|&(_, at), _| at > tick);
        }
        for &player in &disconnects {
            if let Some(user_id) = usize::try_from(player).ok().and_then(|index| self.players.get(index)) {
                self.leaving.remove(user_id);
            }
        }
        self.disconnected.extend(&disconnects);
        if let Some((since, from)) = self.ai_handoff {
            let seats = self.players.len();
            let carried = frames.iter().any(|frame| usize::try_from(frame.player).is_ok_and(|slot| slot >= seats));
            if carried {
                trace(format_args!("the next owner's facts carry the AI skaters at tick {tick}: player {from}'s last frames of them go"));
                self.ai_handoff = None;
                self.standing_owned[from].clear();
            } else if tick.saturating_sub(since) > AI_HANDOFF_GRACE {
                trace(format_args!("no player's facts carried the AI skaters in {AI_HANDOFF_GRACE} ticks after player {from} left: destroyed at tick {tick}"));
                self.ai_handoff = None;
                self.standing_owned[from].clear();
            } else {
                frames.extend(self.standing_owned[from].iter().cloned());
            }
        }
        frames.sort_by_key(|frame| frame.player);
        if let Ok(mut stance) = self.stance.lock() {
            for frame in &frames {
                if let Some(row) = usize::try_from(frame.player).ok().and_then(|player| stance.get_mut(player)) {
                    *row = frame.is_regular_stance;
                }
            }
        }
        let candidates: Vec<Presented> = notes.iter().flat_map(|note| note.presented.iter().copied()).collect();
        let presented = candidates.into_iter().filter(|&presented| self.deliver_presented(presented)).collect();
        self.processed += 1;
        self.ticks += 1;
        Some((frames, disconnects, own, presented, prompts, outcomes))
    }

    /// Co-op fork (M23): the other players' round checks that came with this tick's facts,
    /// against this console's own record of the ticks they name. A first difference with a
    /// player prints this console's whole parts of the ticks around it and asks for the parts'
    /// field dump at a tick every console reaches alike ([`sync_dump_tick`]).
    fn compare_checks(&mut self, notes: &[(i32, FactsNotes)]) {
        use crate::coop::sync_check::{Compared, part_names};
        for (player, note) in notes {
            let Some(check) = note.check.filter(|_| *player != self.local) else { continue };
            let Some(&user_id) = usize::try_from(*player).ok().and_then(|index| self.players.get(index)) else { continue };
            match self.sync_check.compare(user_id, check.tick, check.parts) {
                Compared::Agrees | Compared::StillDiffers { .. } => {}
                Compared::Unknown => crate::coop::trace(format_args!("sync check of player {player} at tick {}: no record of the tick here", check.tick)),
                Compared::Differs { mask, first } => {
                    let ours = self.sync_check.parts_at(check.tick).unwrap_or_default();
                    crate::coop::trace(format_args!(
                        "sync check: player {player} differs at tick {} in {} ({}; ours {:08x?}, theirs {:04x?})",
                        check.tick,
                        part_names(mask),
                        if first { "the first difference" } else { "after agreeing again" },
                        ours,
                        check.parts
                    ));
                    for (tick, parts) in self.sync_check.window(check.tick) {
                        crate::coop::trace(format_args!("sync parts @{tick}: {parts:08x?}"));
                    }
                    if self.sync_dump_at.is_none() {
                        let at = sync_dump_tick(check.tick);
                        self.sync_dump_at = Some(([at, at + SYNC_DUMP_GRID], mask));
                    }
                }
            }
        }
    }

    /// Co-op fork (M23): each player's sync checks this round, for the trace.
    fn trace_sync_standings(&self) {
        if !self.campaign {
            return;
        }
        for (user_id, checked, differed, first) in self.sync_check.standings() {
            let first = first.map_or_else(|| "none".to_owned(), |(tick, mask)| format!("tick {tick} in {}", crate::coop::sync_check::part_names(mask)));
            crate::coop::trace(format_args!("sync check with {user_id:016x}: {checked} checks, {differed} differed, first difference {first}"));
        }
    }

    /// The players pacing the next tick, after the challenge ran this one. A player who starts
    /// sending facts begins a new run of them; the local skater sends from the next tick on, or
    /// stops and drops what it sent ahead.
    ///
    /// PC port: `restart` is the turn player of a game that runs Skate 3's own state graph whose
    /// turn, countdown or run began at this tick (`OnlineChallenge::turn_restart`). Still pacing,
    /// it begins a new run of facts too, alike on every console: what its console sent ahead
    /// while the round waited on other players was sent standing pinned, and would otherwise run
    /// off, in fast motion, against the countdown or the run timer that starts here.
    fn set_contributors(&mut self, next: u8, restart: Option<usize>) {
        for index in 0..self.players.len().min(MAX_PLAYERS) {
            let bit = 1 << index;
            let local = i32::try_from(index).is_ok_and(|index| index == self.local);
            let paced = self.contributors & bit != 0;
            if next & bit != 0 && (!paced || restart == Some(index)) {
                self.stints[index] += 1;
                if local {
                    let processed = self.processed;
                    self.produced = processed;
                    self.local_frames.retain(|&tick, _| tick < processed);
                    // What a dropped tick's facts announced goes out again with the new run's.
                    for (_, announced) in &mut self.presentation_log {
                        *announced = false;
                    }
                }
            } else if local && next & bit == 0 && paced {
                let processed = self.processed;
                self.local_frames.retain(|&tick, _| tick < processed);
            }
        }
        self.contributors = next;
    }
}

/// A Spot Battle's state as it travels (`ClientMessage::Carry`, PC port addition): its players
/// by user id, as the battle it restarts has another table. Version 1: the round; the turn
/// order with how many of it have had this round's turn; the players from the best rank down
/// with score and bonus objective; the highest vote.
const CARRY_VERSION: u8 = 1;

/// The battle's state for the server, its players named by user id; `None` if it names a
/// player the table has not.
fn encode_carry(carry: &SpotBattleCarry, players: &[u64]) -> Option<Vec<u8>> {
    use skate2_net::codec::Writer;
    let user = |index: i32| usize::try_from(index).ok().and_then(|index| players.get(index)).copied();
    let mut w = Writer::new();
    w.u8(CARRY_VERSION);
    w.varint(u64::try_from(carry.round).ok()?);
    w.u8(u8::try_from(carry.played).ok()?);
    w.u8(u8::try_from(carry.order.len()).ok()?);
    for &index in &carry.order {
        w.u64(user(index)?);
    }
    w.u8(u8::try_from(carry.ranking.len()).ok()?);
    for &(index, score, bonus) in &carry.ranking {
        w.u64(user(index)?);
        w.signed(i64::from(score));
        w.u8(u8::from(bonus));
    }
    match carry.vote.and_then(|(score, index)| Some((score, user(index)?))) {
        Some((score, voted)) => {
            w.u8(1);
            w.signed(i64::from(score));
            w.u64(voted);
        }
        None => w.u8(0),
    }
    Some(w.into_vec())
}

/// The state a restarted battle starts from, for its own table: the carried players in their
/// turn order, then the players who joined, so a joiner goes last in the round it joins; the
/// same for the ranks, a joiner with no score behind the players it joins. A carried player no
/// longer in the table is left out. `None` for bytes that are no such state.
fn decode_carry(data: &[u8], players: &[u64]) -> Option<SpotBattleCarry> {
    use skate2_net::codec::Reader;
    let index = |user_id: u64| players.iter().position(|&player| player == user_id).and_then(|index| i32::try_from(index).ok());
    let mut r = Reader::new(data);
    if r.u8().ok()? != CARRY_VERSION {
        return None;
    }
    let round = i32::try_from(r.varint().ok()?).ok().filter(|&round| round >= 1)?;
    let played = usize::from(r.u8().ok()?);
    let mut carry = SpotBattleCarry { round, ..SpotBattleCarry::default() };
    let count = usize::from(r.u8().ok()?);
    if count > MAX_PLAYERS {
        return None;
    }
    for position in 0..count {
        let Some(player) = index(r.u64().ok()?) else { continue };
        if carry.order.contains(&player) {
            return None;
        }
        carry.order.push(player);
        if position < played {
            carry.played += 1;
        }
    }
    let count = usize::from(r.u8().ok()?);
    if count > MAX_PLAYERS {
        return None;
    }
    for _ in 0..count {
        let (user_id, score, bonus) = (r.u64().ok()?, i32::try_from(r.signed().ok()?).ok()?, r.u8().ok()? != 0);
        let Some(player) = index(user_id) else { continue };
        if carry.ranking.iter().any(|&(ranked, _, _)| ranked == player) {
            return None;
        }
        carry.ranking.push((player, score, bonus));
    }
    if r.u8().ok()? != 0 {
        let (score, voted) = (i32::try_from(r.signed().ok()?).ok()?, r.u64().ok()?);
        carry.vote = index(voted).map(|player| (score, player));
    }
    r.finish().ok()?;
    for player in 0..i32::try_from(players.len()).ok()? {
        if !carry.order.contains(&player) {
            carry.order.push(player);
        }
        if !carry.ranking.iter().any(|&(ranked, _, _)| ranked == player) {
            carry.ranking.push((player, 0, false));
        }
    }
    Some(carry)
}

/// The players back end of an online challenge (`StateOnlineGame::UpdateGameInfo` 0x827538c8):
/// a `PT_Human` per player of the `GameReset` table with the player's name; the local player's
/// rows are the profile's.
struct OnlinePlayers {
    settings: Arc<PlayersBackendSettings>,
    /// The local skater's own row in the profile's back end.
    row: i32,
    local: i32,
    names: Vec<String>,
    peers: Vec<u64>,
    stance: Arc<Mutex<[bool; MAX_PLAYERS]>>,
}

impl OnlinePlayers {
    fn local_row(&self, player: i32) -> Option<i32> {
        (player == self.local).then_some(self.row)
    }

    fn count(&self) -> i32 {
        i32::try_from(self.names.len()).unwrap_or(0)
    }

    /// Co-op fork (open item 45): a player past the humans is one of this console's own
    /// challenge AI skaters, whose row the AI manager creates in the profile's back end at the
    /// skater's slot (`AISkaterManager::post`); the row answers for it.
    fn ai_row(&self, player: i32) -> Option<i32> {
        (player >= self.count() && crate::ai::skater_manager::player_type::is_ai(self.settings.player_type(player).cast_unsigned())).then_some(player)
    }

    /// The profile's row a player's settings live in: the local skater's or an AI skater's.
    fn settings_row(&self, player: i32) -> Option<i32> {
        self.local_row(player).or_else(|| self.ai_row(player))
    }
}

impl PlayersBackEnd for OnlinePlayers {
    fn player_type(&self, player: i32) -> i32 {
        if (0..self.count()).contains(&player) {
            PT_HUMAN
        } else {
            self.ai_row(player).map_or(0, |row| self.settings.player_type(row))
        }
    }
    fn player_name(&self, player: i32) -> String {
        match self.ai_row(player) {
            Some(row) => self.settings.player_name(row),
            None => usize::try_from(player).ok().and_then(|player| self.names.get(player)).cloned().unwrap_or_default(),
        }
    }
    fn is_stance_regular(&self, player: i32) -> bool {
        match self.settings_row(player) {
            Some(row) => self.settings.is_stance_regular(row),
            None => usize::try_from(player).ok().and_then(|player| self.stance.lock().ok().map(|stance| stance.get(player).copied().unwrap_or(true))).unwrap_or(true),
        }
    }
    fn peer_id(&self, player: i32) -> i64 {
        match self.ai_row(player) {
            Some(row) => self.settings.peer_id(row),
            None => usize::try_from(player).ok().and_then(|player| self.peers.get(player)).map_or(-1, |&peer| peer.cast_signed()),
        }
    }
    fn set_manuals_are_connectors(&self, player: i32, value: bool) {
        if let Some(row) = self.settings_row(player) {
            self.settings.set_manuals_are_connectors(row, value);
        }
    }
    fn set_reset_sequence(&self, player: i32, value: bool) {
        if let Some(row) = self.settings_row(player) {
            self.settings.set_reset_sequence(row, value);
        }
    }
    fn set_reset_scoring(&self, player: i32, value: bool) {
        if let Some(row) = self.settings_row(player) {
            self.settings.set_reset_scoring(row, value);
        }
    }
    fn character(&self, player: i32) -> u64 {
        self.settings_row(player).map_or(u64::MAX, |row| self.settings.character(row))
    }
    fn num_players(&self) -> i32 {
        self.count()
    }
    fn set_player_rank(&self, player: i32, rank: i32) {
        if let Some(row) = self.settings_row(player) {
            self.settings.set_player_rank(row, rank);
        }
    }
}

/// `GameReset`'s table as players in index order.
fn table(participants: &[u64; MAX_PLAYERS]) -> Vec<u64> {
    participants.iter().copied().take_while(|&user_id| user_id != 0).collect()
}

/// Whether a once-a-second diagnostic trace of a wait is due: the simulation may be paused,
/// so no tick counts the time.
fn trace_due() -> bool {
    static LAST: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    let now = std::time::Instant::now();
    let Ok(mut last) = LAST.lock() else { return false };
    if last.is_some_and(|at| now.duration_since(at) < std::time::Duration::from_secs(1)) {
        return false;
    }
    *last = Some(now);
    true
}

impl super::Online {
    /// PC port addition, Skate During Turn Wait: the game's session marker (`marker`) for this
    /// tick, by [`SelectedLiveSession::online_turn_wait_state`]: the wait's own while the skater
    /// skates through another player's turn, the challenge's from the moment that ends. Whether
    /// the wait's stands in now.
    pub(in super::super) fn swap_turn_wait_marker(&mut self, (free, in_round): (bool, bool), marker: &mut Option<crate::session_marker::SessionMarker>) -> bool {
        match (free, self.held_marker.take()) {
            (true, None) => {
                let wait = self.turn_wait_marker.take().unwrap_or_else(crate::session_marker::SessionMarker::construct);
                self.held_marker = Some(std::mem::replace(marker, Some(wait)));
                trace(format_args!("skate during turn wait: the wait's session marker stands in"));
            }
            (false, Some(held)) => {
                self.turn_wait_marker = std::mem::replace(marker, held);
                trace(format_args!("skate during turn wait: the challenge's session marker is back"));
            }
            (_, held) => self.held_marker = held,
        }
        // A round's end takes the wait's marker with it: the next round is another place.
        if !in_round {
            self.turn_wait_marker = None;
        }
        self.held_marker.is_some()
    }

    /// Whether the round's Leave Challenge is offered: the local player plays it and has not
    /// finished it (PC port addition).
    pub(in super::super) fn can_leave_challenge(&self) -> bool {
        self.round.as_ref().is_some_and(|round| round.stage == Stage::Running)
    }
}

impl<T, C> SelectedLiveSession<T, C> {
    /// The round is between `GameReset(LoadChallenge)` and lobby skate.
    pub(in super::super) fn online_challenge_running(&self) -> bool {
        self.online.round.is_some()
    }

    /// `GameReset` (`StateOnlineGame::HandleLobbySkateReset` 0x82776f90 and the challenge
    /// states): a challenge loads, starts, or the game goes back to lobby skate. A freeskate
    /// activity's reset names it (`mActivity`, `UpdateGameInfo` 0x827538c8's `mActivityKey`).
    pub(super) fn online_game_reset(&mut self, kind: u8, seed: u32, session: u32, participants: &[u64; MAX_PLAYERS], activity: u64) -> Result<(), SelectedLiveError> {
        let players = table(participants);
        let local = self.online.user_id.and_then(|user_id| players.iter().position(|&player| player == user_id));
        trace(format_args!("game reset {kind} session {session} players {players:016x?} local {local:?} activity {activity:016x}"));
        // The activity proposed loads or starts: the proposal is over (`ResetFSAProposal` at the
        // challenge's reset; PC port: at its load too, as the server's proposal ends there).
        if kind != reset::LOBBY_SKATE {
            self.online_reset_proposal()?;
        }
        // UpdateGameInfo's ResetPlayers (0x82753b44): the scoring settings a challenge gave the
        // players, such as S.K.A.T.E.'s manuals that end a line, end with the reset.
        if let Some(player) = self.inner.player.as_ref() {
            player.selected.owners.services.settings.try_reset_scoring().map_err(error)?;
        }
        if let Some(game) = self.online.game.as_mut() {
            game.phase = match kind {
                reset::LOAD_CHALLENGE => Phase::LoadingChallenge,
                reset::CHALLENGE => Phase::Challenge,
                _ => Phase::LobbySkate,
            };
            game.participants.clone_from(&players);
            // PC port: the host's end was the last challenge's.
            if kind != reset::CHALLENGE {
                game.challenge_ended = false;
            }
        }
        // A carried state is the next load's alone.
        if kind == reset::LOBBY_SKATE {
            self.online.carry = None;
        }
        self.online.lobby_hud_dirty = true;
        match kind {
            reset::LOAD_CHALLENGE => {
                let Some(local) = local else { return Ok(()) };
                // Into the challenge from watching another player: the view is the local skater's
                // again as the load's screen goes up.
                self.online_end_spectate()?;
                if self.online.round.is_some() {
                    self.end_online_round()?;
                }
                self.start_online_round(session, players, local, activity)?;
            }
            reset::CHALLENGE => {
                let matches = self.online.round.as_ref().is_some_and(|round| round.session == session);
                match local {
                    Some(local) if matches => self.start_online_challenge(seed, players, local)?,
                    _ => self.end_online_round()?,
                }
            }
            reset::CAMPAIGN => match local {
                Some(local) if self.online_campaign() => self.start_campaign_round(seed, session, players, local)?,
                _ => trace(format_args!("campaign reset ignored: local {local:?}, campaign {}", self.online_campaign())),
            },
            _ => {
                // The host's GameChange ends at a lobby reset at the new challenge.
                let changed = self.online.changing_location.take().is_some();
                if self.online.round.is_some() {
                    self.end_online_round()?;
                } else if changed {
                    self.online_travel_to_location()?;
                }
            }
        }
        Ok(())
    }

    fn online_players_backend(&self, players: &[u64], local: usize, stance: &Arc<Mutex<[bool; MAX_PLAYERS]>>) -> Result<OnlinePlayers, SelectedLiveError> {
        let player = self.inner.player.as_ref().ok_or_else(|| error("player absent"))?;
        let game = self.online.game.as_ref().ok_or_else(|| error("online game absent"))?;
        let names: Vec<String> = players.iter().map(|&user_id| game.name(user_id)).collect();
        let row = self.inner.bound_input.as_ref().and_then(|input| i32::try_from(input.player_index()).ok()).unwrap_or(0);
        let backend = OnlinePlayers {
            settings: Arc::clone(&player.selected.owners.services.settings),
            row,
            local: i32::try_from(local).unwrap_or(0),
            names,
            peers: players.to_vec(),
            stance: Arc::clone(stance),
        };
        Ok(backend)
    }

    /// `ChallengeLoadState`: the challenge system of the round and its assets: the game's
    /// challenge, or the freeskate activity its reset names (`OnlineFreeSkateActivity`, type 42,
    /// whose facts carry everything and whose players play at once).
    fn start_online_round(&mut self, session: u32, players: Vec<u64>, local: usize, activity: u64) -> Result<(), SelectedLiveError> {
        let (location, net_type) = self.online.game.as_ref().map(|game| (game.attributes.challenge_key, game.attributes.challenge_type)).ok_or_else(|| error("online game absent"))?;
        let activity = if activity == 0 { None } else { Some(self.online_round_activity(activity)?) };
        let challenge = activity.as_ref().map_or(location, |activity| activity.key);
        let game_type = if activity.is_some() { Some(crate::online::activities::CHALLENGE_TYPE) } else { catalog::game_challenge_type(net_type) };
        // A Skate 3 game that runs Skate 3's own graph reads what Skate 2's challenge of its type
        // does not: its facts go whole.
        let own_graph = activity.is_none() && crate::skate3::linked().is_some_and(|data| data.runs_own_graph(challenge));
        let scope = if own_graph { FactsScope::ALL } else { game_type.map_or(FactsScope::ALL, FactsScope::for_challenge_type) };
        let stance = Arc::new(Mutex::new([true; MAX_PLAYERS]));
        let backend = self.online_players_backend(&players, local, &stance)?;
        let local = i32::try_from(local).unwrap_or(0);
        // PC port: the host's Infinite Rounds, a Spot Battle's. The option changes in lobby skate
        // only and every member hears it before this load, so every console loads alike.
        let infinite_rounds = game_type == Some(OWN_THE_SPOT) && activity.is_none() && self.online.game.as_ref().is_some_and(|game| game.infinite_rounds);
        // PC port: the host's Round Lock, a 1-Up's that runs Skate 3's graph, where the 1-Up has
        // a round start of its number (none without one); every console reads the same game and
        // data.
        let round_lock = if own_graph && net_type == skate2_net::protocol::challenge::ONE_UP {
            let rounds = crate::skate3::linked().map_or(0, |data| data.one_up_rounds(challenge));
            self.online.game.as_ref().map(|game| game.round_lock).filter(|&round| round <= rounds).unwrap_or(0)
        } else {
            0
        };
        let online = OnlineChallenge { local_player: local, players: i32::try_from(players.len()).unwrap_or(0), infinite_rounds, round_lock, ..OnlineChallenge::default() };
        self.start_online_challenge_load(challenge, Arc::new(backend), online)?;
        trace(format_args!("loading challenge {challenge:016x} as player {local} of {}", players.len()));
        let turn_based = game_type.is_some_and(|kind| matches!(kind, 4 | 9 | 31 | 32));
        let mut round = Round::loading(session, challenge, activity, players, local, scope, turn_based, stance);
        round.bail_limit = match game_type {
            Some(SKATE) => Some(TURN_BAIL_LIMIT),
            Some(OWN_THE_SPOT) => Some(SPOT_BATTLE_BAIL_LIMIT),
            _ => None,
        };
        round.stall_reports = game_type == Some(SKATE);
        round.own_graph = own_graph;
        round.infinite_rounds = infinite_rounds;
        round.round_lock = round_lock;
        if round_lock != 0 {
            trace(format_args!("the 1-Up's rounds are locked to round {round_lock}"));
        }
        // PC port: a Spot Battle takes joiners in, and one loaded again for a joiner starts from
        // the state the server sent ahead of this load.
        round.spot_battle = game_type == Some(OWN_THE_SPOT) && round.activity.is_none() && !own_graph;
        round.carry = self.online.carry.take().filter(|_| round.spot_battle);
        self.online.round = Some(round);
        Ok(())
    }

    /// A player the challenge teleported in a turn-based round (`AI.RepositionAI` and the teleport
    /// manager's other moves, each console seeing every player's): the spot they wait on while
    /// another player's turn plays.
    pub(in super::super) fn online_note_spot(&mut self, player: i32, transform: [f32; 16]) {
        let Some(round) = self.online.round.as_mut().filter(|round| round.turn_based) else { return };
        let Some(slot) = usize::try_from(player).ok().and_then(|index| round.spots.get_mut(index)) else { return };
        let hidden = slot.is_some_and(|spot| spot.hidden);
        *slot = Some(Spot { xz: [transform[12], transform[14]], at: std::time::Instant::now(), reached: false, hidden });
    }

    /// Each frame of a turn-based round: the turn player has no spot, a waiting player whose
    /// skater shows at theirs has reached it, and one who has reached it (or had the grace to) and
    /// shows farther than [`SPOT_LIMIT`] from it while another's turn plays is not drawn.
    pub(in super::super) fn update_spectator_spots(&mut self) {
        let (turn, teleporting) = self
            .career
            .as_ref()
            .and_then(|career| career.system.world().online.as_ref())
            .map_or((None, [false; 9]), |online| (online.turn_player, online.teleporting));
        let (Some(round), Some(game)) = (self.online.round.as_mut(), self.online.game.as_ref()) else { return };
        if !round.turn_based {
            return;
        }
        let turn = turn.and_then(|player| usize::try_from(player).ok());
        if let Some(slot) = turn.and_then(|index| round.spots.get_mut(index)) {
            *slot = None;
        }
        for remote in game.remotes.values() {
            let Some(index) = round.player_index(remote.user_id) else { continue };
            let Some(spot) = round.spots.get_mut(index).and_then(Option::as_mut) else { continue };
            let Some(root) = remote.root() else { continue };
            let distance = distance_xz(root, spot.xz);
            if distance <= SPOT_REACHED {
                spot.reached = true;
            }
            let hidden = turn.is_some_and(|turn| turn != index)
                && !teleporting.get(index).copied().unwrap_or(false)
                && (spot.reached || spot.at.elapsed() > SPOT_GRACE)
                && distance > SPOT_LIMIT;
            if hidden != spot.hidden {
                trace(format_args!(
                    "{} {} their spot through another's turn ({distance:.1} m): {}",
                    game.name(remote.user_id),
                    if hidden { "left" } else { "is back at" },
                    if hidden { "not drawn" } else { "drawn" }
                ));
                spot.hidden = hidden;
            }
        }
    }

    /// Whether this console leaves a remote player's skater undrawn and unheard (see
    /// [`Self::update_spectator_spots`]).
    pub(in super::super) fn online_spot_hidden(&self, user_id: u64) -> bool {
        self.online
            .round
            .as_ref()
            .filter(|round| round.turn_based)
            .and_then(|round| round.player_index(user_id).and_then(|index| round.spots.get(index).copied().flatten()))
            .is_some_and(|spot| spot.hidden)
    }

    /// PC port (user request): in an online round of S.K.A.T.E. or Spot Battle a wipeout lasts at
    /// most [`TURN_BAIL_LIMIT`] or [`SPOT_BATTLE_BAIL_LIMIT`] before its respawn, in either physics,
    /// so Hall of Meat moves that keep the body going cannot hold the next player's turn;
    /// elsewhere the original's timing holds.
    pub(in super::super) fn sync_turn_bail_limit(&mut self) -> Result<(), SelectedLiveError> {
        let limit = self.online.round.as_ref().and_then(|round| round.bail_limit);
        if limit == self.online.turn_bail_limit {
            return Ok(());
        }
        if let Some(owner) = self.inner.world.as_ref() {
            let world = owner.world.lock().map_err(|_| error("world lock poisoned"))?;
            if let Some(services) = world.physical_world_services() {
                services.game_info().try_set_turn_bail_limit(limit).map_err(|e| error(format!("{e:?}")))?;
            }
        }
        trace(format_args!("wipeout limit {limit:?}"));
        self.online.turn_bail_limit = limit;
        Ok(())
    }

    /// Whether the round's objective groups each read their own player's state, every player's
    /// on every console: a freeskate activity's, and a Skate 3 game's that runs Skate 3's own
    /// graph (its 1-Up gives each competitor an objective group).
    pub(in super::super) fn online_round_per_player_objectives(&self) -> bool {
        self.online.round.as_ref().is_some_and(|round| round.activity.is_some() || round.own_graph || round.campaign)
    }

    /// Co-op fork: the campaign round's next tick, while one runs.
    pub(in super::super) fn campaign_round_tick(&self) -> Option<u32> {
        self.online.round.as_ref().filter(|round| round.campaign).map(|round| round.processed)
    }

    /// Co-op fork, `GameReset(CAMPAIGN)`: the campaign's round starts on every console at the same
    /// tick 0 with the server's seed and table. The career stays the career (`GM_SINGLE_PLAYER`,
    /// progression on, autosaves); what changes is who paces its scripts: from here every
    /// console runs them on every player's facts (scope "everything"), the players back end is the
    /// table's, the career's random stream is the seeded one, the objective groups are per player
    /// under the completion rules, and scenes run under the online rule.
    fn start_campaign_round(&mut self, seed: u32, session: u32, players: Vec<u64>, local: usize) -> Result<(), SelectedLiveError> {
        let stance = Arc::new(Mutex::new([true; MAX_PLAYERS]));
        let backend = self.online_players_backend(&players, local, &stance)?;
        let count = players.len();
        let local_index = i32::try_from(local).unwrap_or(0);
        let mut round = Round::loading(session, skate2_net::protocol::campaign::KEY, None, players.clone(), local_index, FactsScope::ALL, false, stance);
        round.campaign = true;
        round.start(players, local_index);
        let mut random = self.inner.bound_input.as_mut().map(|input| input.simulation_random_mut().clone()).ok_or_else(|| error("input absent"))?;
        random.set_seed_before_input_reset(seed);
        let (rules, complaint) = crate::coop::rules::RuleTable::load();
        if let Some(complaint) = complaint {
            crate::coop::trace(format_args!("rules: {complaint}"));
        }
        let rules_hash = rules.hash();
        let rules_source = rules.source.clone();
        let career = self.career.as_mut().ok_or_else(|| error("campaign round without a career"))?;
        let world = career.system.world_mut();
        world.players = Some(Arc::new(backend));
        world.local_player = local_index;
        world.online = Some(OnlineChallenge { local_player: local_index, players: i32::try_from(count).unwrap_or(0), ..OnlineChallenge::default() });
        world.coop = Some(crate::coop::rules::CampaignWorld {
            rules,
            players: (0..i32::try_from(count).unwrap_or(0)).collect(),
            seats: count,
            won: Vec::new(),
            lost: Vec::new(),
            decided: Vec::new(),
            chase_on: false,
        });
        // Co-op fork (open items 45, 53): this console's AI skaters, ambient and challenge, seat
        // themselves after the campaign's humans; one already in a human's slot leaves. M10: the
        // challenge AI skaters are one console's (`CampaignWorld::ai_owner`); every other shows
        // them as puppets of its frames and reads them from its facts.
        let ai_owner = world.coop.as_ref().map_or(0, crate::coop::rules::CampaignWorld::ai_owner);
        if let (Some(ai), Some(player)) = (self.inner.ai.as_mut(), self.inner.player.as_ref()) {
            ai.try_seat_ai_after_humans(count, local, &player.selected.owners.services.settings).map_err(error)?;
            ai.streamed_challenge_ai = local_index != ai_owner;
            ai.campaign = true;
        }
        crate::coop::trace(format_args!("campaign AI skaters: owned by player {ai_owner}{}", if local_index == ai_owner { " (this console)" } else { "; puppets here" }));
        career.online_random = Some(random);
        career.nis.set_online_game(true);
        // The design's online rule for scenes: no skip, so every console sees a scene end at the
        // tick the leader's facts announce.
        career.nis.set_skip_blocked(true);
        career.prompt_notes_on = true;
        self.retire_campaign_notice();
        if let Some(previous) = self.online.round.as_ref() {
            previous.trace_sync_standings();
        }
        self.online.round = Some(round);
        self.online.campaign_hold = false;
        crate::coop::trace(format_args!(
            "campaign round {session} started: seed {seed:08x}, {count} players, this console is player {local}, rules {rules_hash:016x} ({rules_source})"
        ));
        Ok(())
    }

    /// `GameReset(Challenge)` (`HandleLobbySkateReset` 0x82776f90): `PopAllStates`, then
    /// `ResetGame` with the server's seed and `StartMainChallenge` in `GM_OnlineChallenge` with
    /// the players who loaded. `ChallengeSkate`'s Enter: "Disabling pause of sim!"
    /// (`kPAUSE_STATE_ONLINE` pulled, with `Net::ResumeGame`); the loading screen stays up.
    fn start_online_challenge(&mut self, seed: u32, players: Vec<u64>, local: usize) -> Result<(), SelectedLiveError> {
        self.pop_all_front_end_states()?;
        if let Some(input) = self.inner.bound_input.as_ref() {
            input.try_set_online_pause(false).map_err(error)?;
        }
        let stance = self.online.round.as_ref().map(|round| Arc::clone(&round.stance)).ok_or_else(|| error("round absent"))?;
        let backend = self.online_players_backend(&players, local, &stance)?;
        let mut random = self.inner.bound_input.as_mut().map(|input| input.simulation_random_mut().clone()).ok_or_else(|| error("input absent"))?;
        random.set_seed_before_input_reset(seed);
        self.set_game_mode(GM_ONLINE_CHALLENGE, None)?;
        self.pause_menu.set_hub(crate::ui::pause_menu::StartHub::OnlineChallenge);
        let local = i32::try_from(local).unwrap_or(0);
        let count = i32::try_from(players.len()).unwrap_or(0);
        let round = self.online.round.as_mut().ok_or_else(|| error("round absent"))?;
        round.start(players, local);
        let challenge = round.challenge;
        let infinite_rounds = round.infinite_rounds;
        let round_lock = round.round_lock;
        let carry_wanted = round.spot_battle && round.join_wanted;
        let restore = round.carry.take().and_then(|data| {
            let restore = decode_carry(&data, &round.players);
            match &restore {
                Some(carry) => trace(format_args!("the battle goes on from round {}, {} of {} turns played", carry.round, carry.played, carry.order.len())),
                None => trace(format_args!("the battle's carried state ({} bytes) does not read: it starts at its first round", data.len())),
            }
            restore
        });
        let career = self.career.as_mut().ok_or_else(|| error("online challenge system absent"))?;
        let world = career.system.world_mut();
        world.players = Some(Arc::new(backend));
        world.local_player = local;
        world.online = Some(OnlineChallenge { local_player: local, players: count, infinite_rounds, round_lock, carry_wanted, restore, ..OnlineChallenge::default() });
        career.online_random = Some(random);
        career.start_online_main_challenge(challenge);
        trace(format_args!("challenge {challenge:016x} started as player {local} of {count}"));
        // ResetGame(newGame, useLocationSpawner): PlayerUI::ResetPlayer (0x828ad280) spawns the
        // local skater at its spawn of the location `UpdateGameInfo` named, the activity's, else
        // the game challenge's, under the loading screen: nothing of lobby skate (an air, a bail)
        // carries into the challenge's presentation, where a bail with the online challenge's
        // Hall of Meat effect (`IsHallOfMeatEffectEnabled` 0x827acbd0 outside free skate) would
        // draw its treatment over the cutscene. An activity's players start there (its script
        // has no sync point); the other challenges' first sync point moves them on. The game's
        // new session marker goes to that spawn.
        let spawn = self.online_spawn(challenge, u8::try_from(local).unwrap_or(0))?;
        self.online_spawn_teleport(spawn)?;
        self.online_spawn_marker(spawn, local)
    }

    /// `PlayerUI::ResetPlayer`'s spawn command for the local skater: at `transform`, on its
    /// board.
    fn online_spawn_teleport(&mut self, transform: [f32; 16]) -> Result<(), SelectedLiveError> {
        let input = self.inner.bound_input.as_ref().ok_or_else(|| error("input absent"))?;
        let player = &self.inner.player.as_ref().ok_or_else(|| error("player absent"))?.selected;
        let regular = player.owners.services.settings.try_stance_byte(input.player_index()).map_err(error)?;
        let marker = self.inner.session_marker.as_mut().ok_or_else(|| error("session marker absent"))?;
        marker.teleport.send_flow(crate::session_marker::player_ui::TeleportToPosition {
            transform: crate::skeleton_root_transforms::SkeletonRootMatrix::known(transform),
            foot_forward: u32::from(regular != 0),
            on_board: true,
            session_marker: false,
        });
        self.online_fresh_scoring()
    }

    /// The scoring of the new skater native online spawns (`ScoreModule`'s constructor ends with
    /// its `Reset`): the port keeps the local skater, so its scoring, the momentum meter's
    /// multiplier among it, resets at the next conditioner update as `SetResetScoring` resets it.
    /// Nothing of lobby skate's lines carries into the challenge, nor of one contest round's
    /// into the next.
    fn online_fresh_scoring(&self) -> Result<(), SelectedLiveError> {
        let input = self.inner.bound_input.as_ref().ok_or_else(|| error("input absent"))?;
        let settings = &self.inner.player.as_ref().ok_or_else(|| error("player absent"))?.selected.owners.services.settings;
        settings.try_set_scoring(input.player_index(), |scoring| scoring.reset_scoring = true).map_err(error)
    }

    /// `PlayerUI::ResetPlayer` after its spawn command: the game's new session marker is not
    /// valid yet, so `SetSessionMarker` (0x828acf30, forced, on the board) puts it at the spawn
    /// with the UI's stance (`ReloadAttributeData` 0x828aa470), and `cMsgSetMarker` goes out for
    /// the UI's player. Own the Spot, S.K.A.T.E. and Death Race games have no marker
    /// (`CreateSessionMarker` 0x82729c28).
    fn online_spawn_marker(&mut self, transform: [f32; 16], player: i32) -> Result<(), SelectedLiveError> {
        let input = self.inner.bound_input.as_ref().ok_or_else(|| error("input absent"))?;
        let settings = &self.inner.player.as_ref().ok_or_else(|| error("player absent"))?.selected.owners.services.settings;
        let regular = settings.try_stance_byte(input.player_index()).map_err(error)?;
        let Some(marker) = self.inner.session_marker.as_mut().and_then(|marker| marker.marker.as_mut()).filter(|marker| marker.get().is_none()) else {
            return Ok(());
        };
        marker.set_forced(&crate::skeleton_root_transforms::SkeletonRootMatrix::known(transform), u32::from(regular != 0), true);
        trace(format_args!("session marker at the spawn {:?}", [transform[12], transform[13], transform[14]]));
        let local = self.inner.world.as_ref().ok_or_else(|| error("world absent"))?.world.lock()
            .map_err(|_| error("world lock poisoned"))?
            .local_skater_identity
            .clone()
            .ok_or_else(|| error("local skater identity absent"))?;
        local.try_challenge_broadcaster().map_err(error)?.send_marker(true, player).map_err(error)
    }

    /// `StateManagerFrontEndBase::PopAllStates` (`0x82d69f18`): back to the idle HUD state, each
    /// front-end state over the game leaving with its `OnLeave` (`UpdateOnLeave` 0x82d66300),
    /// which saves nothing, and a pausing state's pause going with it.
    pub(in super::super) fn pop_all_front_end_states(&mut self) -> Result<(), SelectedLiveError> {
        // The music player, then Options (`OnLeave` 0x8266af70) under it or on its own.
        self.audio.menu.close();
        self.audio.add_song = None;
        self.options_menu.leave();
        self.skate_profile.close();
        // `FrontEndState_OnlineHubMenu::OnLeave` (0x82661228) closes its popup with its screen.
        self.close_online_hub()?;
        // The map over the StartHub: its frames return before the round's ticks.
        self.map_menu.close();
        // `FrontEndState_StartHub::OnLeave` (0x8264c4d8), then `UnpauseGame`.
        if self.pause_menu.snapshot().visible {
            self.pause_menu.close();
            if let Some(input) = self.inner.bound_input.as_ref() {
                input.try_set_frontend_pause(false).map_err(error)?;
            }
        }
        Ok(())
    }

    /// Lobby skate again (`SetupGame` 0x82778530 after the round): the round's challenge system
    /// goes and the local skater returns to the game's location.
    pub(super) fn end_online_round(&mut self) -> Result<(), SelectedLiveError> {
        self.retire_campaign_notice();
        let Some(round) = self.online.round.take() else { return Ok(()) };
        round.trace_sync_standings();
        self.end_campaign_ai_streams();
        let (ticks, bytes) = round.facts_sent;
        trace(format_args!("facts sent: {ticks} of {} ticks, {bytes} bytes", round.ticks));
        trace(format_args!("back to lobby skate"));
        // A round that ends at a sync point leaves no kPAUSE_STATE_ONLINE behind.
        if let Some(input) = self.inner.bound_input.as_ref() {
            input.try_set_online_pause(false).map_err(error)?;
        }
        if let Some(actor) = self.inner.player.as_ref().and_then(|player| player.selected.actor.as_ref()) {
            actor.try_set_pinned(false).map_err(|e| error(format!("{e:?}")))?;
        }
        self.unload_career_living_world()?;
        self.unload_career_dynamic_hulls()?;
        self.unload_career_activities()?;
        // A round left mid-vote (Leave Challenge) takes its vote down: no activity owns the
        // voting group, which only the script's DeInitVoting ends.
        if let Some(career) = self.career.as_mut() {
            crate::challenge::voting::end(career.system.world_mut());
        }
        self.end_career_hud();
        self.drop_career()?;
        // The lobby's own free-skate hulls, kept through the round, go as well: the travel waits
        // until the last game's hulls are all unloaded (GameModule::GameLoad, IsAllUnloaded
        // 0x82da1680), then the new free-skate game loads its own.
        self.unload_career_dynamic_hulls()?;
        self.freeskate.hulls = None;
        if self.online.game.as_ref().is_some_and(|game| game.entered) {
            self.set_game_mode(GM_ONLINE_FREE_SKATE, None)?;
            self.pause_menu.set_hub(crate::ui::pause_menu::StartHub::OnlineFreeSkate);
            self.freeskate.hulls = Some(self.online_dynamic_hulls()?);
            self.online_travel_to_location()?;
        }
        // The lobby's travel takes the round's loading screen over; without one it comes down.
        if round.streamer.is_some() && self.map_travel.is_none() {
            self.frontend_fade.reveal();
        }
        Ok(())
    }

    /// The local skater goes to the game's challenge location behind the fade, on its board.
    fn online_travel_to_location(&mut self) -> Result<(), SelectedLiveError> {
        let Some((attributes, slot)) = self.online.game.as_ref().filter(|game| game.entered).map(|game| (game.attributes.clone(), game.local_slot)) else { return Ok(()) };
        let start = self.online_location(&attributes, slot)?;
        self.online_load_area()?;
        // GameLoad into the game challenge's world: an additional world's challenge loads it, a
        // main world's unloads one held.
        let world_location = self.online_challenge_world(attributes.challenge_key)?;
        let world = match &world_location {
            Some((location, file)) => self.world_operation(*location, Some(file.as_str())),
            None => self.world_operation(u64::MAX, None),
        };
        // PC port: while the game is paused for the travel, Options and the Skate Profile end
        // each frame before the travel's step, under a fade that covers them: they close, with
        // the StartHub under them, as at a challenge's start. `HandleLobbySkateReset`
        // (0x82776f90) pops none, its `ResetGame` and `WaitForStreamer` going on under them.
        self.pop_all_front_end_states()?;
        self.start_world_travel(world.unwrap_or_default(), start, true);
        Ok(())
    }

    /// Another player's facts of a tick, decoded in their order whatever their run; with them
    /// (M10) the frames of the challenge AI skaters it owns, each decoded at its slot.
    pub(super) fn online_facts(&mut self, user_id: u64, tick: u32, stint: u32, notes: FactsNotes, data: &[u8], owned: &[(u8, Vec<u8>)]) {
        let Some(round) = self.online.round.as_mut() else { return };
        let Some(index) = round.players.iter().position(|&player| player == user_id) else { return };
        let scope = round.scope;
        let facts = round.others.entry(user_id).or_insert_with(|| PlayerFacts::new(scope));
        match facts.coder.decode(data, i32::try_from(index).unwrap_or(i32::MAX)) {
            Ok(frame) => {
                if notes.syncing && tick >= round.base {
                    facts.syncing = Some((stint, tick));
                }
                let mut owned_frames = Vec::with_capacity(owned.len());
                for (slot, data) in owned {
                    match facts.owned_coders.entry(*slot).or_insert_with(|| FactsCoder::new(scope)).decode(data, i32::from(*slot)) {
                        Ok(frame) => owned_frames.push(frame),
                        Err(e) => trace(format_args!("{user_id:016x}: facts of tick {tick}, owned slot {slot}: {e}")),
                    }
                }
                let slots: Vec<u8> = owned.iter().map(|(slot, _)| *slot).collect();
                if slots != facts.owned_slots {
                    trace(format_args!("{user_id:016x}: facts of tick {tick} (stint {stint}) carry the owned AI slots {slots:?} (before: {:?})", facts.owned_slots));
                    facts.owned_slots = slots;
                }
                facts.frames.insert((stint, tick), TickFacts { frame, notes, owned: owned_frames });
                while facts.frames.len() > KEPT_FACTS {
                    facts.frames.pop_last();
                }
            }
            Err(e) => trace(format_args!("{user_id:016x}: facts of tick {tick}: {e}")),
        }
    }

    /// Another player's vote press, for the notes of the next facts this console sends.
    pub(super) fn online_vote(&mut self, user_id: u64, score: i32) {
        let Some(round) = self.online.round.as_mut() else { return };
        let Some(index) = round.players.iter().position(|&player| player == user_id).and_then(|index| u8::try_from(index).ok()) else { return };
        trace(format_args!("player {index} pressed vote {score}"));
        round.votes.insert(index, score);
    }

    /// `Online.SwitchCamera` (0x8288a4e0, `SkaterManager::SetLocalIndex`): the console follows
    /// the player's skater, its own or another player's.
    pub(in super::super) fn online_switch_camera(&mut self, player: i32) {
        if let Some(round) = self.online.round.as_mut() {
            trace(format_args!("camera follows player {player}"));
            round.camera_player = Some(player);
        }
    }

    /// The remote skater the console follows: the player `Online.SwitchCamera` last switched to,
    /// when that is another player's skater that shows, or the one the pre-game lobby's Spectate
    /// shows (PC port addition).
    pub(super) fn online_spectated(&self) -> Option<&super::remote_skater::RemoteSkater> {
        let game = self.online.game.as_ref()?;
        if let Some(spectate) = self.online.spectate.as_ref() {
            return game.remotes.get(&spectate.shown()?).filter(|remote| remote.visible());
        }
        let round = self.online.round.as_ref()?;
        let player = round.camera_player.filter(|&player| player != round.local)?;
        // Co-op fork (M10): a campaign's AI slot (after the players) is an owned skater's
        // puppet here, followed as another player's skater is.
        let user_id = match round.players.get(usize::try_from(player).ok()?) {
            Some(&user_id) => user_id,
            None if round.campaign => {
                let owner = self.career.as_ref().and_then(|career| career.system.world().coop.as_ref()).map_or(0, crate::coop::rules::CampaignWorld::ai_owner);
                skate2_net::protocol::owned::skater_id(u8::try_from(owner).ok()?, u8::try_from(player).ok()?)
            }
            None => return None,
        };
        game.remotes.get(&user_id).filter(|remote| remote.visible())
    }

    /// What an online game shows in place of the game camera, as it keeps simulating under a
    /// scene (`Net::ResumeGame`): a scene's camera while one plays (a challenge presentation's
    /// NIS), else the camera of the other player `Online.SwitchCamera` follows or Spectate shows.
    fn online_view(&self) -> Option<(OnlineView, [f32; 16], f32)> {
        self.online.game.as_ref()?;
        if let (Some(scene), _) = self.career_nis_camera()
            && let Some(matrix) = scene.matrix
        {
            return Some((OnlineView::Scene, matrix, scene.fov));
        }
        if let Some(user_id) = self.online.spectate.as_ref().and_then(super::spectate::Spectate::shown) {
            let camera = self.online_spectated()?.camera()?;
            return Some((OnlineView::Spectate(user_id), camera.matrix, camera.fov));
        }
        let player = self.online.round.as_ref()?.camera_player?;
        let camera = self.online_spectated()?.camera()?;
        Some((OnlineView::Player(player), camera.matrix, camera.fov))
    }

    /// The online view's camera for the frame's visual snapshot, with this console's near plane
    /// (the scene's own while a scene shows).
    pub(in super::super) fn online_view_camera(&self) -> Option<crate::nis::manager::NisCamera> {
        let (view, matrix, fov) = self.online_view()?;
        let near_plane = match (view, self.career_nis_camera()) {
            (OnlineView::Scene, (Some(scene), _)) => scene.near_plane,
            _ => self.inner.world.as_ref()?.local.try_camera_producer_projection().ok()?.near_plane,
        };
        // PC port addition: another player's game camera shows at this console's Field of View, as
        // the restored camera shows it (`CameraRestore::view`); a scene's camera keeps its own.
        let fov = match view {
            OnlineView::Scene => fov,
            OnlineView::Player(_) | OnlineView::Spectate(_) => fov * crate::ui::options::field_of_view_scale(),
        };
        Some(crate::nis::manager::NisCamera { matrix: Some(matrix), fov, near_plane })
    }

    /// `CameraPresentation::DoSimSubmit` (0x827f23a8) in an online game: the submitted frames are
    /// the online view's camera, which the restore then shows, cut (`mbHardCut`) whenever the
    /// view changes so it never blends between a scene and a player's camera, or two players'.
    pub(in super::super) fn online_camera_submission(&mut self, camera: &mut crate::presentation_restore::camera::CameraSubmission) {
        let view = self.online_view();
        let shown = view.map(|(view, _, _)| view);
        let cut = shown != self.online.view;
        self.online.view = shown;
        for frame in &mut camera.frames {
            if let Some((_, matrix, fov)) = view {
                let matrix = crate::skeleton_root_transforms::SkeletonRootMatrix::known(matrix);
                frame.matrix = matrix;
                frame.previous_matrix = matrix;
                frame.velocity = crate::skeleton_root_transforms::SkeletonRootVector::known([0.0; 4]);
                frame.angular_velocity = crate::skeleton_root_transforms::SkeletonRootVector::known([0.0; 4]);
                frame.field_of_view = fov;
                // The console's own camera cutting (its collision) is not a cut of this view.
                frame.hard_cut = cut;
            } else {
                frame.hard_cut |= cut;
            }
        }
    }

    /// The hips and bounds of the skater the console follows, which the local skater's shadow
    /// follows as `GetLocalIndex` (0x827d0090) does.
    pub(in super::super) fn online_spectated_shadow(&self) -> Option<([f32; 3], [[f32; 3]; 2])> {
        self.online_spectated()?.visual().map(|visual| (visual.hips_position, visual.world_bounds))
    }

    /// The root of the skater the console follows, where `Pres_OnlineStreamerController::
    /// UpdateStreamer` (0x8277d310) moves the presentation's stream focus: the root of the skater
    /// at `GetLocalIndex`, which `Online.SwitchCamera` sets.
    pub(in super::super) fn online_spectated_root(&self) -> Option<[f32; 3]> {
        self.online_spectated()?.root()
    }

    /// The player whose skater the trick HUD follows: the one the console follows in a round, as
    /// `PhysOutModule_Hud::Update` (0x827c79b8) reads `GetLocal()`. The pre-game lobby's Spectate
    /// shows no trick HUD.
    pub(in super::super) fn online_spectated_user(&self) -> Option<u64> {
        if self.online.spectate.is_some() {
            return None;
        }
        self.online_spectated().map(|remote| remote.user_id)
    }

    /// PC port addition, Skate During Turn Wait (Online Settings): whether the local skater skates
    /// freely now, its own camera shown in a corner while the screen follows another player's
    /// turn. Only while the view is on that player's camera (no scene plays), the round runs and
    /// the challenge is not teleporting the local skater: a teleporting player's facts pace the
    /// round, so the skater must stand as the challenge left it from the teleport's start. A
    /// waiting player sends no facts and the round reads it at its last ones without their events
    /// (`standing_frame`), so nothing it does counts; the other consoles stop drawing it once it
    /// leaves its spot (`update_spectator_spots`), and at once where its frames say it skates so
    /// (`remote_skater::TURN_WAIT_FLAG`).
    pub(in super::super) fn online_skates_while_waiting(&self) -> bool {
        if !self.options_menu.skate_during_turn_wait() || self.online.spectate.is_some() {
            return false;
        }
        let Some(round) = self.online.round.as_ref().filter(|round| round.turn_based && round.running()) else { return false };
        let teleporting = usize::try_from(round.local).ok().is_some_and(|local| {
            self.career
                .as_ref()
                .and_then(|career| career.system.world().online.as_ref())
                .is_some_and(|online| online.teleporting.get(local).copied().unwrap_or(false))
        });
        !teleporting && matches!(self.online_view(), Some((OnlineView::Player(player), ..)) if player != round.local)
    }

    /// PC port addition, Skate During Turn Wait: whether the local skater's own view shows in its
    /// corner now. The corner draws over the HUD, so it stays away while a menu is up (the pause
    /// menu, Options and its music player, the online hub): the skater skates on unseen.
    pub(in super::super) fn online_turn_wait_preview(&self) -> bool {
        self.online_skates_while_waiting()
            && !self.pause_menu.snapshot().visible
            && !self.options_menu.visible()
            && !self.audio.menu.visible()
            && !self.online_hub_open()
    }

    /// PC port addition, Skate During Turn Wait: the session marker of the wait. While the local
    /// skater skates through another player's turn the game's marker is the wait's own, which the
    /// skater sets and goes to as in a free skate (LB with the D-pad), and which it finds again at
    /// its next wait of the round. The challenge's marker (none in a Spot Battle or a game of
    /// S.K.A.T.E., `CreateSessionMarker` 0x82729c28) is held back meanwhile and stands again from
    /// the moment the wait ends, untouched, so a turn plays with the marker the challenge gave it.
    /// Whether the skater skates through a wait now, and whether a round is on: what
    /// [`super::Online::swap_turn_wait_marker`] goes by.
    pub(in super::super) fn online_turn_wait_state(&self) -> (bool, bool) {
        (self.online_skates_while_waiting(), self.online.round.is_some())
    }

    /// The challenge pinned or unpinned the local skater (`cMsgPinPlayer`).
    pub(in super::super) fn online_local_pinned(&mut self, pin: bool) {
        let tick = self.completed_frames;
        if let Some(round) = self.online.round.as_mut() {
            round.script_pinned = pin;
            if pin {
                round.unpinned = None;
            } else if round.unpinned.is_none() {
                round.unpinned = Some(tick);
                round.turns += 1;
                round.diagnostic_entered.clear();
            }
        }
    }

    /// The local skater's tester entered a trigger volume, for diagnostics.
    pub(in super::super) fn online_local_entered(&mut self, volume: u64) {
        let tick = self.completed_frames;
        if let Some(round) = self.online.round.as_mut().filter(|round| round.unpinned.is_some()) {
            round.diagnostic_entered.push((volume, tick));
        }
    }

    /// Diagnostics: completed ticks since the local skater last entered the trigger volume
    /// (its local id) since the challenge unpinned it.
    #[must_use]
    pub fn diagnostic_online_entered(&self, volume: u64) -> Option<u32> {
        let round = self.online.round.as_ref()?;
        let &(_, tick) = round.diagnostic_entered.iter().rev().find(|&&(entered, _)| entered == volume)?;
        Some(self.completed_frames.saturating_sub(tick))
    }

    /// Diagnostics: completed ticks since the online challenge unpinned the local skater, and
    /// how many times it has.
    #[must_use]
    pub fn diagnostic_online_turn(&self) -> Option<(u32, u32)> {
        let round = self.online.round.as_ref()?;
        Some((self.completed_frames.saturating_sub(round.unpinned?), round.turns))
    }

    /// Whether a running round has the local skater pinned, for diagnostics.
    #[must_use]
    pub fn diagnostic_online_pinned(&self) -> bool {
        self.online.round.as_ref().is_some_and(|round| round.running() && round.unpinned.is_none())
    }

    /// A player left during the round: after their last facts they disconnect.
    pub(super) fn online_round_player_left(&mut self, user_id: u64) {
        if let Some(round) = self.online.round.as_mut() {
            round.player_left(user_id);
        }
    }

    /// Whether `user_id` waits in the pre-game lobby, apart from the round the local player plays:
    /// a member the round's `GameReset` table does not hold, who joined after its load (a PC port
    /// addition: the original takes no join while a challenge runs).
    pub(super) fn online_apart_from_round(&self, user_id: u64) -> bool {
        self.online.round.as_ref().is_some_and(|round| !round.players.contains(&user_id))
    }

    /// The pause menu's Leave Challenge answered yes: the round ends at the next online update,
    /// where a lobby reset ends one.
    pub(in super::super) fn online_request_leave_challenge(&mut self) {
        self.online.leave_challenge = true;
    }

    /// Leave Challenge (PC port addition): the server takes the local player out of the round,
    /// which goes on for the others as without a player who left the game, and the local game
    /// goes back to lobby skate at the game's location, where it waits in the pre-game lobby and
    /// watches the round's players as ghosts until the game is back in lobby skate.
    pub(super) fn online_leave_challenge(&mut self) -> Result<(), SelectedLiveError> {
        if !self.online.can_leave_challenge() {
            return Ok(());
        }
        trace(format_args!("leaving the challenge for the pre-game lobby"));
        self.online.send(ClientMessage::LeaveChallenge);
        if let Some(game) = self.online.game.as_mut()
            && let Some(local) = self.online.user_id
        {
            game.participants.retain(|&participant| participant != local);
        }
        self.abandon_online_round()
    }

    /// The local player abandons the round: the audio system's cMsgAbandonChallenge handler
    /// (QuitChallenge 0x824ff478) ends its mission music, and what its challenge gave the
    /// players' scoring ends as UpdateGameInfo's ResetPlayers (0x82753b44) ends it at a reset.
    fn abandon_online_round(&mut self) -> Result<(), SelectedLiveError> {
        self.quit_career_challenge_music();
        if let Some(player) = self.inner.player.as_ref() {
            player.selected.owners.services.settings.try_reset_scoring().map_err(error)?;
        }
        self.end_online_round()
    }

    /// The server took the local player out of a S.K.A.T.E. round that every other player's
    /// round waited on (`ClientMessage::Stalled`): the local game leaves it as Leave Challenge
    /// does, its facts no longer reaching anyone. After the local player's own Leave Challenge
    /// the round has already ended.
    fn online_taken_out_of_challenge(&mut self) -> Result<(), SelectedLiveError> {
        if !self.online.round.as_ref().is_some_and(|round| matches!(round.stage, Stage::Running | Stage::Complete)) {
            return Ok(());
        }
        trace(format_args!("taken out of the challenge: every other player's round waited on this console's facts"));
        self.online.event_log.add(EventText::literal("You were removed from the challenge (out of sync).".to_owned()));
        self.abandon_online_round()
    }

    /// `ChallengeJoining` (PC port addition, user request): a player waits to join the Spot
    /// Battle this console plays. Where the battle would start its next turn it holds instead and
    /// its state goes to the server (`send_spot_battle_carry`), which loads the battle again
    /// with the player in it.
    pub(super) fn online_challenge_joining(&mut self) {
        let Some(round) = self.online.round.as_mut().filter(|round| round.spot_battle) else { return };
        trace(format_args!("a player waits to join the battle"));
        round.join_wanted = true;
        if matches!(round.stage, Stage::Running)
            && let Some(online) = self.career.as_mut().and_then(|career| career.system.world_mut().online.as_mut())
        {
            online.carry_wanted = true;
        }
    }

    /// `ChallengeCarry` (PC port addition): the state the battle about to load starts from.
    pub(super) fn online_challenge_carry(&mut self, data: Vec<u8>) {
        trace(format_args!("the next battle starts from a carried state ({} bytes)", data.len()));
        self.online.carry = Some(data);
    }

    /// The battle held its next turn for a player waiting to join it
    /// (`runtime::spot_battle_join`): its state goes to the server, its players named by user
    /// id. The battle stays held until the server loads it again.
    fn send_spot_battle_carry(&mut self) {
        let Some(carry) = self.career.as_mut().and_then(|career| career.system.world_mut().online.as_mut()).and_then(|online| online.carry.take()) else { return };
        let Some(round) = self.online.round.as_ref() else { return };
        let session = round.session;
        match encode_carry(&carry, &round.players) {
            Some(data) => {
                trace(format_args!("tick {}: the battle holds at round {} for a player joining it ({} bytes)", round.processed, carry.round, data.len()));
                self.online.send(ClientMessage::Carry { session, data });
            }
            None => trace(format_args!("the battle's state names a player its table has not: {carry:?}")),
        }
    }

    /// `ChallengeEnded` (PC port addition): the host ended a Spot Battle that runs under Infinite
    /// Rounds. Every member's log says so; a console playing the battle hands it to its facts'
    /// notes, and every console's battle ends from the tick they name.
    pub(super) fn online_challenge_ended(&mut self) {
        let Some(game) = self.online.game.as_mut() else { return };
        game.challenge_ended = true;
        self.online.lobby_hud_dirty = true;
        if game.entered {
            self.online.event_log.add(EventText::literal(HOST_ENDED_CHALLENGE.to_owned()));
        }
        if let Some(round) = self.online.round.as_mut() {
            round.host_ended();
        }
    }

    /// `ChallengeLeave` (PC port addition): a player left the round for the pre-game lobby, or the
    /// server took them out of it (`ClientMessage::Stalled`). To the round's other players they
    /// leave as one who left the game does (their facts end, then they disconnect) and their
    /// skater goes at once; the members waiting see them as themselves, no longer a ghost.
    /// Everyone else's log names them. A local player still in the round was taken out of it.
    pub(super) fn online_challenge_leave(&mut self, user_id: u64) -> Result<(), SelectedLiveError> {
        let local = Some(user_id) == self.online.user_id;
        let played = !local && self.online.round.as_ref().is_some_and(|round| round.players.contains(&user_id));
        let stalled = played
            && self.online.round.as_ref().and_then(|round| round.awaiting).is_some_and(|awaiting| awaiting.reported() && awaiting.user_id == user_id);
        if played {
            self.online_round_player_left(user_id);
        }
        let Some(game) = self.online.game.as_mut() else { return Ok(()) };
        game.participants.retain(|&participant| participant != user_id);
        if local {
            return self.online_taken_out_of_challenge();
        }
        trace(format_args!("{} left the challenge{}", game.name(user_id), if stalled { ": every other player's round waited on their facts" } else { "" }));
        if played && let Some(remote) = game.remotes.get_mut(&user_id) {
            remote.hide();
        }
        if game.entered {
            let line = if stalled {
                format!("{} was removed from the challenge (out of sync).", game.name(user_id))
            } else {
                format!("{} has left the challenge.", game.name(user_id))
            };
            self.online.event_log.add(EventText::literal(line));
        }
        Ok(())
    }

    /// Co-op fork (M23): the campaign round's parts after the tick the scripts just ran, whose
    /// frames hashed to `inputs`, for the sync check; the parts' field dumps the bench asked for
    /// (`SKATE2_COOP_SYNC_DUMP=<ticks>`, every that many ticks) or a difference did; and
    /// `SKATE2_DIAGNOSTIC_DESYNC=<tick>`, a planted divergence of this console's vars at the tick.
    fn record_round_check(&mut self, inputs: u32) {
        use crate::coop::sync_check::{PARTS, Part};
        static DUMP_EVERY: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
        static PLANT_AT: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
        let Some(round) = self.online.round.as_mut().filter(|round| round.campaign) else { return };
        let Some(career) = self.career.as_mut() else { return };
        let tick = round.processed.saturating_sub(1);
        if PLANT_AT.get_or_init(|| std::env::var("SKATE2_DIAGNOSTIC_DESYNC").ok().and_then(|value| value.trim().parse().ok())).is_some_and(|at| at == tick) {
            match crate::coop::round_hash::plant_divergence(&mut career.system) {
                Some(name) => crate::coop::trace(format_args!("diagnostic: a divergence planted in {name}'s vars at tick {tick}")),
                None => crate::coop::trace(format_args!("diagnostic: no activity's vars to plant a divergence in at tick {tick}")),
            }
        }
        let mut parts = crate::coop::round_hash::round_parts(&career.system);
        parts[0] = inputs;
        round.sync_check.record(tick, parts);
        let every = DUMP_EVERY.get_or_init(|| std::env::var("SKATE2_COOP_SYNC_DUMP").ok().and_then(|value| value.trim().parse().ok()).filter(|&every: &u32| every > 0));
        let asked = round.sync_dump_at.filter(|&(at, _)| tick >= at[0]).map(|(at, mask)| (at[0], mask));
        let mask = match (asked, every.is_some_and(|every| tick % every == 0)) {
            (Some((at, mask)), periodic) => {
                round.sync_dump_at = round.sync_dump_at.and_then(|(at, mask)| (at[1] != u32::MAX && at[1] > tick).then_some(([at[1], u32::MAX], mask)));
                if at != tick {
                    crate::coop::trace(format_args!("sync dump asked for tick {at} comes at tick {tick}: the round here was past it"));
                }
                if periodic { u8::MAX } else { mask }
            }
            (None, true) => u8::MAX,
            (None, false) => return,
        };
        for (index, part) in Part::ALL.into_iter().enumerate().take(PARTS) {
            if mask & (1 << index) == 0 || part == Part::Inputs {
                continue;
            }
            for line in crate::coop::round_hash::dump(&career.system, part) {
                crate::coop::trace(format_args!("sync dump @{tick} {}: {line}", part.name()));
            }
        }
    }

    /// Co-op fork (M22): one simulation tick of a campaign round's pace and stall notice, from
    /// the lead before the round ran and whether a round tick ran.
    fn update_campaign_pace(&mut self, lead: Option<u32>, ran: bool) {
        let (rtt, frame) = (self.online.round_trip, self.completed_frames);
        let Some(round) = self.online.round.as_mut().filter(|round| round.campaign) else { return };
        let resumed = round.paced_frame.is_some_and(|last| frame.saturating_sub(last) > 1);
        round.paced_frame = Some(frame);
        let (tick, before, paced_before) = (round.processed, round.pace.hz(), round.pace.paced());
        if let Some(paced) = round.pace.update(lead, rtt) {
            match paced.lead {
                Some(lead) => crate::coop::trace(format_args!(
                    "pace at tick {tick}: timer {before} -> {} Hz, {lead} local facts not yet run (free/target {:.2}, target {:.1}, ping {:.0} ms)",
                    paced.hz, paced.ratio, paced.target, paced.ping_ms
                )),
                None => crate::coop::trace(format_args!("pace at tick {tick}: timer {before} -> {} Hz, {}", paced.hz, round.unpaced_reason())),
            }
        } else if paced_before && !round.pace.paced() {
            crate::coop::trace(format_args!("pace at tick {tick}: the game's own timer again, {}", round.unpaced_reason()));
        }
        let notice = if round.running() && round.processed > 0 && !resumed { round.stall.update(ran) } else { round.stall.pause() };
        let awaited = round.awaited();
        let shown = round.stall.shown();
        let text = EventText::plain(WAITING_FOR_SKATERS);
        match notice {
            Some(crate::coop::pace::Notice::Open) => crate::coop::trace(format_args!(
                "waiting for players at tick {tick}: no round tick for {} simulation ticks, awaiting {}, notice shown",
                crate::coop::pace::STALL_TICKS,
                awaited.map_or_else(|| "no other player's facts".to_owned(), |user_id| format!("user {user_id}'s facts"))
            )),
            Some(crate::coop::pace::Notice::Close(waited)) => {
                crate::coop::trace(format_args!("waiting for players over at tick {tick} after {waited} simulation ticks{}, notice taken down", if resumed { " (this console's own pause in the round's updates)" } else { "" }));
                self.online.event_log.retire(&text);
            }
            None => {}
        }
        if shown {
            self.online.event_log.keep(&text);
        }
    }

    /// Co-op fork (M22): a campaign round going (its end, a new round in its place) takes its
    /// "waiting for players" notice down.
    fn retire_campaign_notice(&mut self) {
        if let Some(round) = self.online.round.as_mut().filter(|round| round.campaign)
            && let Some(crate::coop::pace::Notice::Close(waited)) = round.stall.pause()
        {
            crate::coop::trace(format_args!("waiting for players over with the round after {waited} simulation ticks, notice taken down"));
            self.online.event_log.retire(&EventText::plain(WAITING_FOR_SKATERS));
        }
    }

    /// Co-op fork (M22): the timer rate a campaign round's pace sets for the 60 Hz baseline,
    /// while another player paces the round with the local skater and the round was stepped in
    /// the last frame.
    pub(in super::super) fn campaign_pace_hz(&self) -> Option<i32> {
        let frame = self.completed_frames;
        self.online
            .round
            .as_ref()
            .filter(|round| round.campaign && round.pace.paced() && round.paced_frame.is_some_and(|last| frame.saturating_sub(last) <= 1))
            .map(|round| round.pace.hz())
    }

    /// The players pacing the next challenge tick (`Round::contributors`). Pins do not count:
    /// a script pins `vars.LocalPlayerID`, a different player on each console.
    fn online_contributors(&self) -> Option<u8> {
        let round = self.online.round.as_ref()?;
        let online = self.career.as_ref()?.system.world().online.as_ref()?;
        let players = round.players.len().min(MAX_PLAYERS);
        let present = |index: usize| {
            !online.disconnected.get(index).copied().unwrap_or(false) && i32::try_from(index).is_ok_and(|player| !round.disconnected.contains(&player))
        };
        let turn = online.turn_player.and_then(|player| usize::try_from(player).ok()).filter(|&index| round.turn_based && index < players && present(index));
        let contributors = (0..players)
            .filter(|&index| present(index) && turn.is_none_or(|turn| index == turn || online.teleporting[index]))
            .fold(0_u8, |bits, index| bits | (1 << index));
        Some(contributors)
    }

    /// `StateOnlineGame`'s update of the round, which the flow runs every frame whatever
    /// front-end screen is up (a pause menu or Options open on one console would otherwise hold
    /// every player's start until it closed):
    /// - `ChallengeLoadState` (`0x82771a28`), entered once the lobby's travel is done
    ///   (`LobbySkate` handles no reset while its streamer wait is up): the round's challenge
    ///   system loads, and `GameChallengeLoaded` goes to the server once its assets are in and
    ///   the loading screen's streamer wait holds.
    /// - `ChallengeSkate` (`0x82776a78`): at a sync point, `GameSyncPoint` once the streamer wait
    ///   holds and no challenge hull is loading; otherwise the loading screen comes down once the
    ///   streamer wait holds.
    pub(in super::super) fn update_online_round(&mut self) -> Result<(), SelectedLiveError> {
        let Some(round) = self.online.round.as_ref() else { return Ok(()) };
        match round.stage {
            Stage::Loading { sent } => {
                if self.map_travel.is_some() {
                    if trace_due() {
                        trace(format_args!("challenge load waits for the travel"));
                    }
                    return Ok(());
                }
                if !round.load_entered {
                    self.enter_online_challenge_load()?;
                }
                let Some(round) = self.online.round.as_ref() else { return Ok(()) };
                let (local, challenge) = (round.local, round.challenge);
                self.career_step(Some((Vec::new(), local, None)))?;
                let streamed = !self.update_online_loading_screen(false)?;
                if !sent && trace_due() {
                    let assets = self.career.as_ref().is_some_and(|career| career.challenge_assets_loaded(challenge));
                    trace(format_args!("challenge {challenge:016x} loading: streamed {streamed}, assets {assets}"));
                }
                if !sent && streamed && self.career.as_ref().is_some_and(|career| career.challenge_assets_loaded(challenge)) {
                    trace(format_args!("challenge loaded"));
                    self.online.send(ClientMessage::ChallengeLoaded);
                    if let Some(round) = self.online.round.as_mut() {
                        round.stage = Stage::Loading { sent: true };
                    }
                }
            }
            Stage::Running | Stage::Complete => {
                let Some((sync_round, sent)) = round.sync.as_ref().map(|sync| (sync.round, sync.sent)) else {
                    self.update_online_loading_screen(true)?;
                    return Ok(());
                };
                let streamed = !self.update_online_loading_screen(false)?;
                let hulls = self.career.as_ref().is_some_and(|career| career.loading_challenge_hulls());
                if !sent && trace_due() {
                    trace(format_args!("round {sync_round} sync point waits: streamed {streamed}, hulls loading {hulls}"));
                }
                if !sent && streamed && !hulls {
                    trace(format_args!("round {sync_round} sync point reached"));
                    if let Some(sync) = self.online.round.as_mut().and_then(|round| round.sync.as_mut()) {
                        sync.sent = true;
                    }
                    self.online.send(ClientMessage::SyncPoint { round: sync_round });
                }
            }
            Stage::Results => {}
        }
        Ok(())
    }

    /// `ChallengeLoadState`'s Enter (`0x82771bfc`): "Pausing the sim!" (`kPAUSE_STATE_ONLINE`
    /// pulled and pushed, with `Net::PauseGame`), then `WaitForStreamer` at the local player's
    /// `Net::GetStreamerPosition`, its spawn point at the location of the round's challenge (the
    /// activity's for a freeskate activity, `mActivityKey`).
    fn enter_online_challenge_load(&mut self) -> Result<(), SelectedLiveError> {
        if let Some(input) = self.inner.bound_input.as_ref() {
            input.try_set_online_pause(true).map_err(error)?;
        }
        let (challenge, local) = self.online.round.as_ref().map(|round| (round.challenge, round.local)).ok_or_else(|| error("round absent"))?;
        let streamer = self.online_spawn(challenge, u8::try_from(local).unwrap_or(0))?;
        self.online_wait_for_streamer([streamer[12], streamer[13], streamer[14]]);
        if let Some(round) = self.online.round.as_mut() {
            round.load_entered = true;
        }
        Ok(())
    }

    /// `WaitForStreamer` (`0x82748ae8`): `SetLoadingState(true, true, true)` puts the loading
    /// screen up (`EnableBackground` snapped black; no pause menu, online hub or backpack opens
    /// while it holds) and the wait starts at `position`. The stream focus stays where the
    /// online game's area holds it, and `CloseAllPopups(false)` closes only popups that take no
    /// input, which an online game has none of.
    fn online_wait_for_streamer(&mut self, position: [f32; 3]) {
        trace(format_args!("loading screen up, streamer at {position:?}"));
        self.frontend_fade.cover_snap();
        if let Some(round) = self.online.round.as_mut() {
            round.streamer = Some(StreamerWait { position, ticks: 0 });
        }
    }

    /// `UpdateLoadingScreenState` (`0x82748970`): whether the loading screen's wait goes on. The
    /// area at its position (`IsAreaLoaded`, 30 m) or the stream focus (`IsStreamFocusLoaded`)
    /// has to stay loaded for more than 15 updates; then `take_down` ends the wait
    /// (`SetLoadingState(false, false, false)`: the background fades out and the pause menu
    /// opens again), and without it the count starts over.
    fn update_online_loading_screen(&mut self, take_down: bool) -> Result<bool, SelectedLiveError> {
        let Some(position) = self.online.round.as_ref().and_then(|round| round.streamer.as_ref()).map(|wait| wait.position) else { return Ok(false) };
        let terrain = self.inner.terrain.as_ref().ok_or_else(|| error("online terrain absent"))?;
        let loaded = terrain.try_area_loaded(position, 30.0).map_err(|e| error(format!("{e:?}")))?
            || terrain.try_focus_loaded().map_err(|e| error(format!("{e:?}")))?;
        let Some(round) = self.online.round.as_mut() else { return Ok(false) };
        let Some(wait) = round.streamer.as_mut() else { return Ok(false) };
        if !loaded {
            wait.ticks = 0;
            return Ok(true);
        }
        wait.ticks += 1;
        if wait.ticks <= 15 {
            return Ok(true);
        }
        if take_down {
            round.streamer = None;
            trace(format_args!("loading screen down"));
            self.frontend_fade.reveal();
        } else {
            wait.ticks = 0;
        }
        Ok(false)
    }

    /// The loading screen is up (`mbDisableBackpackAndPauseMenu`): no pause menu or online hub
    /// opens.
    pub(in super::super) fn online_loading_screen(&self) -> bool {
        self.online.round.as_ref().is_some_and(|round| round.streamer.is_some())
    }

    /// PC port (user request): a round of a linked Skate 3 data folder's online game, whose
    /// screen fades go to black where Skate 2's go to white, over the same timing.
    #[must_use]
    pub fn online_skate3_round(&self) -> bool {
        self.online.round.is_some() && self.online_game_is_skate3()
    }

    /// `ChallengeLoadState` holds the simulation (`kPAUSE_STATE_ONLINE` with `Net::PauseGame`).
    pub(in super::super) fn online_round_loading(&self) -> bool {
        self.online.round.as_ref().is_some_and(|round| round.load_entered && matches!(round.stage, Stage::Loading { .. }))
    }

    /// A simulation tick of the round: the local facts out and every challenge tick whose facts
    /// are all in (a loading round loads in `update_online_round`).
    pub(in super::super) fn update_online_challenge(&mut self) -> Result<(), SelectedLiveError> {
        let Some(round) = self.online.round.as_ref() else { return Ok(()) };
        match round.stage {
            Stage::Loading { .. } => Ok(()),
            Stage::Running | Stage::Complete => {
                let local = round.local;
                self.take_held_presentation();
                // Co-op fork: the prompts answered here since the last facts ride the next.
                if let Some(career) = self.career.as_mut() {
                    let answered = career.take_pending_prompts();
                    let decided = std::mem::take(&mut career.system.world_mut().coop_outcomes);
                    if let Some(round) = self.online.round.as_mut() {
                        for answer in answered {
                            let (kind, value) = answer.to_wire();
                            round.queue_prompt(kind, value);
                            crate::coop::trace(format_args!("prompt {answer:?} answered here: queued for the next facts"));
                        }
                        for outcome in decided {
                            // M13: every console's scripts decide alike on a human's event; the
                            // note is that human's console's to send.
                            if outcome.player() >= 0 && outcome.player() != local {
                                crate::coop::trace(format_args!("outcome {outcome:?} decided here for player {}: its own console's note carries it", outcome.player()));
                                continue;
                            }
                            let (kind, activity, value) = outcome.to_wire();
                            round.queue_outcome(kind, activity, value);
                            crate::coop::trace(format_args!("outcome {outcome:?} decided here: queued for the next facts"));
                        }
                    }
                }
                self.produce_online_facts()?;
                self.diagnostic_race_gates(local)?;
                let completed = self.completed_frames;
                let pace_lead = self.online.round.as_ref().and_then(Round::campaign_pace_lead);
                let (ticks, stalled) = self.online.round.as_mut().map_or((0, None), |round| {
                    let lead = round.ready_lead(CATCH_UP_LEAD + 1);
                    let ticks = round.ticks_to_run(lead);
                    if ticks == 0 && round.running() && round.processed > 0 {
                        round.stalls += 1;
                    }
                    if completed % 600 == 0 && round.processed > 0 {
                        trace(format_args!("round at tick {}: {} ready, stalled {} of the last 600 ticks, timer {} Hz", round.processed, lead, std::mem::take(&mut round.stalls), round.pace.hz()));
                        round.trace_sync_standings();
                    }
                    // Only a round with no tick ready waits at its next tick, which every console
                    // whose game matches the others' reaches with the same players pacing it.
                    let awaited = if round.stall_reports && lead == 0 && round.running() && round.processed > 0 { round.awaited() } else { None };
                    (ticks, round.watch_stall(awaited))
                });
                if let Some((user_id, tick)) = stalled {
                    let name = self.online.game.as_ref().map(|game| game.name(user_id)).unwrap_or_default();
                    trace(format_args!("the round has waited {STALL_TICKS} ticks or more at tick {tick} for {name}'s facts: telling the server"));
                    self.online.send(ClientMessage::Stalled { user_id, tick });
                }
                let mut ran = 0;
                for _ in 0..ticks {
                    let Some((frames, disconnects, own, presented, prompts, outcomes)) = self.online.round.as_mut().filter(|round| round.running()).and_then(Round::next_tick) else { break };
                    ran += 1;
                    if std::env::var_os("SKATE2_TRACE_ONLINE_INPUTS").is_some() {
                        self.trace_online_inputs(&frames);
                    }
                    let local = self.online.round.as_ref().map_or(0, |round| round.local);
                    let mut new_owner = None;
                    if let Some(career) = self.career.as_mut() {
                        for player in disconnects {
                            trace(format_args!("player {player} disconnected from the challenge"));
                            if let Some(owner) = career.online_player_disconnected(player) {
                                new_owner = Some(owner);
                            }
                        }
                    }
                    if let Some(owner) = new_owner {
                        self.online_ai_owner_changed(owner);
                    }
                    // PC port: the host's end of an infinite Spot Battle, from the tick whose notes
                    // named it; a battle still before its first turn takes it at a later tick, the
                    // same one on every console.
                    if self.online.round.as_ref().is_some_and(|round| round.end_due)
                        && let Some(career) = self.career.as_mut()
                        && career.online_host_ended()
                        && let Some(round) = self.online.round.as_mut()
                    {
                        trace(format_args!("tick {}: the challenge took the host's end", round.processed));
                        round.end_due = false;
                    }
                    let lead = self.online.round.as_ref().filter(|round| round.local_contributes()).map_or(0, |round| round.produced.saturating_sub(round.processed));
                    let tick = self.online.round.as_ref().map_or(0, |round| round.processed);
                    if let Some(career) = self.career.as_mut() {
                        let world = career.system.world_mut();
                        world.timers.display_lead = lead;
                        if let Some(online) = world.online.as_mut() {
                            online.tick = tick;
                        }
                    }
                    self.update_online_round_nis()?;
                    if let Some(career) = self.career.as_mut() {
                        let campaign = career.system.world().coop.is_some();
                        for presented in presented {
                            if campaign {
                                crate::coop::trace(format_args!("presentation {presented:?} delivered to the scripts at tick {tick}"));
                            }
                            career.deliver_presentation(match presented {
                                Presented::Nis { notification, .. } => HeldPresentation::Nis(notification),
                                Presented::Speech { character, id, .. } => HeldPresentation::Speech { character, id },
                                Presented::Fmv { .. } => HeldPresentation::Fmv,
                            });
                        }
                        for (player, prompt) in prompts {
                            match crate::coop::prompts::PromptAnswer::from_wire(prompt.kind, prompt.value) {
                                Some(answer) => {
                                    crate::coop::trace(format_args!("prompt {answer:?} from player {player} applied at tick {tick}"));
                                    career.apply_prompt(answer);
                                }
                                None => crate::coop::trace(format_args!("prompt kind {} from player {player} unknown at tick {tick}", prompt.kind)),
                            }
                        }
                        for (player, outcome) in outcomes {
                            match crate::coop::outcomes::Outcome::from_wire(outcome.kind, outcome.activity, outcome.value) {
                                Some(outcome) => career.system.apply_coop_outcome(player, outcome, tick),
                                None => crate::coop::trace(format_args!("outcome kind {} from player {player} unknown at tick {tick}", outcome.kind)),
                            }
                        }
                    }
                    // Co-op fork (open item 45, M10): the owned challenge AI skaters' frames came
                    // with their owner's facts, so the scripts of every console read the humans
                    // and one AI run and nothing else. An AI slot's first and last frame are its
                    // creation and destruction to the scripts, the same tick on every console.
                    // This console's own ambient skaters (each console's own, open item 53) stay
                    // out of the scripts' frames (M10.3, open item 69): their creation reached
                    // the scripts as AI players at each console's own ticks and slots.
                    if let Some(humans) = self.career.as_ref().and_then(|career| career.system.world().coop.as_ref()).map(|campaign| campaign.seats) {
                        let ambient = self.career_ai_skaters(humans, false).count();
                        if let Some(round) = self.online.round.as_mut().filter(|round| round.ambient_seen != ambient) {
                            round.ambient_seen = ambient;
                            crate::coop::trace(format_args!("{ambient} ambient skaters here at tick {tick}: each console's own, out of the scripts' frames"));
                        }
                        self.online_round_ai_messages(&frames, humans, tick);
                    }
                    let inputs = if self.online.round.as_ref().is_some_and(|round| round.campaign) { crate::coop::round_hash::inputs(&frames) } else { 0 };
                    self.career_step(Some((frames, local, own)))?;
                    self.record_round_check(inputs);
                    self.send_spot_battle_carry();
                    self.take_held_presentation();
                    let voting = self.career.as_ref().is_some_and(|career| career.system.world().voting.in_voting());
                    // PC port: a Skate 3 graph began its turn player's turn, countdown or run at
                    // this tick.
                    let restart = self
                        .career
                        .as_mut()
                        .and_then(|career| career.system.world_mut().online.as_mut())
                        .and_then(|online| std::mem::take(&mut online.turn_restart).then_some(online.turn_player).flatten())
                        .and_then(|player| usize::try_from(player).ok());
                    if let Some(contributors) = self.online_contributors()
                        && let Some(round) = self.online.round.as_mut()
                    {
                        if contributors != round.contributors {
                            trace(format_args!("tick {} paced by {contributors:#04x}", round.processed));
                        }
                        let restart = restart.filter(|&player| round.turn_based && player < MAX_PLAYERS);
                        if let Some(player) = restart {
                            let ahead = if round.local_contributes() && i32::try_from(player).is_ok_and(|player| player == round.local) {
                                round.produced.saturating_sub(round.processed)
                            } else {
                                0
                            };
                            trace(format_args!("tick {}: player {player}'s facts start afresh, {ahead} sent ahead", round.processed));
                        }
                        round.set_contributors(contributors, restart);
                        if !voting {
                            round.votes.clear();
                        }
                    }
                    if self.online.round.as_ref().is_none_or(|round| !round.running()) {
                        break;
                    }
                }
                self.update_campaign_pace(pace_lead, ran > 0);
                Ok(())
            }
            Stage::Results => Ok(()),
        }
    }

    /// `SKATE2_TRACE_ONLINE_INPUTS`: the running hash of every tick's frames every 60 ticks,
    /// without what only the owner's frame has (the local flag, its pushes); alike on every
    /// console while their challenges take the same facts. `=<from>-<to>` also prints those
    /// ticks' frames.
    fn trace_online_inputs(&mut self, frames: &[MissionFrame]) {
        use std::hash::{Hash as _, Hasher as _};
        let Some(round) = self.online.round.as_mut() else { return };
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        round.input_hash.hash(&mut hasher);
        let range = std::env::var("SKATE2_TRACE_ONLINE_INPUTS").ok().and_then(|value| {
            let (from, to) = value.split_once('-')?;
            Some(from.trim().parse::<u32>().ok()?..=to.trim().parse::<u32>().ok()?)
        });
        let tick = round.processed.saturating_sub(1);
        for frame in frames {
            let text = crate::coop::round_hash::shared_text(frame);
            if range.as_ref().is_some_and(|range| range.contains(&tick)) {
                trace(format_args!("inputs frame @{tick}: {text}"));
            }
            text.hash(&mut hasher);
        }
        round.input_hash = hasher.finish();
        if round.processed % 60 == 0 {
            trace(format_args!("inputs @{}: {:016x}", round.processed, round.input_hash));
        }
    }

    /// Co-op fork (M10): the AI slots the tick's frames hold against the last tick's: a slot
    /// that appears is `cMsgSkaterCreated` to the scripts (with the AI's ordinal, as they number
    /// it), one that goes `cMsgSkaterDestroyed`; a streamed skater's puppet hides with its last
    /// frame.
    fn online_round_ai_messages(&mut self, frames: &[MissionFrame], humans: usize, tick: u32) {
        use crate::hermes_challenge_queue::{ChallengeMessage, ChallengePayload, SkaterNotification};
        let present: BTreeSet<i32> = frames.iter().map(|frame| frame.player).filter(|&slot| usize::try_from(slot).is_ok_and(|slot| slot >= humans)).collect();
        let Some(round) = self.online.round.as_mut() else { return };
        if present == round.ai_present {
            return;
        }
        let was = std::mem::replace(&mut round.ai_present, present.clone());
        let Some(career) = self.career.as_mut() else { return };
        let mut hidden = Vec::new();
        let mut changed = Vec::new();
        let owner = crate::challenge::players::owns_challenge_ai(career.system.world());
        for (slot, notification) in was.difference(&present).map(|&slot| (slot, SkaterNotification::Destroyed)).chain(present.difference(&was).map(|&slot| (slot, SkaterNotification::Created))) {
            let world = career.system.world();
            let ordinal = crate::challenge::players::script_index(world, slot);
            let streamed = crate::challenge::players::streamed_ai_slot(world, slot);
            crate::coop::trace(format_args!("AI skater at slot {slot} (the scripts' AI player {ordinal}{}): {notification:?} at tick {tick}", if streamed { ", a puppet here" } else { "" }));
            career.system.post(super::super::career::challenge_queue_event(ChallengeMessage::new(ChallengePayload::Skater { notification, player_id: ordinal })));
            if streamed && notification == SkaterNotification::Destroyed {
                hidden.push(slot);
            }
            if owner && notification == SkaterNotification::Destroyed {
                changed.push(slot);
            }
        }
        let owner_index = self.career.as_ref().and_then(|career| career.system.world().coop.as_ref()).map_or(0, crate::coop::rules::CampaignWorld::ai_owner);
        for slot in hidden {
            self.online_hide_owned_puppet(u8::try_from(owner_index).unwrap_or(0), slot);
        }
        for slot in changed {
            self.online_owned_slot_changed(slot);
        }
    }

    /// Co-op fork (M10): a campaign round's end: no challenge AI skater is streamed here any
    /// more, and the owned skaters' puppets go.
    pub(super) fn end_campaign_ai_streams(&mut self) {
        if let Some(ai) = self.inner.ai.as_mut() {
            ai.streamed_challenge_ai = false;
            ai.campaign = false;
            ai.streamed_spawns.clear();
            let mut manager = ai.manager();
            manager.campaign_players = 1;
            manager.campaign_index = 0;
        }
        self.online_drop_owned_puppets();
    }

    /// Co-op fork (M11, open item 65): the challenge AI skaters' owner left the round and the
    /// next player owns them, at the tick every console delivers the leave. The new owner
    /// spawns them afresh from the spawn commands it had dropped as the old owner's, each at
    /// the last frame the old owner's facts carried (its path and behaviour start over), drops
    /// their puppets, and from its next facts carries their frames and streams their bodies as
    /// the old owner did; every other console keeps its puppets for the new stream.
    fn online_ai_owner_changed(&mut self, new_owner: i32) {
        let Some(round) = self.online.round.as_ref() else { return };
        let (local, tick) = (round.local, round.processed);
        let standing: Vec<MissionFrame> = round.ai_handoff.map(|(_, from)| round.standing_owned[from].clone()).unwrap_or_default();
        let mine = new_owner == local;
        crate::coop::trace(format_args!("campaign AI skaters: their owner left; owned by player {new_owner}{} from tick {tick}", if mine { " (this console)" } else { "; puppets here" }));
        if !mine {
            return;
        }
        let Some(ai) = self.inner.ai.as_mut() else { return };
        ai.streamed_challenge_ai = false;
        let spawns = std::mem::take(&mut ai.streamed_spawns);
        let mut taken = Vec::with_capacity(spawns.len());
        for (slot, command) in spawns {
            let crate::ai::skater_manager::AmbientCommand::Spawn { slot, client_type, recipe, mut transform, profile_name } = command else { continue };
            let last = standing.iter().find(|frame| usize::try_from(frame.player) == Ok(slot)).map(|frame| frame.position);
            if let Some(position) = last {
                transform[3] = [position[0], position[1], position[2], 1.0];
            }
            crate::coop::trace(format_args!("owned AI skater at slot {slot} (recipe {recipe:#018x}): this console takes it over at tick {tick}, spawned {}", last.map_or_else(|| "at its spawn".to_owned(), |position| format!("at its last frame {position:?}"))));
            ai.deferred.push(crate::ai::skater_manager::AmbientCommand::Spawn { slot, client_type, recipe, transform, profile_name });
            taken.push(slot);
        }
        for slot in taken {
            if let Ok(slot) = i32::try_from(slot) {
                self.online_drop_owned_puppet(slot);
            }
        }
    }

    /// PC port (user request): the game let `ticks` of its clock go after a stall instead of
    /// running them off in fast motion; a round the local skater paces skips them
    /// (`Round::skip_ticks`).
    pub fn clock_ticks_dropped(&mut self, ticks: u32) {
        if let Some(round) = self.online.round.as_mut() {
            round.skip_ticks(ticks);
        }
    }

    /// The local skater's completed output of this tick, as the round's next facts; with it
    /// (M10) the frames of the challenge AI skaters this console owns, at their slots.
    fn produce_online_facts(&mut self) -> Result<(), SelectedLiveError> {
        let frames = self.career_mission_frames()?;
        let owned_frames = match self.career.as_ref().and_then(|career| career.system.world().coop.as_ref().map(|campaign| (campaign.seats, crate::challenge::players::owns_challenge_ai(career.system.world())))) {
            Some((humans, true)) => self.career_ai_skater_frames(humans, true)?,
            _ => Vec::new(),
        };
        let Some(round) = self.online.round.as_mut() else { return Ok(()) };
        let Some(mut own) = frames.into_iter().find(|frame| frame.local) else { return Ok(()) };
        own.player = round.local;
        if let Some(gesture) = own.performed_gesture.filter(|_| round.unpinned.is_none()) {
            trace(format_args!("gesture {gesture} while pinned"));
        }
        round.own = Some(own.clone());
        if !round.local_contributes() {
            return Ok(());
        }
        let shared = round.scope.shared(&own);
        let data = round.coder.encode(&shared);
        let tick = round.produced;
        let mut notes = round.notes();
        notes.presented = round.announce_presentation();
        if round.campaign {
            notes.check = round.sync_check.take_unsent().map(|(tick, parts)| skate2_net::protocol::RoundCheck { tick, parts });
        }
        let stint = usize::try_from(round.local).ok().and_then(|local| round.stints.get(local)).copied().unwrap_or(0);
        let scope = round.scope;
        let mut owned = Vec::with_capacity(owned_frames.len());
        let mut owned_shared = Vec::with_capacity(owned_frames.len());
        if owned_frames.len() > skate2_net::protocol::owned::MOST {
            crate::coop::trace(format_args!("{} owned AI skaters at tick {tick}: the wire carries {}; the rest stay here", owned_frames.len(), skate2_net::protocol::owned::MOST));
        }
        for frame in owned_frames.into_iter().take(skate2_net::protocol::owned::MOST) {
            let Ok(slot) = u8::try_from(frame.player) else { continue };
            let frame = scope.shared(&frame);
            let coded = round.owned_coders.entry(slot).or_insert_with(|| FactsCoder::new(scope)).encode(&frame);
            owned.push((slot, coded));
            owned_shared.push(frame);
        }
        let owned_bytes: usize = owned.iter().map(|(_, data)| data.len()).sum();
        let slots: Vec<u8> = owned.iter().map(|(slot, _)| *slot).collect();
        if slots != round.owned_slots {
            trace(format_args!("facts of tick {tick} produced (processed {}) carry the owned AI slots {slots:?} (before: {:?})", round.processed, round.owned_slots));
            round.owned_slots = slots;
        }
        round.local_frames.insert(tick, LocalTickFacts { shared, whole: own, notes: notes.clone(), owned: owned_shared });
        round.produced += 1;
        if round.produced % 300 == 0 {
            trace(format_args!("produced {}, processed {}, paced by {:#04x}", round.produced, round.processed, round.contributors));
        }
        round.facts_sent = (round.facts_sent.0 + 1, round.facts_sent.1 + data.len() + owned_bytes);
        self.online.queue(ClientMessage::Facts { tick, stint, notes, data, owned });
        Ok(())
    }

    /// What the challenge system asked of the online game: the local result
    /// (`MESSAGE_ChallengeComplete`) and the challenge's clean-up.
    pub(in super::super) fn online_challenge_outputs(&mut self, outputs: Vec<Output>) -> Result<(), SelectedLiveError> {
        for output in outputs {
            match output {
                Output::OnlineChallengeComplete { ranking, score, time, players_choice, finish_reason } => {
                    let Some(round) = self.online.round.as_mut() else { continue };
                    let result = GameResult {
                        event_time: time,
                        score,
                        finish_reason: u32::try_from(finish_reason).unwrap_or(1),
                        ranking: u16::try_from(ranking).unwrap_or(0),
                        players_choice,
                    };
                    trace(format_args!("challenge complete: {result:?}"));
                    round.result = Some(result);
                    if round.stage == Stage::Running {
                        round.stage = Stage::Complete;
                        self.online.send(ClientMessage::Complete);
                    }
                }
                Output::OnlineChallengeCleanupDone => trace(format_args!("challenge cleaned up")),
                Output::OnlineRoundSyncPoint { round, location } => self.online_round_sync_point(round, &location)?,
                Output::OpenEndChallengeOptions(_) => self.online_script_results()?,
                _ => {}
            }
        }
        Ok(())
    }

    /// `FrontEnd.OpenOnlineContinueRetryDialog` in the round: FE state 8's online display on the
    /// script's results, without the quit (`OnEnter` clears `mEnableOnlineQuitButton`).
    fn online_script_results(&mut self) -> Result<(), SelectedLiveError> {
        let Some(challenge) = self.online.round.as_ref().map(|round| round.challenge) else { return Ok(()) };
        let game = self.online.game.as_ref().ok_or_else(|| error("online game absent"))?;
        let net_type = game.attributes.challenge_type;
        let ranked = game.attributes.ranked;
        let challenge_type = catalog::game_challenge_type(net_type).unwrap_or(0);
        let (title, icon) = self.online_challenge_presentation(challenge)?;
        self.online.results_instance += 1;
        let online = OnlineResults {
            title,
            icon,
            free_skate: matches!(challenge_type, 22 | 42),
            ranked,
            column_header: column_header(challenge_type).to_owned(),
            skate: matches!(challenge_type, 4 | 5),
            type_name: catalog::challenge_title(net_type).to_owned(),
            players_choice: -1,
            instance: self.online.results_instance,
            ..OnlineResults::default()
        };
        if let Some(career) = self.career.as_mut() {
            career.show_online_script_results(online);
        }
        Ok(())
    }

    /// `ChallengeVotingManager::Update`'s input (0x825b1a18): while the local player votes on
    /// another player's turn, X, A, B and Y (states 139, 136, 137 and 138) vote bad, OK, good and
    /// sick. The press goes to every console, and the next facts of a player pacing the round
    /// carry it, so every console counts it at one tick.
    pub(in super::super) fn online_vote_input(&mut self, input: crate::ui::pause_menu::PauseMenuInput) {
        use crate::challenge::voting::score;
        let voting = self.career.as_ref().is_some_and(|career| career.system.world().voting.shows_hud());
        // Diagnostics: SKATE2_DIAGNOSTIC_ONLINE_VOTE=<score> votes once on each turn the HUD shows.
        if let Some(round) = self.online.round.as_mut()
            && let Some(diagnostic) = std::env::var("SKATE2_DIAGNOSTIC_ONLINE_VOTE").ok().and_then(|value| value.parse::<i32>().ok())
        {
            let press = voting && !round.diagnostic_voted;
            round.diagnostic_voted = voting;
            if press {
                trace(format_args!("diagnostic vote {diagnostic}"));
                self.online_vote_press(diagnostic);
                return;
            }
        }
        if !voting {
            return;
        }
        let vote = if input.misc {
            score::BAD
        } else if input.select {
            score::OK
        } else if input.cancel {
            score::GOOD
        } else if input.option {
            score::SICK
        } else {
            return;
        };
        self.online_vote_press(vote);
    }

    fn online_vote_press(&mut self, score: i32) {
        let Some(round) = self.online.round.as_mut() else { return };
        if let Ok(local) = u8::try_from(round.local) {
            round.votes.insert(local, score);
        }
        self.online.send(ClientMessage::Vote { score });
    }

    /// The round's challenge ticks run: the NIS manager then updates with them
    /// (`update_online_round_nis`), not with each simulation tick.
    pub(in super::super) fn online_round_ticking(&self) -> bool {
        self.online.round.as_ref().is_some_and(Round::running)
    }

    /// `SimController::Update1`'s NIS manager update, once a challenge tick of an online round:
    /// the original's simulation updates were the lockstep's ticks, so a scene the script plays
    /// ends at the same tick on every console.
    fn update_online_round_nis(&mut self) -> Result<(), SelectedLiveError> {
        let Some(career) = self.career.as_mut() else { return Ok(()) };
        let world = self.inner.world.as_ref().map(|owner| &owner.world);
        let terrain = self.inner.terrain.as_deref();
        career.nis.update_simulation(&mut |start, end| super::super::nis_ground_line(world, terrain, start, end)).map_err(error)?;
        let pauses = career.take_nis_events();
        if let Some(input) = self.inner.bound_input.as_ref() {
            for paused in pauses {
                input.try_set_nis_pause(paused).map_err(error)?;
            }
        }
        Ok(())
    }

    /// The scenes' and speech lines' events the round's challenge has not taken, into the round's
    /// log: the challenge takes them at the tick the pacing leader's facts announce them.
    fn take_held_presentation(&mut self) {
        let Some(career) = self.career.as_mut() else { return };
        career.hold_presentation = true;
        let held = std::mem::take(&mut career.held_presentation);
        if let Some(round) = self.online.round.as_mut() {
            for event in held {
                round.log_presentation(event);
            }
        }
    }

    /// The round waits at a sync point with its simulation paused.
    pub(in super::super) fn online_round_syncing(&self) -> bool {
        self.online.round.as_ref().is_some_and(|round| round.sync.is_some())
    }

    /// `MESSAGE_ChallengeSyncPoint` (`StateOnlineGame::FilterImpl` 0x82771530): the challenge
    /// stops at this tick on every console, its simulation paused (`kPAUSE_STATE_ONLINE`, then
    /// `Net::PauseGame` as the round stops running), and `WaitForStreamer` puts the loading screen
    /// up at the spawn point. `ChallengeSkate` (0x82776a78) sends `GameSyncPoint` once the wait
    /// holds (`update_online_round`), and the round waits until the server has the sync point
    /// from every player.
    fn online_round_sync_point(&mut self, round: i32, location: &str) -> Result<(), SelectedLiveError> {
        let Some(local) = self.online.round.as_ref().map(|state| state.local) else { return Ok(()) };
        let spawn = match self.career.as_ref() {
            Some(career) => career.online_spawn_location(location, local)?,
            None => None,
        };
        let round = u32::try_from(round).unwrap_or(0);
        let tick = self.online.round.as_ref().map_or(0, |state| state.processed);
        trace(format_args!("round {round} sync point at {location} {:?}, tick {tick}", spawn.map(|spawn| [spawn[12], spawn[13], spawn[14]])));
        if let Some(input) = self.inner.bound_input.as_ref() {
            input.try_set_online_pause(true).map_err(error)?;
        }
        let Some(state) = self.online.round.as_mut() else { return Ok(()) };
        state.sync = Some(SyncPoint { round, spawn, sent: false });
        if state.released.remove(&round) {
            return self.online_sync_point_complete();
        }
        // The players still on their way to the sync point stand this console's skater still from
        // its next tick: its simulation is paused until every player is there.
        if state.local_contributes()
            && let Some(own) = state.own.clone()
        {
            let shared = facts::standing(&state.scope.shared(&own));
            let data = state.coder.encode(&shared);
            let tick = state.produced;
            let stint = usize::try_from(state.local).ok().and_then(|local| state.stints.get(local)).copied().unwrap_or(0);
            state.produced += 1;
            state.facts_sent = (state.facts_sent.0 + 1, state.facts_sent.1 + data.len());
            self.online.send(ClientMessage::Facts { tick, stint, notes: FactsNotes { syncing: true, ..FactsNotes::default() }, data, owned: Vec::new() });
        }
        // The message's transform is the spawn point (`GetSpawnLocation`). A location the
        // manager lacks has none (natively an unset translation): the stream focus decides.
        self.online_wait_for_streamer(spawn.map_or([0.0; 3], |spawn| [spawn[12], spawn[13], spawn[14]]));
        Ok(())
    }

    /// `GameSyncPoint` from the server: every player has reached the round's sync point.
    pub(super) fn online_sync_point_released(&mut self, round: u32) -> Result<(), SelectedLiveError> {
        let Some(state) = self.online.round.as_mut() else { return Ok(()) };
        if state.sync.as_ref().is_some_and(|sync| sync.round == round) {
            return self.online_sync_point_complete();
        }
        state.released.insert(round);
        Ok(())
    }

    /// `StateOnlineGame::OnMessage`'s sync point completion (0x8276b9cc): `kPAUSE_STATE_ONLINE`
    /// pulled and `Net::ResumeGame` (the round runs again), the local skater's `cMsgTeleport` to
    /// its spawn point, then `cMsgOnlineSyncComplete`, which every console's next challenge tick
    /// delivers. Before the teleport the original sends `cMsgSpawnPlayer` (kind `0x4D9828B9`,
    /// through the folded `SendMsg<cMsgSetPlayer>`) for the round's start location: spawn command 3
    /// (`PlayerUI::ReceiveSpawnMsg` 0x828ac8b0), so `SkaterManager::OnCommand` destroys the skater
    /// and spawns a new one. The port keeps its skater, as it does for `PlayerUI::ResetPlayer`.
    fn online_sync_point_complete(&mut self) -> Result<(), SelectedLiveError> {
        let Some(state) = self.online.round.as_mut() else { return Ok(()) };
        let Some(sync) = state.sync.take() else { return Ok(()) };
        let challenge = state.challenge;
        // Every console reached the sync point at a tick of its own (each plays its own player's
        // highlight, for one); all go on from the same tick, the next run of ticks.
        state.syncs += 1;
        state.base = state.syncs << 20;
        state.sync_check.sync_point();
        state.sync_dump_at = None;
        state.processed = state.base;
        state.produced = state.base;
        state.local_frames.clear();
        let base = state.base;
        for facts in state.others.values_mut() {
            facts.frames.retain(|&(_, at), _| at >= base);
            facts.syncing = None;
        }
        trace(format_args!("round {} sync point complete", sync.round));
        if let Some(input) = self.inner.bound_input.as_ref() {
            input.try_set_online_pause(false).map_err(error)?;
        }
        if let Some(spawn) = sync.spawn {
            self.online_sync_teleport(spawn)?;
            self.online_fresh_scoring()?;
        }
        if let Some(career) = self.career.as_mut() {
            career.online_sync_complete(challenge);
        }
        Ok(())
    }

    /// `GameAllPlayersComplete`: `SendPlayerResults` reports the local result as it stands.
    pub(super) fn online_all_players_complete(&mut self) {
        let Some(round) = self.online.round.as_mut() else { return };
        let completing = round.stage == Stage::Running;
        if completing {
            round.stage = Stage::Complete;
        }
        let result = round.result.unwrap_or_default();
        let activity = round.activity.as_ref().map(|activity| activity.key);
        if completing {
            self.online.send(ClientMessage::Complete);
        }
        self.online.send(ClientMessage::Results(result));
        // ReportLocalPlayerResults: an activity the local player finished normally is completed.
        if result.finish_reason == super::activity::NORMAL_COMPLETION
            && let Some(key) = activity
            && let Err(e) = self.online_activity_completed(key)
        {
            trace(format_args!("activity {key:016x} not marked completed: {e:?}"));
        }
    }

    /// `GameFinalResults` (`StateOnlineGame::OnMessage(MT_GameFinalResults)` 0x8276b060): each
    /// player's line into FE state 27's final results at its place ([`result_places`]), the
    /// players' choice and cash; then `DisplayFinalResults` (0x82772e58) pauses the game and
    /// shows FE state 8 in its online display.
    pub(super) fn online_final_results(&mut self, rows: &[PlayerStats]) -> Result<(), SelectedLiveError> {
        let Some(round) = self.online.round.as_mut() else { return Ok(()) };
        round.stage = Stage::Results;
        let local = self.online.user_id;
        let win = rows.iter().find(|row| Some(row.user_id) == local).is_some_and(|row| row.result.ranking == 1);
        let challenge = round.challenge;
        let places = result_places(&round.players, rows);
        trace(format_args!("final results: {rows:?}"));
        let game = self.online.game.as_ref().ok_or_else(|| error("online game absent"))?;
        let lines: Vec<OnlineResultRow> = rows
            .iter()
            .filter_map(|row| {
                let player = places.iter().position(|&user_id| user_id == row.user_id)?;
                Some(OnlineResultRow {
                    player,
                    name: game.name(row.user_id),
                    position: i32::from(row.result.ranking),
                    score: row.result.score,
                    experience: row.exp,
                    time: row.result.event_time,
                    cash: row.cash,
                    players_choice: row.result.players_choice,
                })
            })
            .collect();
        let net_type = game.attributes.challenge_type;
        let ranked = game.attributes.ranked;
        let activity = round.activity.clone();
        // An activity's results type and header follow its own type (its challenge type 42); a
        // challenge's are Death Race's time or place and score.
        let (challenge_type, results_type, header) = match activity.as_ref() {
            Some(activity) => (
                crate::online::activities::CHALLENGE_TYPE,
                crate::online::activities::results_type(activity.kind, activity.accumulation),
                crate::online::activities::column_header(activity.kind, activity.survival),
            ),
            None => {
                let challenge_type = catalog::game_challenge_type(net_type).unwrap_or(0);
                (challenge_type, if challenge_type == 19 { 3 } else { 0 }, column_header(challenge_type))
            }
        };
        let mut board = OnlineResults::leaderboard(&lines, results_type);
        // SetActiveResultsLeaderboard keeps what the script wrote into the final results: an
        // activity's status column (`Results_SetPlayerStatus`), the same on every console.
        if activity.is_some()
            && let Some(career) = self.career.as_ref()
        {
            let scripted = &career.system.world().results.final_results;
            for line in &lines {
                let lua_index = places.get(line.player).and_then(|user_id| round.players.iter().position(|player| player == user_id));
                if let (Some(status), Some(player)) = (lua_index.and_then(|index| scripted.players.get(index)).map(|player| player.status.clone()), board.players.get_mut(line.player)) {
                    player.status = status;
                }
            }
        }
        let activity_won = activity.is_some() && rows.iter().find(|row| Some(row.user_id) == local).is_some_and(|row| row.result.finish_reason == super::activity::NORMAL_COMPLETION);
        let mut by_rank: Vec<(i32, usize)> = lines.iter().map(|line| (line.position, line.player)).collect();
        by_rank.sort_unstable();
        let mut cash = vec![0; places.len()];
        let mut experience = vec![0.0; places.len()];
        for line in &lines {
            cash[line.player] = line.cash;
            experience[line.player] = line.experience;
        }
        let (title, icon) = self.online_challenge_presentation(challenge)?;
        self.online.results_instance += 1;
        let online = OnlineResults {
            title,
            icon,
            free_skate: matches!(challenge_type, 22 | 42),
            activity_won,
            ranked,
            column_header: header.to_owned(),
            skate: matches!(challenge_type, 4 | 5),
            last_round: false,
            round: false,
            type_name: catalog::challenge_title(net_type).to_owned(),
            players_choice: lines.iter().find(|line| line.players_choice).map_or(-1, |line| i32::try_from(line.player).unwrap_or(-1)),
            cash,
            experience,
            // The port saves no profile and asks no ranks and badges, so DisplayFinalResults
            // enables the quit at once.
            quit_enabled: true,
            timer: 0,
            round_timer: 0,
            instance: self.online.results_instance,
            selected: 0,
            player_by_rank: by_rank.into_iter().map(|(_, player)| player).collect(),
        };
        if let Some(career) = self.career.as_mut() {
            career.open_online_results(challenge, win, board, online);
        }
        Ok(())
    }

    /// `GetOnlineChallengeInstanceData`'s challenge `Title` and its global data's
    /// `ChallengeIcon` `fe_map_icons` name, from the challenge system that runs the round (whose
    /// database holds a linked Skate 3 game's challenge, [`crate::skate3`]).
    fn online_challenge_presentation(&mut self, challenge: u64) -> Result<(String, String), SelectedLiveError> {
        const TITLE: u64 = 0xE2A5_0788_B6C2_0C92;
        const GLOBAL_DATA: u64 = 0x2449_1BF0_0316_AA45;
        const CHALLENGE_ICON: u64 = 0xE616_ACDF_5A36_BFAC;
        let db = match self.career.as_ref() {
            Some(career) => Arc::clone(&career.system.world().db),
            None => {
                self.online_data()?;
                Arc::clone(&self.freeskate.data.as_ref().ok_or_else(|| error("data absent"))?.0)
            }
        };
        let title = db.attribute(crate::challenge::keys::class::CHALLENGES, challenge, TITLE).and_then(|value| value.text(0)).unwrap_or_default().to_owned();
        let icon = db
            .attribute(crate::challenge::keys::class::CHALLENGES, challenge, GLOBAL_DATA)
            .and_then(|global| global.ref_spec(0))
            .and_then(|(class, key)| db.attribute(class, key, CHALLENGE_ICON))
            .and_then(|icon| icon.ref_spec(0))
            .and_then(|(_, key)| db.layout_text(crate::challenge::keys::class::MAP_ICON, key, 0))
            .unwrap_or("default")
            .to_owned();
        trace(format_args!("challenge {challenge:016x}: title {title}, icon {icon}"));
        Ok((title, icon))
    }

    /// `GameTimer(DisplayResults)` (`OnlineBackEnd::StartPostEventTimer`): when the board leaves.
    pub(super) fn online_results_timer(&mut self, seconds: u64) {
        self.online.results_deadline = Some(std::time::Instant::now() + std::time::Duration::from_secs(seconds));
    }

    /// FE state 8's `OnUpdate`: FE state 27's round timer and `GetPostEventDisplayTime`'s whole
    /// seconds left.
    pub(super) fn update_online_results(&mut self) {
        if let Some(career) = self.career.as_mut() {
            career.update_online_round_timer();
        }
        let Some(deadline) = self.online.results_deadline else { return };
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        let seconds = i32::try_from(left.as_secs() + u64::from(left.subsec_nanos() > 0)).unwrap_or(i32::MAX);
        if let Some(career) = self.career.as_mut() {
            career.update_online_results(seconds, true);
        }
    }

    /// `GameExitPostChallenge`: the board goes, and the game host peer sends the next
    /// challenge (`UpdateNextChallenge`, the same challenge in an unranked game).
    pub(super) fn online_exit_post_challenge(&mut self) {
        self.online.results_deadline = None;
        if let Some(career) = self.career.as_mut()
            && career.online_results_open()
        {
            career.close_online_results();
        }
        if self.online_is_host()
            && let Some(attributes) = self.online.game.as_ref().map(|game| game.attributes.clone())
        {
            self.online.send(ClientMessage::ResetAttributes(attributes));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CATCH_UP_LEAD, CATCH_UP_TICKS, REFILL_LEAD, SMOOTH_LEAD, pace, ticks_for_lead};

    #[test]
    fn a_turn_wait_has_its_own_session_marker() {
        use crate::{session_marker::SessionMarker, skeleton_root_transforms::SkeletonRootMatrix};
        let at = |x: f32| {
            let mut matrix = [0.0; 16];
            (matrix[0], matrix[5], matrix[10], matrix[15]) = (1.0, 1.0, 1.0, 1.0);
            matrix[12] = x;
            SkeletonRootMatrix::known(matrix)
        };
        let mut challenge = SessionMarker::construct();
        challenge.set_forced(&at(10.0), 1, true);
        let mut online = super::super::Online::default();
        let mut marker = Some(challenge.clone());
        // The skater's own turn: the challenge's marker stands.
        assert!(!online.swap_turn_wait_marker((false, true), &mut marker));
        assert_eq!(marker, Some(challenge.clone()));
        // A wait begins: a marker of its own, not yet set.
        assert!(online.swap_turn_wait_marker((true, true), &mut marker));
        assert_eq!(marker.as_ref().and_then(SessionMarker::get), None);
        marker.as_mut().unwrap().set_forced(&at(50.0), 0, true);
        let wait = marker.clone();
        assert!(online.swap_turn_wait_marker((true, true), &mut marker));
        assert_eq!(marker, wait, "the wait's marker stays through the wait");
        // The wait ends: the challenge's marker is back as it was, and the next wait finds its own.
        assert!(!online.swap_turn_wait_marker((false, true), &mut marker));
        assert_eq!(marker, Some(challenge.clone()));
        assert!(online.swap_turn_wait_marker((true, true), &mut marker));
        assert_eq!(marker, wait);
        // The round ends: the challenge's is back, and the wait's goes with the round.
        assert!(!online.swap_turn_wait_marker((false, false), &mut marker));
        assert_eq!(marker, Some(challenge));
        assert!(online.swap_turn_wait_marker((true, true), &mut marker));
        assert_eq!(marker.as_ref().and_then(SessionMarker::get), None);
        // A game without a marker (a Spot Battle, S.K.A.T.E.) has one through a wait alone.
        let mut online = super::super::Online::default();
        let mut none = None;
        assert!(online.swap_turn_wait_marker((true, true), &mut none));
        assert!(none.is_some());
        assert!(!online.swap_turn_wait_marker((false, true), &mut none));
        assert_eq!(none, None);
    }

    #[test]
    fn a_few_ready_ticks_run_one_a_tick_and_a_backlog_catches_up() {
        assert_eq!(ticks_for_lead(0), 0);
        assert_eq!(ticks_for_lead(1), 1);
        assert_eq!(ticks_for_lead(SMOOTH_LEAD), 1, "a jitter buffer's worth runs at the simulation's pace");
        assert_eq!(ticks_for_lead(SMOOTH_LEAD + 1), 2);
        assert_eq!(ticks_for_lead(CATCH_UP_LEAD), 2);
        assert_eq!(ticks_for_lead(CATCH_UP_LEAD + 1), CATCH_UP_TICKS);
    }

    #[test]
    fn a_round_out_of_facts_refills_a_buffer_then_keeps_pace() {
        assert_eq!(pace(0, false), (0, true));
        assert_eq!(pace(REFILL_LEAD - 1, true), (0, true), "refilling");
        assert_eq!(pace(REFILL_LEAD, true), (1, false));
        assert_eq!(pace(1, false), (1, false), "the buffer drains one a tick");
        // Facts three ticks at a time every three ticks, in order and up to seven ticks late:
        // after the first fill the round never stops.
        let jitter = [0, 7, 2, 0, 4, 1, 3, 0, 7, 0];
        let mut late = 0;
        let arrivals: Vec<u32> = (0..200_u32)
            .map(|run| {
                late = (run * 3 + 2 + jitter[usize::try_from(run).unwrap() % jitter.len()]).max(late);
                late
            })
            .collect();
        let (mut received, mut processed, mut refilling, mut stops, mut started) = (0, 0, false, 0, false);
        for tick in 0..590 {
            received += 3 * arrivals.iter().filter(|&&at| at == tick).count();
            let (ticks, waiting) = pace(u32::try_from(received - processed).unwrap(), refilling);
            refilling = waiting;
            processed += ticks;
            started |= ticks > 0;
            if started && ticks == 0 {
                stops += 1;
            }
        }
        assert_eq!(stops, 0);
    }

    /// The final results of a round a player left hold their lines from the first place on.
    #[test]
    fn final_results_close_up_over_a_player_who_left() {
        let row = |user_id| super::PlayerStats { user_id, ..Default::default() };
        assert_eq!(super::result_places(&[10, 20], &[row(20)]), [20]);
        assert_eq!(super::result_places(&[10, 20, 30], &[row(30), row(10)]), [10, 30]);
        assert_eq!(super::result_places(&[10, 20], &[row(10), row(20)]), [10, 20]);
    }

    /// A player who leaves during the load is not in the challenge's table, which the server
    /// packs again without them: the notes must not name whoever takes their index.
    #[test]
    fn a_player_who_left_during_the_load_names_no_one_in_the_challenge() {
        let stance = std::sync::Arc::new(std::sync::Mutex::new([true; super::MAX_PLAYERS]));
        let mut round = super::Round::loading(1, 0, None, vec![10, 20, 30], 0, super::FactsScope::ALL, false, stance);
        round.player_left(20);
        assert_eq!(round.notes().left, [1]);
        round.start(vec![10, 30], 0);
        assert!(round.notes().left.is_empty());
        round.player_left(30);
        assert_eq!(round.notes().left, [1]);
        // A player the challenge already disconnected needs no note when they leave the game.
        round.disconnected.push(1);
        round.leaving.clear();
        round.player_left(30);
        assert!(round.notes().left.is_empty());
    }

    /// A battle of users 10, 20 and 30 in round 2: 20 leads on 900 with the bonus objective and
    /// the highest vote, 10 has 400, 30 nothing; the order is 10, 30, 20 and 10 has had its turn.
    fn carried() -> super::SpotBattleCarry {
        super::SpotBattleCarry { round: 2, order: vec![0, 2, 1], played: 1, ranking: vec![(1, 900, true), (0, 400, false), (2, 0, false)], vote: Some((3, 1)) }
    }

    /// The same table reads the state back as it was.
    #[test]
    fn a_battle_s_state_reads_back_for_the_same_players() {
        let players = [10, 20, 30];
        let data = super::encode_carry(&carried(), &players).expect("a state of the table's players");
        assert!(data.len() <= skate2_net::protocol::MAX_CARRY);
        assert_eq!(super::decode_carry(&data, &players), Some(carried()));
    }

    /// The battle restarted for a joiner has another table: 40 joined (and took the game's free
    /// second slot, so every later player moved up one), and 30 left meanwhile. The joiner goes
    /// last in the round and last on the board; the turn already had stays had.
    #[test]
    fn a_joiner_goes_last_in_the_round_it_joins() {
        let data = super::encode_carry(&carried(), &[10, 20, 30]).expect("a state");
        let restarted = super::decode_carry(&data, &[10, 40, 20]).expect("the state for the new table");
        assert_eq!(restarted.round, 2);
        assert_eq!(restarted.order, [0, 2, 1], "10, then 20, then the joiner");
        assert_eq!(restarted.played, 1, "10's turn");
        assert_eq!(restarted.ranking, [(2, 900, true), (0, 400, false), (1, 0, false)]);
        assert_eq!(restarted.vote, Some((3, 2)), "20's, under its new index");
        // Every player of the old order has had its turn: the joiner still plays this round.
        let done = super::SpotBattleCarry { played: 3, ..carried() };
        let data = super::encode_carry(&done, &[10, 20, 30]).expect("a state");
        let restarted = super::decode_carry(&data, &[10, 20, 30, 40]).expect("the state for the new table");
        assert_eq!((restarted.order.as_slice(), restarted.played), (&[0, 2, 1, 3][..], 3), "the joiner is the round's fourth turn");
        // A player whose vote led has left: no one's vote leads.
        let data = super::encode_carry(&carried(), &[10, 20, 30]).expect("a state");
        assert_eq!(super::decode_carry(&data, &[10, 30]).expect("the state").vote, None);
    }

    /// Bytes that are no such state start the battle afresh rather than half of one.
    #[test]
    fn bytes_that_are_no_state_read_as_none() {
        let players = [10, 20, 30];
        let data = super::encode_carry(&carried(), &players).expect("a state");
        assert_eq!(super::decode_carry(&[], &players), None);
        assert_eq!(super::decode_carry(&data[..data.len() - 1], &players), None, "cut short");
        let mut longer = data.clone();
        longer.push(0);
        assert_eq!(super::decode_carry(&longer, &players), None, "bytes left over");
        let mut other = data.clone();
        other[0] = 2;
        assert_eq!(super::decode_carry(&other, &players), None, "another version");
        let twice = super::SpotBattleCarry { order: vec![0, 0, 1], ..carried() };
        let data = super::encode_carry(&twice, &players).expect("it codes");
        assert_eq!(super::decode_carry(&data, &players), None, "a player twice in the order");
        assert_eq!(super::encode_carry(&super::SpotBattleCarry { order: vec![0, 5], ..carried() }, &players), None, "a player the table has not");
    }

    /// The host's end of an infinite battle rides the pacing player's facts until a tick counts
    /// it, once; a battle with its rounds takes none.
    #[test]
    fn the_host_s_end_rides_the_facts_to_one_tick() {
        let stance = std::sync::Arc::new(std::sync::Mutex::new([true; super::MAX_PLAYERS]));
        let frame = |player: i32| super::MissionFrame { player, local: player == 0, ..Default::default() };
        let mut round = super::Round::loading(1, 0, None, vec![10, 20], 0, super::FactsScope::ALL, true, std::sync::Arc::clone(&stance));
        round.start(vec![10, 20], 0);
        round.host_ended();
        assert!(!round.notes().ended, "a battle with its rounds has no end to take from its host");
        round.infinite_rounds = true;
        round.contributors = 0b01;
        assert!(!round.notes().ended);
        round.host_ended();
        assert!(round.notes().ended);
        // A tick whose facts went out before the end reached this console does not count it.
        round.local_frames.insert(0, super::LocalTickFacts { shared: frame(0), whole: frame(0), notes: super::FactsNotes::default(), owned: Vec::new() });
        round.local_frames.insert(1, super::LocalTickFacts { shared: frame(0), whole: frame(0), notes: round.notes(), owned: Vec::new() });
        round.local_frames.insert(2, super::LocalTickFacts { shared: frame(0), whole: frame(0), notes: round.notes(), owned: Vec::new() });
        assert!(round.next_tick().is_some());
        assert!(!round.ended && !round.end_due && round.notes().ended);
        assert!(round.next_tick().is_some());
        assert!(round.ended && round.end_due, "the tick whose notes name the end");
        assert!(!round.notes().ended, "later facts say nothing more of it");
        round.end_due = false;
        assert!(round.next_tick().is_some());
        assert!(!round.end_due, "a note of an end already counted counts for nothing");
        round.host_ended();
        assert!(!round.notes().ended);
        // Another console, watching: the turn player's facts bring it the end.
        let mut watching = super::Round::loading(1, 0, None, vec![10, 20], 1, super::FactsScope::ALL, true, stance);
        watching.start(vec![10, 20], 1);
        watching.infinite_rounds = true;
        watching.contributors = 0b01;
        watching.stints[0] = 1;
        let mut facts = super::PlayerFacts::new(super::FactsScope::ALL);
        facts.frames.insert((1, 0), super::TickFacts { frame: frame(0), notes: super::FactsNotes { ended: true, ..Default::default() }, owned: Vec::new() });
        watching.others.insert(10, facts);
        assert!(watching.next_tick().is_some());
        assert!(watching.ended && watching.end_due);
    }

    /// A stall the console let go skips the local skater's facts over its ticks: every console,
    /// this one too, runs those ticks with the skater standing at its last facts.
    #[test]
    fn ticks_a_stall_let_go_stand_at_the_last_facts() {
        let stance = std::sync::Arc::new(std::sync::Mutex::new([true; super::MAX_PLAYERS]));
        let mut round = super::Round::loading(1, 0, None, vec![10, 20], 0, super::FactsScope::ALL, false, stance);
        round.start(vec![10, 20], 0);
        round.contributors = 0b01;
        let frame = |x: f32, pushing: bool| super::MissionFrame { player: 0, local: true, position: [x, 0.0, 0.0], is_pushing: pushing, ..Default::default() };
        let produce = |round: &mut super::Round, frame: super::MissionFrame| {
            round.local_frames.insert(round.produced, super::LocalTickFacts { shared: frame.clone(), whole: frame, notes: super::FactsNotes::default(), owned: Vec::new() });
            round.produced += 1;
        };
        produce(&mut round, frame(1.0, true));
        assert_eq!(round.ready_lead(10), 1);
        round.skip_ticks(2);
        produce(&mut round, frame(4.0, false));
        assert_eq!(round.ready_lead(10), 4, "the skipped ticks run");
        let local = |round: &mut super::Round| round.next_tick().map(|(frames, ..)| frames.into_iter().find(|frame| frame.player == 0).map(|frame| (frame.position[0], frame.is_pushing)));
        assert_eq!(local(&mut round), Some(Some((1.0, true))));
        for _ in 0..2 {
            assert_eq!(local(&mut round), Some(Some((1.0, false))), "standing at its last facts");
        }
        assert_eq!(local(&mut round), Some(Some((4.0, false))));
        assert!(round.next_tick().is_none());
        // While another player paces the round the local skater sends nothing to skip.
        round.contributors = 0b10;
        let produced = round.produced;
        round.skip_ticks(5);
        assert_eq!(round.produced, produced);
    }

    /// A refilling round runs the ticks it has once they have waited `REFILL_WAIT` simulation
    /// ticks for more: a player whose teleport is done at once paces two ticks and sends no more
    /// until the round has run them. A round the local skater alone paces buffers nothing.
    #[test]
    fn a_refill_that_stops_filling_runs_what_it_has() {
        let stance = std::sync::Arc::new(std::sync::Mutex::new([true; super::MAX_PLAYERS]));
        let mut round = super::Round::loading(1, 0, None, vec![10, 20], 0, super::FactsScope::ALL, true, stance);
        round.start(vec![10, 20], 0);
        round.contributors = 0b11;
        assert_eq!(round.ticks_to_run(0), 0, "out of ticks");
        assert!(round.refilling);
        for _ in 0..super::REFILL_WAIT {
            assert_eq!(round.ticks_to_run(2), 0, "two ticks ready wait for the buffer");
        }
        assert_eq!(round.ticks_to_run(2), 1, "no more came: they run");
        assert!(!round.refilling);
        assert_eq!(round.ticks_to_run(1), 1);
        // A buffer that fills runs as before.
        assert_eq!(round.ticks_to_run(0), 0);
        assert_eq!(round.ticks_to_run(REFILL_LEAD - 1), 0);
        assert_eq!(round.ticks_to_run(REFILL_LEAD), 1);
        // No facts at all is a wait for the player, however long.
        assert_eq!(round.ticks_to_run(0), 0);
        for _ in 0..4 * super::REFILL_WAIT {
            assert_eq!(round.ticks_to_run(0), 0);
        }
        assert_eq!(round.ticks_to_run(1), 0, "the count starts with the first ready tick");
        // Alone, the local skater's ticks run as they are produced.
        round.contributors = 0b01;
        assert_eq!(round.ticks_to_run(1), 1);
        assert!(!round.refilling);
    }

    /// A Skate 3 graph's turn player whose turn, countdown or run begins while it already paces
    /// the round starts its facts afresh: what it sent ahead is dropped on its own console and
    /// passed over on the others', and the round goes on with the next tick's new facts.
    #[test]
    fn a_turn_that_begins_starts_its_players_facts_afresh() {
        let stance = std::sync::Arc::new(std::sync::Mutex::new([true; super::MAX_PLAYERS]));
        let frame = |player: i32, x: f32| super::MissionFrame { player, local: player == 0, position: [x, 0.0, 0.0], ..Default::default() };
        // The turn player's own console: three ticks sent ahead while the round waited on player 1.
        let mut round = super::Round::loading(1, 0, None, vec![10, 20], 0, super::FactsScope::ALL, true, stance.clone());
        round.start(vec![10, 20], 0);
        round.contributors = 0b11;
        round.processed = 100;
        round.produced = 100;
        for x in [1.0, 2.0, 3.0] {
            round.local_frames.insert(round.produced, super::LocalTickFacts { shared: frame(0, x), whole: frame(0, x), notes: super::FactsNotes::default(), owned: Vec::new() });
            round.produced += 1;
        }
        round.presentation_log.push((super::Presented::Nis { seq: 1, notification: 7 }, true));
        assert!(!round.paced_by_local_alone());
        // Player 1's teleport is done and the turn begins at this tick.
        round.set_contributors(0b01, Some(0));
        assert_eq!((round.stints[0], round.stints[1], round.produced, round.processed), (1, 0, 100, 100));
        assert!(round.local_frames.is_empty(), "what it sent ahead is dropped");
        assert!(round.presentation_log.iter().all(|&(_, announced)| !announced), "and what that announced goes out again");
        assert!(round.paced_by_local_alone());
        assert_eq!(round.ready_lead(10), 0, "the round waits for the new run's first facts");
        round.local_frames.insert(round.produced, super::LocalTickFacts { shared: frame(0, 9.0), whole: frame(0, 9.0), notes: super::FactsNotes::default(), owned: Vec::new() });
        round.produced += 1;
        let local = round.next_tick().and_then(|(frames, ..)| frames.into_iter().find(|frame| frame.player == 0)).map(|frame| frame.position[0]);
        assert_eq!(local, Some(9.0));
        // No restart leaves a pacing player's run as it is.
        round.set_contributors(0b01, None);
        assert_eq!(round.stints[0], 1);
        // Another console: the turn player's old run is passed over from the tick on.
        let mut other = super::Round::loading(1, 0, None, vec![10, 20], 1, super::FactsScope::ALL, true, stance);
        other.start(vec![10, 20], 1);
        other.contributors = 0b01;
        other.processed = 100;
        let facts = other.others.entry(10).or_insert_with(|| super::PlayerFacts::new(super::FactsScope::ALL));
        for tick in 100..103 {
            facts.frames.insert((0, tick), super::TickFacts { frame: frame(0, 1.0), notes: super::FactsNotes::default(), owned: Vec::new() });
        }
        assert_eq!(other.ready_lead(10), 3);
        other.set_contributors(0b01, Some(0));
        assert_eq!(other.stints[0], 1);
        assert_eq!(other.ready_lead(10), 0, "the old run's ticks no longer count");
        assert_eq!(other.awaited(), Some(10));
        other.others.get_mut(&10).unwrap().frames.insert((1, 100), super::TickFacts { frame: frame(0, 9.0), notes: super::FactsNotes::default(), owned: Vec::new() });
        let turn = other.next_tick().and_then(|(frames, ..)| frames.into_iter().find(|frame| frame.player == 0)).map(|frame| frame.position[0]);
        assert_eq!(turn, Some(9.0));
        // A player who only now starts to pace begins its run as before, from the next tick.
        other.produced = 0;
        other.set_contributors(0b11, None);
        assert_eq!((other.stints[0], other.stints[1], other.produced), (1, 1, other.processed));
    }

    /// A round waiting at a tick for one other player's facts tells the server after
    /// `STALL_TICKS` of it, then every `STALL_REPORT_TICKS` while the wait goes on; a tick run,
    /// or another player waited on, counts again.
    #[test]
    fn a_round_waiting_on_one_player_keeps_reporting_them() {
        let stance = std::sync::Arc::new(std::sync::Mutex::new([true; super::MAX_PLAYERS]));
        let mut round = super::Round::loading(1, 0, None, vec![10, 20, 30], 0, super::FactsScope::ALL, true, stance);
        round.start(vec![10, 20, 30], 0);
        round.processed = 50;
        // Player 30's turn: its facts pace the round, and none came.
        round.contributors = 0b100;
        assert_eq!(round.awaited(), Some(30));
        for _ in 1..super::STALL_TICKS {
            assert_eq!(round.watch_stall(Some(30)), None);
        }
        assert_eq!(round.watch_stall(Some(30)), Some((30, 50)));
        for _ in 1..super::STALL_REPORT_TICKS {
            assert_eq!(round.watch_stall(Some(30)), None);
        }
        assert_eq!(round.watch_stall(Some(30)), Some((30, 50)), "again while the wait goes on");
        assert_eq!(round.watch_stall(None), None);
        assert!(round.awaiting.is_none());
        assert_eq!(round.watch_stall(Some(20)), None);
        assert_eq!(round.awaiting.map(|awaiting| awaiting.waited), Some(1));
        // The local skater pacing the tick without its own facts yet waits on no one else.
        round.contributors = 0b101;
        assert_eq!(round.awaited(), None);
        // A player who left lets the tick run.
        round.contributors = 0b100;
        round.player_left(30);
        assert_eq!(round.awaited(), None);
    }
}
