//! Co-op fork: the campaign's shared progression on the session (M2, `docs/coop/plan/topics/
//! progression.md`). The round itself is the port's facts lockstep (`online_challenge.rs`,
//! `start_campaign_round`); this file holds what is around it: the hold before the round starts,
//! the progression save sent to the server, and the bench's deterministic inputs at a round tick.

use super::{SelectedLiveError, SelectedLiveSession};
use crate::coop::trace;
use skate2_net::protocol::ClientMessage;

type Leaf = (&'static str, u32, Vec<Vec<u8>>);

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
fn diagnostic_completes() -> &'static [u32] {
    static SPEC: std::sync::OnceLock<Vec<u32>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| {
        std::env::var("SKATE2_DIAGNOSTIC_COOP_COMPLETE").ok().map_or_else(Vec::new, |spec| spec.split(';').filter_map(|tick| tick.trim().parse().ok()).collect())
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

/// `SKATE2_DIAGNOSTIC_COOP_EXECUTE=<tick>:<name>`: at that round tick the named challenge is
/// activated and executed, the same tick on every console (the bench's way into a mission).
fn diagnostic_execute() -> Option<(u32, String)> {
    static SPEC: std::sync::OnceLock<Option<(u32, String)>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| {
        let spec = std::env::var("SKATE2_DIAGNOSTIC_COOP_EXECUTE").ok()?;
        let (tick, name) = spec.split_once(':')?;
        Some((tick.trim().parse().ok()?, name.trim().to_owned()))
    })
    .clone()
}

/// The progression savables as one byte string, for the server's store and the hash every
/// console's log can compare: each leaf's name, version and tuples.
fn progression_blob(leaves: &[Leaf]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, version, tuples) in leaves {
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

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, &byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3))
}

impl<T, C> SelectedLiveSession<T, C> {
    /// The career is loaded in a campaign and the round has not started.
    pub(in super::super) fn online_campaign_holding(&self) -> bool {
        self.online_campaign() && self.online.campaign_hold && self.online.round.is_none()
    }

    /// The career autosaved in a campaign: its progression to the server, hashed.
    pub(in super::super) fn online_campaign_saved(&mut self, leaves: &[Leaf], unlocks: &[Leaf], hulls: &[Leaf]) {
        let Some(tick) = self.campaign_round_tick() else { return };
        let mut all: Vec<Leaf> = Vec::new();
        all.extend_from_slice(leaves);
        all.extend_from_slice(unlocks);
        all.extend_from_slice(hulls);
        let data = progression_blob(&all);
        let hash = fnv1a(&data);
        trace(format_args!("campaign save at round tick {tick}: {} B, hash {hash:016x}, to the server", data.len()));
        self.online.send(ClientMessage::CampaignSave { tick, hash, data });
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
        if let Some((at, name)) = diagnostic_execute()
            && tick == at
            && let Some(career) = self.career.as_mut()
        {
            let executed = career.system.diagnostic_execute_challenge(&name);
            trace(format_args!("diagnostic: execute {name} at round tick {tick}: {executed}"));
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
        if diagnostic_completes().contains(&tick)
            && let Some(career) = self.career.as_mut()
        {
            let completed = career.system.diagnostic_complete_active_objectives();
            trace(format_args!("diagnostic: completed the active objectives at round tick {tick}: {completed:?}"));
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
