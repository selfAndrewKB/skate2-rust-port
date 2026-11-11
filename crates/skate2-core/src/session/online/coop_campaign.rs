//! Co-op fork: the campaign's shared progression on the session (M2, `docs/coop/plan/topics/
//! progression.md`). The round itself is the port's facts lockstep (`online_challenge.rs`,
//! `start_campaign_round`); this file holds what is around it: the hold before the round starts,
//! the progression save sent to the server, and the bench's deterministic inputs at a round tick.

use super::{SelectedLiveError, SelectedLiveSession};
use crate::coop::trace;
use skate2_net::protocol::ClientMessage;

pub(super) type Leaf = (&'static str, u32, Vec<Vec<u8>>);

/// `SKATE2_DIAGNOSTIC_COOP_ANSWER=yes|no[:<ticks>]`: the progression's Yes/No dialog answered that
/// many round ticks (default 60) after it opens, the same tick on every console, so the bench
/// gets past "New to skate?" without a pad (M3 makes a person's answer travel as a note).
fn diagnostic_answer() -> Option<(bool, u32)> {
    static SPEC: std::sync::OnceLock<Option<(bool, u32)>> = std::sync::OnceLock::new();
    *SPEC.get_or_init(|| {
        let spec = std::env::var("SKATE2_DIAGNOSTIC_COOP_ANSWER").ok()?;
        let (answer, delay) = spec.split_once(':').unwrap_or((spec.as_str(), "60"));
        let yes = match answer.trim() {
            "yes" => true,
            "no" => false,
            _ => return None,
        };
        Some((yes, delay.trim().parse().unwrap_or(60)))
    })
}

/// `SKATE2_DIAGNOSTIC_COOP_TRIGGERS=<tick>:<radius>`: at that round tick, the trigger volumes
/// around the local skater and every objective group's trigger volume ids, to find where a go-to
/// mission wants the skaters.
fn diagnostic_triggers() -> Option<(u32, f32)> {
    static SPEC: std::sync::OnceLock<Option<(u32, f32)>> = std::sync::OnceLock::new();
    *SPEC.get_or_init(|| {
        let spec = std::env::var("SKATE2_DIAGNOSTIC_COOP_TRIGGERS").ok()?;
        let (tick, radius) = spec.split_once(':')?;
        Some((tick.trim().parse().ok()?, radius.trim().parse().ok()?))
    })
}

/// `SKATE2_DIAGNOSTIC_COOP_COMPLETE=<tick>[;<tick>...]`: at each round tick the first active
/// objective of every group of the main challenge is completed as its script would complete it,
/// the same tick on every console: the bench's way past the steps a teleport cannot prove (an
/// ollie, a duck, a prop), after the ones it can.
fn diagnostic_completes() -> &'static [(u32, Option<i32>)] {
    static SPEC: std::sync::OnceLock<Vec<(u32, Option<i32>)>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| {
        std::env::var("SKATE2_DIAGNOSTIC_COOP_COMPLETE").ok().map_or_else(Vec::new, |spec| {
            spec.split(';')
                .filter_map(|entry| {
                    let (tick, player) = entry.split_once('@').map_or((entry, None), |(tick, player)| (tick, Some(player)));
                    Some((tick.trim().parse().ok()?, player.map(|player| player.trim().parse().ok()).flatten()))
                })
                .collect()
        })
    })
}

/// `SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=<tick>:<x>,<y>,<z>[;...]`: as `SKATE2_DIAGNOSTIC_COOP_TELEPORT`,
/// but at round ticks, so a run's stops line up with the mission's ticks whenever the round
/// started.
fn diagnostic_round_teleports() -> &'static [(u32, [f32; 3])] {
    static SPEC: std::sync::OnceLock<Vec<(u32, [f32; 3])>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| {
        let Ok(spec) = std::env::var("SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND") else { return Vec::new() };
        spec.split(';')
            .filter_map(|entry| {
                let (at, position) = entry.split_once(':')?;
                let values: Vec<f32> = position.split(',').filter_map(|v| v.trim().parse().ok()).collect();
                let [x, y, z] = values[..] else { return None };
                Some((at.trim().parse().ok()?, [x, y, z]))
            })
            .collect()
    })
}

/// `SKATE2_DIAGNOSTIC_COOP_PROMPT=<tick>:<answer>[;...]`: at that round tick this console answers
/// a prompt as a pad would (`PromptAnswer::parse`: yes, no, option:<n>, skate101, unlock, signup,
/// release, choice:<n>, cancel, accept, decline); the answer travels as a note.
fn diagnostic_prompts() -> &'static [(u32, crate::coop::prompts::PromptAnswer)] {
    static SPEC: std::sync::OnceLock<Vec<(u32, crate::coop::prompts::PromptAnswer)>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| {
        let Ok(spec) = std::env::var("SKATE2_DIAGNOSTIC_COOP_PROMPT") else { return Vec::new() };
        spec.split(';')
            .filter_map(|entry| {
                let (at, answer) = entry.split_once(':')?;
                Some((at.trim().parse().ok()?, crate::coop::prompts::PromptAnswer::parse(answer)?))
            })
            .collect()
    })
}

/// `SKATE2_DIAGNOSTIC_COOP_ZONES=<round tick>`: at that round tick the security guard areas
/// (world painter layer 6) within 600 m of the local skater are scanned at 8 m and the nearest
/// point of each named region traced, so the bench can teleport into one (M5).
fn diagnostic_zones() -> Option<u32> {
    static TICK: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *TICK.get_or_init(|| std::env::var("SKATE2_DIAGNOSTIC_COOP_ZONES").ok().and_then(|tick| tick.trim().parse().ok()))
}

/// `SKATE2_DIAGNOSTIC_COOP_EXECUTE=<tick>:<name>[;...]`: at each round tick the named challenge
/// is activated and executed, the same tick on every console (the bench's way into a mission;
/// a list for the crawl, M8).
fn diagnostic_execute() -> &'static [(u32, String)] {
    static SPEC: std::sync::OnceLock<Vec<(u32, String)>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| tick_names("SKATE2_DIAGNOSTIC_COOP_EXECUTE"))
}

/// `SKATE2_DIAGNOSTIC_COOP_PRELOAD=<round tick>:<name>[;...]`: at each round tick the named
/// challenge's assets (its hulls with their locators, its AI characters and paths) start loading,
/// as `StateLoading::StartChallengeState` loads them behind the loading screen before a mission
/// starts; the crawl preloads 300 round ticks before its execute, so a script that spawns its AI
/// as it starts finds the hull's locators registered and the asset load's completion still ahead
/// of it (open items 45, 47).
fn diagnostic_preload() -> &'static [(u32, String)] {
    static SPEC: std::sync::OnceLock<Vec<(u32, String)>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| tick_names("SKATE2_DIAGNOSTIC_COOP_PRELOAD"))
}

/// `SKATE2_DIAGNOSTIC_COOP_COMPLETE_CHALLENGE=<round tick>:<name>[;...]`: at each round tick the
/// named challenge is completed as the progression's own `CompleteChallenge(name)` does it (a
/// go-to's arrival, a location challenge's trigger): the crawl's way through a mission that has
/// no main challenge to execute and win (`challenge_global_data.Global` 0 or 2) (M8).
fn diagnostic_complete_challenges() -> &'static [(u32, String)] {
    static SPEC: std::sync::OnceLock<Vec<(u32, String)>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| tick_names("SKATE2_DIAGNOSTIC_COOP_COMPLETE_CHALLENGE"))
}

/// `<round tick>:<name>[;<name>|<round tick>:<name>...]` from the variable: a bare name takes
/// the tick before it; an entry whose tick does not parse, or a bare name before any tick, is
/// dropped.
fn tick_names(variable: &str) -> Vec<(u32, String)> {
    std::env::var(variable).ok().map_or_else(Vec::new, |spec| tick_names_of(&spec))
}

fn tick_names_of(spec: &str) -> Vec<(u32, String)> {
    let mut last = None;
    spec.split(';')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (tick, name) = match entry.split_once(':') {
                Some((tick, name)) => (tick.trim().parse().ok()?, name.trim()),
                None => (last?, entry),
            };
            last = Some(tick);
            Some((tick, name.to_owned()))
        })
        .collect()
}

/// `SKATE2_DIAGNOSTIC_COOP_WIN=<round tick>|running+<ticks>[;...]`: at each round tick named,
/// or the given number of round ticks after the main challenge's graph enters
/// `challenge_running` (its own state, past any intro scene: open item 51), every objective of
/// the main challenge's groups that is not complete is completed as a win would complete it
/// (`force_complete`, points objectives included), so the real win, outro and
/// `challenge_cleanup` play on every console: the crawl's way through a mission whose
/// objectives need tricks, gates or a camera (M8).
fn diagnostic_wins() -> &'static [WinAt] {
    static SPEC: std::sync::OnceLock<Vec<WinAt>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| std::env::var("SKATE2_DIAGNOSTIC_COOP_WIN").ok().map_or_else(Vec::new, |spec| win_ats_of(&spec)))
}

/// One entry of `SKATE2_DIAGNOSTIC_COOP_WIN`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WinAt {
    Tick(u32),
    /// So many round ticks after the main challenge entered `challenge_running`.
    AfterRunning(u32),
}

/// The states the win waits for under `running+<ticks>`: each family's "the mission is on" state
/// after its intro and setup (the crawl's state histogram: `challenge_running` for most,
/// `run_heat` the follows, `run_round` the demos, `round_setup` S.K.A.T.E.).
const RUNNING_STATES: [&str; 4] = ["challenge_running", "run_heat", "run_round", "round_setup"];

fn win_ats_of(spec: &str) -> Vec<WinAt> {
    spec.split(';')
        .filter_map(|entry| {
            let entry = entry.trim();
            match entry.strip_prefix("running+") {
                Some(delay) => delay.trim().parse().ok().map(WinAt::AfterRunning),
                None => entry.parse().ok().map(WinAt::Tick),
            }
        })
        .collect()
}

/// The progression savables as one byte string, for the server's store and the hash every
/// console's log can compare: each leaf's name, version and tuples. The skater's own
/// orientation leaf (`cSignPost::Position`) is each console's, not the career's: two seats
/// that completed the same mission at the same tick differed in its low bits alone (M4,
/// `2026-11-10-m4d-*.log`), so it stays out of the blob and of the hash.
fn progression_blob(leaves: &[Leaf]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, version, tuples) in leaves.iter().filter(|leaf| leaf.0 != crate::challenge::signpost_progress::POSITION) {
        out.extend_from_slice(&(name.len() as u32).to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&version.to_le_bytes());
        out.extend_from_slice(&(tuples.len() as u32).to_le_bytes());
        for tuple in tuples {
            out.extend_from_slice(&(tuple.len() as u32).to_le_bytes());
            out.extend_from_slice(tuple);
        }
    }
    out
}

/// The blob back into leaves (M8): the inverse of [`progression_blob`], each name mapped to its
/// savable's constant; a name no savable has, or a cut blob, is `None`.
pub(super) fn progression_leaves(data: &[u8]) -> Option<Vec<Leaf>> {
    use crate::challenge::{dynamic_hull, signpost_progress, unlockables};
    fn u32_at(data: &[u8], at: &mut usize) -> Option<u32> {
        let bytes: [u8; 4] = data.get(*at..*at + 4)?.try_into().ok()?;
        *at += 4;
        Some(u32::from_le_bytes(bytes))
    }
    let mut at = 0_usize;
    let mut leaves = Vec::new();
    while at < data.len() {
        let name_len = u32_at(data, &mut at)? as usize;
        let name = std::str::from_utf8(data.get(at..at + name_len)?).ok()?;
        at += name_len;
        let name: &'static str = match name {
            signpost_progress::DATA => signpost_progress::DATA,
            signpost_progress::MESSAGES => signpost_progress::MESSAGES,
            signpost_progress::POSITION => signpost_progress::POSITION,
            unlockables::DATA => unlockables::DATA,
            dynamic_hull::DATA => dynamic_hull::DATA,
            _ => return None,
        };
        let version = u32_at(data, &mut at)?;
        let count = u32_at(data, &mut at)? as usize;
        let mut tuples = Vec::with_capacity(count.min(4096));
        for _ in 0..count {
            let len = u32_at(data, &mut at)? as usize;
            tuples.push(data.get(at..at + len)?.to_vec());
            at += len;
        }
        leaves.push((name, version, tuples));
    }
    Some(leaves)
}

/// The hash of leaves as the server compares them (M8: the local file's against the server's).
fn blob_hash(leaves: &[Leaf]) -> u64 {
    fnv1a(&progression_blob(leaves))
}

fn fnv1a(bytes: &[u8]) -> u64 {
    skate2_net::protocol::campaign_save_hash(bytes)
}

impl<T, C> SelectedLiveSession<T, C> {
    /// The career is loaded in a campaign and the round has not started.
    pub(in super::super) fn online_campaign_holding(&self) -> bool {
        self.online_campaign() && self.online.campaign_hold && self.online.round.is_none()
    }

    /// M8: the hash of the local file's progression leaves, as the server would compare them.
    pub(in super::super) fn campaign_blob_hash(&self, leaves: &[(&'static str, u32, Vec<Vec<u8>>)]) -> u64 {
        blob_hash(leaves)
    }

    /// The career autosaved in a campaign: its progression to the server, hashed.
    pub(in super::super) fn online_campaign_saved(&mut self, leaves: &[Leaf], unlocks: &[Leaf], hulls: &[Leaf]) {
        let Some(tick) = self.campaign_round_tick() else { return };
        let session = self.online.round.as_ref().map_or(0, |round| round.session);
        let mut all: Vec<Leaf> = Vec::new();
        all.extend_from_slice(leaves);
        all.extend_from_slice(unlocks);
        all.extend_from_slice(hulls);
        let data = progression_blob(&all);
        let hash = fnv1a(&data);
        trace(format_args!("campaign save at session {session} round tick {tick}: {} B, hash {hash:016x}, to the server", data.len()));
        self.online.send(ClientMessage::CampaignSave { session, tick, hash, data });
    }

    /// Before a campaign round's tick runs the scripts: the bench's inputs, at the round tick.
    pub(in super::super) fn coop_campaign_before_step(&mut self) -> Result<(), SelectedLiveError> {
        let Some(tick) = self.campaign_round_tick() else { return Ok(()) };
        if let Some((yes, delay)) = diagnostic_answer() {
            let open = self.career.as_ref().is_some_and(|career| career.progression_dialog_open());
            let opened = &mut self.online.coop_dialog_opened;
            match (open, *opened) {
                (true, None) => {
                    *opened = Some(tick);
                    trace(format_args!("diagnostic: the progression dialog opened at round tick {tick}; answering {} at {}", if yes { "yes" } else { "no" }, tick + delay));
                }
                (true, Some(at)) if tick >= at + delay => {
                    if let Some(career) = self.career.as_mut()
                        && career.answer_progression_dialog(yes)
                    {
                        trace(format_args!("diagnostic: answered {} at round tick {tick}", if yes { "yes" } else { "no" }));
                    }
                    *opened = None;
                }
                (false, Some(_)) => *opened = None,
                _ => {}
            }
        }
        for (at, name) in diagnostic_preload() {
            if *at == tick
                && let Some(career) = self.career.as_mut()
            {
                let loading = career.system.diagnostic_preload_challenge(name);
                trace(format_args!("diagnostic: preload {name} at round tick {tick}: {loading}"));
            }
        }
        for (at, name) in diagnostic_execute() {
            if *at == tick
                && let Some(career) = self.career.as_mut()
            {
                let executed = career.system.diagnostic_execute_challenge(name);
                trace(format_args!("diagnostic: execute {name} at round tick {tick}: {executed}"));
            }
        }
        for (at, name) in diagnostic_complete_challenges() {
            if *at == tick
                && let Some(career) = self.career.as_mut()
            {
                let completed = career.system.diagnostic_complete_challenge(name);
                trace(format_args!("diagnostic: complete challenge {name} at round tick {tick}: {completed}"));
            }
        }
        if !diagnostic_wins().is_empty()
            && let Some(career) = self.career.as_ref()
        {
            // The main challenge's entry into `challenge_running`, once per main challenge.
            match career.system.main_challenge_key_and_state() {
                Some((key, state)) if RUNNING_STATES.contains(&state) && self.online.coop_running_since.is_none_or(|(seen, _)| seen != key) => {
                    self.online.coop_running_since = Some((key, tick));
                    trace(format_args!("diagnostic: the main challenge {key:#018x} is running from round tick {tick}"));
                }
                _ => {}
            }
        }
        let win_now = diagnostic_wins().iter().any(|win| match *win {
            WinAt::Tick(at) => at == tick,
            WinAt::AfterRunning(delay) => self.online.coop_running_since.is_some_and(|(_, since)| since.saturating_add(delay) == tick),
        });
        if win_now && let Some(career) = self.career.as_mut() {
            let won = career.system.diagnostic_win_main_challenge();
            trace(format_args!("diagnostic: won the main challenge at round tick {tick}: {won} objectives completed"));
        }
        for &(at, answer) in diagnostic_prompts() {
            if at == tick
                && let Some(career) = self.career.as_mut()
            {
                trace(format_args!("diagnostic: answering {answer:?} at round tick {tick}"));
                career.prompt(answer);
            }
        }
        for &(at, position) in diagnostic_round_teleports() {
            if at == tick {
                trace(format_args!("diagnostic: teleport to {position:?} at round tick {tick}: {}", self.diagnostic_teleport_local_player(position)));
            }
        }
        for &(at, player) in diagnostic_completes() {
            if at == tick
                && let Some(career) = self.career.as_mut()
            {
                let completed = career.system.diagnostic_complete_active_objectives(player);
                trace(format_args!("diagnostic: completed the active objectives of {} at round tick {tick}: {completed:?}", player.map_or("every player".to_owned(), |player| format!("player {player}"))));
            }
        }
        if let Some(at) = diagnostic_zones()
            && tick == at
        {
            self.diagnostic_security_zones(1500., 12.);
        }
        if let Some((at, radius)) = diagnostic_triggers()
            && tick == at
        {
            self.diagnostic_dump_triggers(radius)?;
            if let Some(career) = self.career.as_ref() {
                for (name, centre) in career.system.diagnostic_sign_up_entries() {
                    trace(format_args!("diagnostic: sign-up box {name} at {centre:?}"));
                }
                for (player, local, valid, base) in career.system.diagnostic_skater_frames() {
                    trace(format_args!("diagnostic: skater frame of player {player}: local {local}, tester valid {valid} at {base:?}"));
                }
                for (player, challenge, volumes, conditions) in career.system.diagnostic_objective_volumes() {
                    trace(format_args!("diagnostic: objective group of player {player} for {challenge:016x}: colliding volumes {volumes:x?}; conditions {conditions:?}"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blob_comes_back_as_leaves() {
        let leaves: Vec<Leaf> = vec![
            (crate::challenge::signpost_progress::DATA, 14, vec![vec![1, 2, 3], vec![]]),
            (crate::challenge::signpost_progress::MESSAGES, 1, vec![vec![9; 300]]),
            (crate::challenge::unlockables::DATA, 1, vec![vec![4, 5]]),
            (crate::challenge::dynamic_hull::DATA, 3, vec![vec![0, 0, 0, 2], vec![7, 7]]),
        ];
        let data = progression_blob(&leaves);
        assert_eq!(progression_leaves(&data), Some(leaves.clone()));
        assert_eq!(progression_leaves(&data[..data.len() - 1]), None, "a cut blob is refused");
        let mut other = data.clone();
        other[4] = b'x';
        assert_eq!(progression_leaves(&other), None, "an unknown savable is refused");
        assert_eq!(progression_leaves(&[]), Some(Vec::new()));
    }

    #[test]
    fn the_blob_leaves_the_skaters_own_orientation_out() {
        let shared: Vec<Leaf> = vec![
            (crate::challenge::signpost_progress::DATA, 7, vec![vec![1, 2, 3]]),
            (crate::challenge::signpost_progress::MESSAGES, 1, vec![]),
        ];
        let mut with_position = shared.clone();
        with_position.insert(1, (crate::challenge::signpost_progress::POSITION, 1, vec![vec![0x3f, 0x2a, 0xc6, 0xaa]]));
        let mut other_position = shared.clone();
        other_position.insert(1, (crate::challenge::signpost_progress::POSITION, 1, vec![vec![0x3f, 0x2a, 0xc6, 0xae]]));
        assert_eq!(progression_blob(&with_position), progression_blob(&shared));
        assert_eq!(fnv1a(&progression_blob(&with_position)), fnv1a(&progression_blob(&other_position)));
        assert_ne!(fnv1a(&progression_blob(&shared)), fnv1a(&progression_blob(&shared[..1])));
    }
}

#[cfg(test)]
mod win_ats_tests {
    use super::{WinAt, win_ats_of};

    #[test]
    fn ticks_and_running_delays_parse_and_the_rest_drops() {
        assert_eq!(win_ats_of("1800; running+300 ;x;running+"), vec![WinAt::Tick(1800), WinAt::AfterRunning(300)]);
        assert!(win_ats_of("").is_empty());
    }
}

#[cfg(test)]
mod tick_names_tests {
    use super::tick_names_of;

    #[test]
    fn entries_and_bare_names() {
        let spec = tick_names_of("900:m_a; m_b ;1800:m_c;;x:m_d;1800:");
        assert_eq!(spec, vec![(900, "m_a".to_owned()), (900, "m_b".to_owned()), (1800, "m_c".to_owned()), (1800, String::new())]);
        assert!(tick_names_of("m_a;900:m_b").len() == 1, "a bare name before any tick is dropped");
        assert!(tick_names_of("").is_empty());
    }
}
