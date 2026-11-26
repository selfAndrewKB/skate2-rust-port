//! Co-op fork: the campaign's part of an online round, beside upstream's `online_challenge.rs`
//! (its child module, so it reads the round's own state). [`CoopRound`] is the one field the
//! fork adds to `Round`; the round's and the live session's campaign methods live here, and the
//! upstream file calls them from marked one-line hooks.

use super::{Arc, BTreeMap, BTreeSet, EventText, FactsCoder, FactsNotes, FactsScope, MAX_PLAYERS, MissionFrame, Mutex, OnlineChallenge, OnlinePlayers, PlayersBackEnd, Presented, Round, SelectedLiveError, SelectedLiveSession, error, trace};
use serde_json::json;

/// Co-op fork (M11): how long the challenge AI skaters stand at their leaving owner's last
/// frames before the next owner's facts must carry them (its lead over the lockstep is about
/// 120 ticks, and its spawns may wait on a character's scene).
const AI_HANDOFF_GRACE: u32 = 600;

/// Co-op fork (M22): the "waiting for players" notice's text, the game's own (`SetupLobbyHUD`'s
/// waiting line); the original's `eSk8FEPopup_Online_WaitForPlayersSmall` is not read here.
const WAITING_FOR_SKATERS: &str = "ID_ONLINE_WAITING_FOR_SKATERS";

/// Co-op fork (M23): the tick a difference found at checkpoint `tick` dumps its parts' fields
/// at: the next multiple of [`SYNC_DUMP_GRID`] at least a grid past it, which every console
/// reaches after hearing of the difference unless its round trails by more than that.
const fn sync_dump_tick(tick: u32) -> u32 {
    (tick / SYNC_DUMP_GRID + 2) * SYNC_DUMP_GRID
}

/// Co-op fork (M23): the grid of ticks field dumps fall on.
const SYNC_DUMP_GRID: u32 = 300;

/// Co-op fork: what a round holds for the campaign and its challenge AI skaters. Every round has
/// one; a round that is not a campaign's keeps it at its default (no campaign, nothing owned),
/// which the hooks read as nothing to do.
#[derive(Default)]
pub(super) struct CoopRound {
    /// Co-op fork: the round is a campaign's, the whole session long: every player paces it,
    /// its objective groups each read their own player, and it never completes.
    pub(super) campaign: bool,
    /// Co-op fork (M22): a campaign round's pace and its "waiting for players" notice
    /// (`crate::coop::pace`).
    pub(super) pace: crate::coop::pace::Pace,
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
    /// Co-op fork (M23): the campaign round's sync check (`crate::coop::sync_check`) and the
    /// grid ticks whose field dumps a difference asked for (two, so seats that heard of it a
    /// grid apart still dump at one tick alike) and the parts.
    sync_check: crate::coop::sync_check::SyncCheck,
    sync_dump_at: Option<([u32; 2], u8)>,
    /// Co-op fork (M10): the coders of the challenge AI skaters this console owns, by slot, and
    /// the AI slots whose frames the last tick carried (for `cMsgSkaterCreated`/`Destroyed`).
    owned: OwnedCoders,
    ai_present: BTreeSet<i32>,
    /// This console's ambient skaters last counted, for the trace (they are no frame of the scripts').
    ambient_seen: usize,
    /// Co-op fork (M10): the owned skaters' frames that came with each player's last facts: a
    /// player standing at its last facts has its AI stand too.
    pub(super) standing_owned: [Vec<MissionFrame>; MAX_PLAYERS],
    /// Co-op fork (M11): a player who owned challenge AI skaters left at this tick (and its
    /// index): its skaters stand at its last frames until another player's facts carry them,
    /// `AI_HANDOFF_GRACE` ticks at most (open item 65).
    ai_handoff: Option<(u32, usize)>,
    /// Co-op fork: movie ends, counted as the NIS events are.
    pub(super) fmv_seen: u32,
    pub(super) fmv_delivered: u32,
}

/// Co-op fork (M10): the coders of the challenge AI skaters one player owns, by their slot, each
/// coded as that player codes it; the slots its last facts carried, for the trace.
#[derive(Default)]
pub(super) struct OwnedCoders {
    coders: BTreeMap<u8, FactsCoder>,
    slots: Vec<u8>,
}

impl OwnedCoders {
    /// Co-op fork (M10): another player's owned frames of a tick, each decoded at its slot.
    pub(super) fn decode(&mut self, owned: &[(u8, Vec<u8>)], scope: FactsScope, user_id: u64, tick: u32, stint: u32) -> Vec<MissionFrame> {
        let mut owned_frames = Vec::with_capacity(owned.len());
        for (slot, data) in owned {
            match self.coders.entry(*slot).or_insert_with(|| FactsCoder::new(scope)).decode(data, i32::from(*slot)) {
                Ok(frame) => owned_frames.push(frame),
                Err(e) => trace(format_args!("{user_id:016x}: facts of tick {tick}, owned slot {slot}: {e}")),
            }
        }
        let slots: Vec<u8> = owned.iter().map(|(slot, _)| *slot).collect();
        if slots != self.slots {
            trace(format_args!("{user_id:016x}: facts of tick {tick} (stint {stint}) carry the owned AI slots {slots:?} (before: {:?})", self.slots));
            self.slots = slots;
        }
        owned_frames
    }
}

impl CoopRound {
    /// Co-op fork: the prompts and (M9) outcomes the local skater's next facts carry.
    pub(super) fn pending_notes(&self) -> (Vec<skate2_net::protocol::Prompt>, Vec<skate2_net::protocol::Outcome>) {
        let prompts = self.prompts_pending.iter().take(skate2_net::protocol::MAX_PROMPTS).copied().collect();
        let outcomes = self.outcomes_pending.iter().take(skate2_net::protocol::MAX_OUTCOMES).copied().collect();
        (prompts, outcomes)
    }

    /// Co-op fork (M23): a campaign round's next facts carry its newest round check not yet sent.
    pub(super) fn attach_sync_check(&mut self, notes: &mut FactsNotes) {
        if self.campaign {
            notes.check = self.sync_check.take_unsent().map(|(tick, parts)| skate2_net::protocol::RoundCheck { tick, parts });
        }
    }

    /// Co-op fork (M23): a sync point starts the round's ticks again: the checks start over and
    /// no field dump stays asked for.
    pub(super) fn sync_point(&mut self) {
        self.sync_check.sync_point();
        self.sync_dump_at = None;
    }

    /// Co-op fork (M10): a player's facts at a tick into the round's: its frame, then the frames
    /// of the AI skaters it owns (which stand there until its next facts), and its notes under
    /// its player number.
    pub(super) fn push_facts(&mut self, frames: &mut Vec<MissionFrame>, notes: &mut Vec<(i32, FactsNotes)>, index: usize, frame: MissionFrame, owned: Vec<MissionFrame>, note: FactsNotes) {
        frames.push(frame);
        self.standing_owned[index].clone_from(&owned);
        frames.extend(owned);
        notes.push((i32::try_from(index).unwrap_or(i32::MAX), note));
    }

    /// Co-op fork (M11): player `player` (index `index`) left at `tick`: the AI skaters it owned
    /// stand at its last frames for the hand-off.
    pub(super) fn ai_owner_left(&mut self, player: i32, tick: u32, index: usize) {
        if !self.standing_owned[index].is_empty() && self.ai_handoff.is_none() {
            trace(format_args!("player {player} left at tick {tick} with {} owned AI skaters: they stand at its last frames until the next owner's facts carry them, {AI_HANDOFF_GRACE} ticks at most", self.standing_owned[index].len()));
            self.ai_handoff = Some((tick, index));
        }
    }

    /// Co-op fork (M11): while a hand-off is under way, the leaver's AI skaters stand in the
    /// tick's frames until another player's facts carry AI slots (past the `seats` humans), or
    /// the grace runs out.
    pub(super) fn carry_ai_handoff(&mut self, frames: &mut Vec<MissionFrame>, tick: u32, seats: usize) {
        if let Some((since, from)) = self.ai_handoff {
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
    }
}

impl Round {
    /// Co-op fork: a campaign round traces a presentation event this console saw.
    pub(super) fn trace_presentation_seen(&self, presented: Presented) {
        if self.coop.campaign {
            crate::coop::trace(format_args!(
                "presentation {presented:?} seen here at tick {} ({})",
                self.processed,
                if self.presentation_leader() { "this console leads: its facts will announce it" } else { "waiting for the leader's facts to deliver it" }
            ));
        }
    }

    /// Co-op fork: a campaign round traces the events its next facts announce.
    pub(super) fn trace_presentation_announced(&self, announced: &[Presented]) {
        if self.coop.campaign && !announced.is_empty() {
            crate::coop::trace(format_args!("presentation {announced:?} announced with the facts of tick {}", self.produced));
        }
    }

    /// Co-op fork: a campaign round traces an event announced again after its delivery.
    pub(super) fn trace_presentation_again(&self, presented: Presented, new: bool) {
        if self.coop.campaign && !new {
            crate::coop::trace(format_args!("presentation {presented:?} announced again at tick {}: already delivered", self.processed));
        }
    }

    /// Co-op fork: the tick's prompts and (M9) outcomes, each player's applied once each in
    /// order, the presser's or decider's own pending ones done with once its own facts carried
    /// them; then (M23) a campaign round compares the tick's round checks.
    /// Returns the notes without their players, for upstream's own reading of them.
    #[expect(clippy::type_complexity, reason = "one tick's notes, and its prompts and outcomes by player")]
    pub(super) fn take_coop_notes(&mut self, notes: Vec<(i32, FactsNotes)>) -> (Vec<FactsNotes>, Vec<(i32, skate2_net::protocol::Prompt)>, Vec<(i32, skate2_net::protocol::Outcome)>) {
        // Co-op fork: each player's prompts, applied once each in order; the presser's own
        // pending ones are done with once its own facts carried them.
        let prompts = take_sequenced(&notes, |note| &note.prompts, &mut self.coop.prompts_applied, &mut self.coop.prompts_pending, self.local);
        // Co-op fork (M9): each player's outcomes, applied once each in order, the decider's own
        // pending ones done with once its own facts carried them.
        let outcomes = take_sequenced(&notes, |note| &note.outcomes, &mut self.coop.outcomes_applied, &mut self.coop.outcomes_pending, self.local);
        if self.coop.campaign {
            self.compare_checks(&notes);
        }
        (notes.into_iter().map(|(_, note)| note).collect(), prompts, outcomes)
    }

    /// Co-op fork (M10): a player standing at a tick stands with the AI skaters it owns, at their
    /// last frames.
    pub(super) fn push_standing(&self, frames: &mut Vec<MissionFrame>, index: usize) {
        frames.push(self.standing_frame(index));
        frames.extend(self.coop.standing_owned[index].iter().cloned());
    }

    /// Co-op fork (M10): the owned challenge AI skaters' frames for the facts of `tick`: coded at
    /// their slots for the wire, their shared frames for the round, and the coded bytes.
    #[expect(clippy::type_complexity, reason = "the wire's, the round's and the trace's share of the owned frames")]
    pub(super) fn encode_owned(&mut self, tick: u32, owned_frames: Vec<MissionFrame>) -> (Vec<(u8, Vec<u8>)>, Vec<MissionFrame>, usize) {
        let scope = self.scope;
        let mut owned = Vec::with_capacity(owned_frames.len());
        let mut owned_shared = Vec::with_capacity(owned_frames.len());
        if owned_frames.len() > skate2_net::protocol::owned::MOST {
            crate::coop::trace(format_args!("{} owned AI skaters at tick {tick}: the wire carries {}; the rest stay here", owned_frames.len(), skate2_net::protocol::owned::MOST));
        }
        for frame in owned_frames.into_iter().take(skate2_net::protocol::owned::MOST) {
            let Ok(slot) = u8::try_from(frame.player) else { continue };
            let frame = scope.shared(&frame);
            let coded = self.coop.owned.coders.entry(slot).or_insert_with(|| FactsCoder::new(scope)).encode(&frame);
            owned.push((slot, coded));
            owned_shared.push(frame);
        }
        let owned_bytes: usize = owned.iter().map(|(_, data)| data.len()).sum();
        let slots: Vec<u8> = owned.iter().map(|(slot, _)| *slot).collect();
        if slots != self.coop.owned.slots {
            trace(format_args!("facts of tick {tick} produced (processed {}) carry the owned AI slots {slots:?} (before: {:?})", self.processed, self.coop.owned.slots));
            self.coop.owned.slots = slots;
        }
        (owned, owned_shared, owned_bytes)
    }

    /// Co-op fork (M22): the local facts produced and not yet run, while a running campaign round
    /// is paced by the local skater and at least one other player (`UpdateInputRate`'s "sent and
    /// not simulated"); none otherwise.
    pub(super) fn campaign_pace_lead(&self) -> Option<u32> {
        let others = self.players.len().saturating_sub(self.disconnected.len()) > 1;
        (self.coop.campaign && self.running() && others && self.local_contributes()).then(|| self.produced.saturating_sub(self.processed))
    }

    /// Co-op fork (M22): why the pace has no lead to read, for the trace.
    pub(super) fn unpaced_reason(&self) -> &'static str {
        if !self.running() {
            "the round not running (a sync point)"
        } else if !self.local_contributes() {
            "the local skater not pacing the round"
        } else {
            "no other player in the round"
        }
    }

    /// The player `Online.SwitchCamera` last made this console follow.
    pub(in super::super) fn followed(&self) -> Option<i32> {
        self.camera_player
    }

    /// Co-op fork: a prompt this console answered, to travel with its next facts.
    pub(in super::super) fn queue_prompt(&mut self, kind: u8, value: i32) {
        self.coop.prompt_seq += 1;
        self.coop.prompts_pending.push(skate2_net::protocol::Prompt { seq: self.coop.prompt_seq, kind, value });
    }

    /// Co-op fork (M9): an outcome this console's scripts decided, to travel with its next facts.
    pub(in super::super) fn queue_outcome(&mut self, kind: u8, activity: u64, value: i32) {
        self.coop.outcome_seq += 1;
        self.coop.outcomes_pending.push(skate2_net::protocol::Outcome { seq: self.coop.outcome_seq, kind, activity, value });
    }

    /// Co-op fork (M23): the other players' round checks that came with this tick's facts,
    /// against this console's own record of the ticks they name. A first difference with a
    /// player prints this console's whole parts of the ticks around it and asks for the parts'
    /// field dump at a tick every console reaches alike ([`sync_dump_tick`]).
    pub(super) fn compare_checks(&mut self, notes: &[(i32, FactsNotes)]) {
        use crate::coop::sync_check::{Compared, part_list, part_names};
        for (player, note) in notes {
            let Some(check) = note.check.filter(|_| *player != self.local) else { continue };
            let Some(&user_id) = usize::try_from(*player).ok().and_then(|index| self.players.get(index)) else { continue };
            match self.coop.sync_check.compare(user_id, check.tick, check.parts) {
                Compared::Agrees | Compared::StillDiffers { .. } => {}
                Compared::Unknown => {
                    crate::coop::trace(format_args!("sync check of player {player} at tick {}: no record of the tick here", check.tick));
                    crate::coop::event("sync_unknown", json!({"player": player, "tick": check.tick}));
                }
                Compared::Differs { mask, first } => {
                    let ours = self.coop.sync_check.parts_at(check.tick).unwrap_or_default();
                    crate::coop::trace(format_args!(
                        "sync check: player {player} differs at tick {} in {} ({}; ours {:08x?}, theirs {:04x?})",
                        check.tick,
                        part_names(mask),
                        if first { "the first difference" } else { "after agreeing again" },
                        ours,
                        check.parts
                    ));
                    crate::coop::event("sync_differs", json!({"player": player, "tick": check.tick, "parts": part_list(mask), "first": first}));
                    for (tick, parts) in self.coop.sync_check.window(check.tick) {
                        crate::coop::trace(format_args!("sync parts @{tick}: {parts:08x?}"));
                        crate::coop::event("sync_parts", json!({"tick": tick, "parts": parts}));
                    }
                    if self.coop.sync_dump_at.is_none() {
                        let at = sync_dump_tick(check.tick);
                        self.coop.sync_dump_at = Some(([at, at + SYNC_DUMP_GRID], mask));
                    }
                }
            }
        }
    }

    /// Co-op fork (M23): each player's sync checks this round, for the trace.
    pub(super) fn trace_sync_standings(&self) {
        if !self.coop.campaign {
            return;
        }
        for (user_id, checked, differed, first_at) in self.coop.sync_check.standings() {
            let first = first_at.map_or_else(|| "none".to_owned(), |(tick, mask)| format!("tick {tick} in {}", crate::coop::sync_check::part_names(mask)));
            crate::coop::trace(format_args!("sync check with {user_id:016x}: {checked} checks, {differed} differed, first difference {first}"));
            crate::coop::event("sync_standing", json!({
                "user": format!("{user_id:016x}"), "checked": checked, "differed": differed,
                "first_tick": first_at.map(|(tick, _)| tick), "first_parts": first_at.map(|(_, mask)| crate::coop::sync_check::part_list(mask)),
            }));
        }
    }
}

impl OnlinePlayers {
    /// Co-op fork (open item 45): a player past the humans is one of this console's own
    /// challenge AI skaters, whose row the AI manager creates in the profile's back end at the
    /// skater's slot (`AISkaterManager::post`); the row answers for it.
    #[expect(clippy::cast_sign_loss, reason = "the player type's bits, read as the AI manager's unsigned kind")]
    pub(super) fn ai_row(&self, player: i32) -> Option<i32> {
        (player >= self.count() && crate::ai::skater_manager::player_type::is_ai(self.settings.player_type(player) as u32)).then_some(player)
    }

    /// The profile's row a player's settings live in: the local skater's or an AI skater's.
    pub(super) fn settings_row(&self, player: i32) -> Option<i32> {
        self.local_row(player).or_else(|| self.ai_row(player))
    }
}

impl<T, C> SelectedLiveSession<T, C> {
    /// Co-op fork, `GameReset(CAMPAIGN)`: the campaign's round starts, when this console is in
    /// the table and its game is a campaign lobby; otherwise the reset is traced and ignored.
    pub(super) fn campaign_reset(&mut self, seed: u32, session: u32, players: Vec<u64>, local: Option<usize>) -> Result<(), SelectedLiveError> {
        match local {
            Some(local) if self.online_campaign() => self.start_campaign_round(seed, session, players, local)?,
            _ => trace(format_args!("campaign reset ignored: local {local:?}, campaign {}", self.online_campaign())),
        }
        Ok(())
    }

    /// Co-op fork: the prompts answered here since the last facts ride the next, and (M9) the
    /// outcomes this console's scripts decided for its own player or for no player.
    pub(super) fn queue_coop_notes(&mut self, local: i32) {
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
    }

    /// Co-op fork: the prompts and (M9) outcomes a round tick delivered, applied to the career
    /// at that tick on every console.
    pub(super) fn apply_coop_notes(&mut self, prompts: Vec<(i32, skate2_net::protocol::Prompt)>, outcomes: Vec<(i32, skate2_net::protocol::Outcome)>, tick: u32) {
        if let Some(career) = self.career.as_mut() {
            for (player, prompt) in prompts {
                match crate::coop::prompts::PromptAnswer::from_wire(prompt.kind, prompt.value) {
                    Some(answer) => {
                        crate::coop::trace(format_args!("prompt {answer:?} from player {player} applied at tick {tick}"));
                        crate::coop::event("prompt_applied", json!({}));
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
    }

    /// Co-op fork (open item 45, M10): the owned challenge AI skaters' frames came
    /// with their owner's facts, so the scripts of every console read the humans
    /// and one AI run and nothing else. An AI slot's first and last frame are its
    /// creation and destruction to the scripts, the same tick on every console.
    /// This console's own ambient skaters (each console's own, open item 53) stay
    /// out of the scripts' frames (M10.3, open item 69): their creation reached
    /// the scripts as AI players at each console's own ticks and slots.
    pub(super) fn campaign_round_frames(&mut self, frames: &[MissionFrame], tick: u32) {
        if let Some(humans) = self.career.as_ref().and_then(|career| career.system.world().coop.as_ref()).map(|campaign| campaign.seats) {
            let ambient = self.career_ai_skaters(humans, false).count();
            if let Some(round) = self.online.round.as_mut().filter(|round| round.coop.ambient_seen != ambient) {
                round.coop.ambient_seen = ambient;
                crate::coop::trace(format_args!("{ambient} ambient skaters here at tick {tick}: each console's own, out of the scripts' frames"));
            }
            self.online_round_ai_messages(frames, humans, tick);
        }
    }

    /// Co-op fork (M23): the hash of a campaign round tick's frames, for its sync check; none
    /// (0) outside a campaign.
    pub(super) fn campaign_round_inputs(&self, frames: &[MissionFrame]) -> u32 {
        if self.online.round.as_ref().is_some_and(|round| round.coop.campaign) { crate::coop::round_hash::inputs(frames) } else { 0 }
    }

    /// Co-op fork (M10): the frames of the challenge AI skaters this console owns in a
    /// campaign, which ride its next facts; none otherwise.
    pub(super) fn owned_ai_frames(&self) -> Result<Vec<MissionFrame>, SelectedLiveError> {
        Ok(match self.career.as_ref().and_then(|career| career.system.world().coop.as_ref().map(|campaign| (campaign.seats, crate::challenge::players::owns_challenge_ai(career.system.world())))) {
            Some((humans, true)) => self.career_ai_skater_frames(humans, true)?,
            _ => Vec::new(),
        })
    }

    /// Co-op fork (M10): the user id of the puppet a campaign's AI slot (after the players)
    /// shows here, followed as another player's skater is.
    pub(super) fn campaign_ai_puppet(&self, player: i32) -> Option<u64> {
        let owner = self.career.as_ref().and_then(|career| career.system.world().coop.as_ref()).map_or(0, crate::coop::rules::CampaignWorld::ai_owner);
        Some(skate2_net::protocol::owned::skater_id(u8::try_from(owner).ok()?, u8::try_from(player).ok()?))
    }

    /// Co-op fork: the campaign round's next tick, while one runs.
    pub(in super::super::super) fn campaign_round_tick(&self) -> Option<u32> {
        self.online.round.as_ref().filter(|round| round.coop.campaign).map(|round| round.processed)
    }

    /// Co-op fork, `GameReset(CAMPAIGN)`: the campaign's round starts on every console at the same
    /// tick 0 with the server's seed and table. The career stays the career (`GM_SINGLE_PLAYER`,
    /// progression on, autosaves); what changes is who paces its scripts: from here every
    /// console runs them on every player's facts (scope "everything"), the players back end is the
    /// table's, the career's random stream is the seeded one, the objective groups are per player
    /// under the completion rules, and scenes run under the online rule.
    pub(super) fn start_campaign_round(&mut self, seed: u32, session: u32, players: Vec<u64>, local: usize) -> Result<(), SelectedLiveError> {
        let stance = Arc::new(Mutex::new([true; MAX_PLAYERS]));
        let backend = self.online_players_backend(&players, local, &stance)?;
        let count = players.len();
        let local_index = i32::try_from(local).unwrap_or(0);
        let mut round = Round::loading(session, skate2_net::protocol::campaign::KEY, None, players.clone(), local_index, FactsScope::ALL, false, stance);
        round.coop.campaign = true;
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

    /// Co-op fork (M23): the campaign round's parts after the tick the scripts just ran, whose
    /// frames hashed to `inputs`, for the sync check; the parts' field dumps the bench asked for
    /// (`SKATE2_COOP_SYNC_DUMP=<ticks>`, every that many ticks) or a difference did; and
    /// `SKATE2_DIAGNOSTIC_DESYNC=<tick>`, a planted divergence of this console's vars at the tick.
    pub(super) fn record_round_check(&mut self, inputs: u32) {
        use crate::coop::sync_check::{PARTS, Part};
        static DUMP_EVERY: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
        static PLANT_AT: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
        let Some(round) = self.online.round.as_mut().filter(|round| round.coop.campaign) else { return };
        let Some(career) = self.career.as_mut() else { return };
        let tick = round.processed.saturating_sub(1);
        if PLANT_AT.get_or_init(|| std::env::var("SKATE2_DIAGNOSTIC_DESYNC").ok().and_then(|value| value.trim().parse().ok())).is_some_and(|at| at == tick) {
            match crate::coop::round_hash::plant_divergence(&mut career.system) {
                Some(name) => {
                    crate::coop::trace(format_args!("diagnostic: a divergence planted in {name}'s vars at tick {tick}"));
                    crate::coop::event("desync_planted", json!({"activity": name, "tick": tick}));
                }
                None => crate::coop::trace(format_args!("diagnostic: no activity's vars to plant a divergence in at tick {tick}")),
            }
        }
        let mut parts = crate::coop::round_hash::round_parts(&career.system);
        parts[0] = inputs;
        round.coop.sync_check.record(tick, parts);
        let every = DUMP_EVERY.get_or_init(|| std::env::var("SKATE2_COOP_SYNC_DUMP").ok().and_then(|value| value.trim().parse().ok()).filter(|&every: &u32| every > 0));
        let asked = round.coop.sync_dump_at.filter(|&(at, _)| tick >= at[0]).map(|(at, mask)| (at[0], mask));
        let mask = match (asked, every.is_some_and(|every| tick % every == 0)) {
            (Some((at, mask)), periodic) => {
                round.coop.sync_dump_at = round.coop.sync_dump_at.and_then(|(at, mask)| (at[1] != u32::MAX && at[1] > tick).then_some(([at[1], u32::MAX], mask)));
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
                crate::coop::event("sync_dump", json!({"tick": tick, "part": part.name(), "line": line}));
            }
        }
    }

    /// Co-op fork (M22): one simulation tick of a campaign round's pace and stall notice, from
    /// the lead before the round ran and whether a round tick ran.
    pub(super) fn update_campaign_pace(&mut self, lead: Option<u32>, ran: bool) {
        let (rtt, frame) = (self.online.round_trip, self.completed_frames);
        let Some(round) = self.online.round.as_mut().filter(|round| round.coop.campaign) else { return };
        let resumed = round.coop.paced_frame.is_some_and(|last| frame.saturating_sub(last) > 1);
        round.coop.paced_frame = Some(frame);
        let (tick, before, paced_before) = (round.processed, round.coop.pace.hz(), round.coop.pace.paced());
        if let Some(paced) = round.coop.pace.update(lead, rtt) {
            crate::coop::event("pace", json!({"tick": tick, "from_hz": before, "to_hz": paced.hz}));
            match paced.lead {
                Some(lead) => crate::coop::trace(format_args!(
                    "pace at tick {tick}: timer {before} -> {} Hz, {lead} local facts not yet run (free/target {:.2}, target {:.1}, ping {:.0} ms)",
                    paced.hz, paced.ratio, paced.target, paced.ping_ms
                )),
                None => crate::coop::trace(format_args!("pace at tick {tick}: timer {before} -> {} Hz, {}", paced.hz, round.unpaced_reason())),
            }
        } else if paced_before && !round.coop.pace.paced() {
            crate::coop::trace(format_args!("pace at tick {tick}: the game's own timer again, {}", round.unpaced_reason()));
        }
        let notice = if round.running() && round.processed > 0 && !resumed { round.coop.stall.update(ran) } else { round.coop.stall.pause() };
        let awaited = round.awaited();
        let shown = round.coop.stall.shown();
        let text = EventText::plain(WAITING_FOR_SKATERS);
        match notice {
            Some(crate::coop::pace::Notice::Open) => {
                crate::coop::trace(format_args!(
                    "waiting for players at tick {tick}: no round tick for {} simulation ticks, awaiting {}, notice shown",
                    crate::coop::pace::STALL_TICKS,
                    awaited.map_or_else(|| "no other player's facts".to_owned(), |user_id| format!("user {user_id}'s facts"))
                ));
                crate::coop::event("notice_shown", json!({"tick": tick}));
            }
            Some(crate::coop::pace::Notice::Close(waited)) => {
                crate::coop::trace(format_args!("waiting for players over at tick {tick} after {waited} simulation ticks{}, notice taken down", if resumed { " (this console's own pause in the round's updates)" } else { "" }));
                crate::coop::event("notice_down", json!({"tick": tick, "waited": waited}));
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
    pub(super) fn retire_campaign_notice(&mut self) {
        if let Some(round) = self.online.round.as_mut().filter(|round| round.coop.campaign)
            && let Some(crate::coop::pace::Notice::Close(waited)) = round.coop.stall.pause()
        {
            crate::coop::trace(format_args!("waiting for players over with the round after {waited} simulation ticks, notice taken down"));
            self.online.event_log.retire(&EventText::plain(WAITING_FOR_SKATERS));
        }
    }

    /// Co-op fork (M22): the timer rate a campaign round's pace sets for the 60 Hz baseline,
    /// while another player paces the round with the local skater and the round was stepped in
    /// the last frame.
    pub(in super::super::super) fn campaign_pace_hz(&self) -> Option<i32> {
        let frame = self.completed_frames;
        self.online
            .round
            .as_ref()
            .filter(|round| round.coop.campaign && round.coop.pace.paced() && round.coop.paced_frame.is_some_and(|last| frame.saturating_sub(last) <= 1))
            .map(|round| round.coop.pace.hz())
    }

    /// Co-op fork (M10): the AI slots the tick's frames hold against the last tick's: a slot
    /// that appears is `cMsgSkaterCreated` to the scripts (with the AI's ordinal, as they number
    /// it), one that goes `cMsgSkaterDestroyed`; a streamed skater's puppet hides with its last
    /// frame.
    pub(super) fn online_round_ai_messages(&mut self, frames: &[MissionFrame], humans: usize, tick: u32) {
        use crate::hermes_challenge_queue::{ChallengeMessage, ChallengePayload, SkaterNotification};
        let present: BTreeSet<i32> = frames.iter().map(|frame| frame.player).filter(|&slot| usize::try_from(slot).is_ok_and(|slot| slot >= humans)).collect();
        let Some(round) = self.online.round.as_mut() else { return };
        if present == round.coop.ai_present {
            return;
        }
        let was = std::mem::replace(&mut round.coop.ai_present, present.clone());
        let Some(career) = self.career.as_mut() else { return };
        let mut hidden = Vec::new();
        let mut changed = Vec::new();
        let owner = crate::challenge::players::owns_challenge_ai(career.system.world());
        for (slot, notification) in was.difference(&present).map(|&slot| (slot, SkaterNotification::Destroyed)).chain(present.difference(&was).map(|&slot| (slot, SkaterNotification::Created))) {
            let world = career.system.world();
            let ordinal = crate::challenge::players::script_index(world, slot);
            let streamed = crate::challenge::players::streamed_ai_slot(world, slot);
            crate::coop::trace(format_args!("AI skater at slot {slot} (the scripts' AI player {ordinal}{}): {notification:?} at tick {tick}", if streamed { ", a puppet here" } else { "" }));
            career.system.post(super::super::super::career::challenge_queue_event(ChallengeMessage::new(ChallengePayload::Skater { notification, player_id: ordinal })));
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
    pub(in super::super) fn end_campaign_ai_streams(&mut self) {
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
    pub(super) fn online_ai_owner_changed(&mut self, new_owner: i32) {
        let Some(round) = self.online.round.as_ref() else { return };
        let (local, tick) = (round.local, round.processed);
        let standing: Vec<MissionFrame> = round.coop.ai_handoff.map(|(_, from)| round.coop.standing_owned[from].clone()).unwrap_or_default();
        let mine = new_owner == local;
        crate::coop::trace(format_args!("campaign AI skaters: their owner left; owned by player {new_owner}{} from tick {tick}", if mine { " (this console)" } else { "; puppets here" }));
        if !mine {
            return;
        }
        let Some(ai) = self.inner.ai.as_mut() else { return };
        ai.streamed_challenge_ai = false;
        let spawns = std::mem::take(&mut ai.streamed_spawns);
        let mut taken = Vec::with_capacity(spawns.len());
        for (_, command) in spawns {
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
}

/// Co-op fork: a note numbered in its player's run (prompts, M9's outcomes).
trait Sequenced: Copy {
    fn seq(&self) -> u32;
}
impl Sequenced for skate2_net::protocol::Prompt {
    fn seq(&self) -> u32 {
        self.seq
    }
}
impl Sequenced for skate2_net::protocol::Outcome {
    fn seq(&self) -> u32 {
        self.seq
    }
}

/// Co-op fork: the tick's notes of one kind (`of` picks them from a player's facts notes), each
/// player's taken once each in order: one numbered past what `applied` holds for its player is
/// taken and raises it. Once the local player's own facts carried its notes, its `pending` ones
/// up to that number are done with.
fn take_sequenced<N: Sequenced>(notes: &[(i32, FactsNotes)], of: impl Fn(&FactsNotes) -> &[N], applied: &mut [u32; MAX_PLAYERS], pending: &mut Vec<N>, local: i32) -> Vec<(i32, N)> {
    let mut taken = Vec::new();
    for (player, note) in notes {
        let Some(slot) = usize::try_from(*player).ok().filter(|&slot| slot < MAX_PLAYERS) else { continue };
        for &item in of(note) {
            if item.seq() > applied[slot] {
                applied[slot] = item.seq();
                taken.push((*player, item));
            }
        }
        if *player == local {
            let done = applied[slot];
            pending.retain(|item| item.seq() > done);
        }
    }
    taken
}
