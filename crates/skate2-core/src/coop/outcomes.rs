//! Co-op fork (M9, the competing kinds): what a script decides about its own player, carried as
//! a note on that player's facts (`skate2_net::protocol::Outcome`) and applied to that player's
//! copy of the objectives on every console at the note's tick, the deciding console's included.
//!
//! M13: a decision names its player. Every console's scripts hear every human's skater events
//! in lockstep (a human's event as player 0's, acting on that human's copy of the objectives:
//! `World::event_player`), so a script's forced outcome in reaction to a human's event is that
//! human's and is applied where it is made, at the event's tick, on every console alike
//! (`challenge::system::apply_coop_outcome_to`): a note of it would land a lockstep lead
//! later, after the restart the same handler's timer starts. A decision made outside any
//! human's event (the AI's run, a scene's end, the bench's win) stays the deciding console's
//! own player's and travels as a note naming it.
//!
//! In a campaign every console runs every mission script against its own AI skaters
//! ([`crate::challenge::players::script_slot`]), so a script that reads its AI's run (S.K.A.T.E.,
//! the jams, the contests) decides for its own player alone: an objective forced complete, the
//! objectives all failed, the mission won or lost. The ledger's "the one who did it" kind,
//! as the prompts: the decision is not applied where it was made but travels, and the rules
//! (`coop::rules`) settle the mission from every player's copy. A script's decision that follows
//! the shared objective completion (`Challenge.CompleteChallenge` once the copies are complete,
//! the same tick on every console) is applied where it is made, as before.

/// One decision of a script about a player, for the activity named by its challenge key.
/// A script may hold several groups of one activity for a player (`MakeObjectives` and
/// `MakeObjectivesForAI`): `group` is the group's ordinal among the player's groups of the
/// activity, in creation order. `player` is the campaign slot the decision is about (M13);
/// on the wire it rides the value's top byte, plus one, so a note of a build before M13
/// (never one this protocol admits) would read as the sender's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// `Objectives.ForceObjectiveComplete(group, index)` of the player's copy.
    ObjectiveComplete { activity: u64, group: u8, index: u8, player: i32 },
    /// `Objectives.FailAllObjectiveStatus(group)` of the player's copy.
    ObjectivesFailed { activity: u64, group: u8, player: i32 },
    /// `Challenge.CompleteChallenge(activity, processResults)` by a script while the activity's
    /// copies had not completed it under its rule: the player won on its own terms.
    Won { activity: u64, process: bool, player: i32 },
    /// `Challenge.LoseChallenge(activity)`: the player lost, with the result code the native
    /// gives (`end_challenge_unwon`), carried as a 16-bit value.
    Lost { activity: u64, result: i32, player: i32 },
}

/// The wire's player byte: the slot plus one, 0 for none.
const PLAYER_SHIFT: i32 = 24;
const PAYLOAD_MASK: i32 = 0x00ff_ffff;

impl Outcome {
    pub fn to_wire(self) -> (u8, u64, i32) {
        let tag = |player: i32| (player + 1).clamp(0, 0x7f) << PLAYER_SHIFT;
        match self {
            Self::ObjectiveComplete { activity, group, index, player } => (1, activity, tag(player) | i32::from(group) << 8 | i32::from(index)),
            Self::ObjectivesFailed { activity, group, player } => (2, activity, tag(player) | i32::from(group)),
            Self::Won { activity, process, player } => (3, activity, tag(player) | i32::from(process)),
            Self::Lost { activity, result, player } => (4, activity, tag(player) | i32::from(result.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16 as u16)),
        }
    }

    pub fn from_wire(kind: u8, activity: u64, value: i32) -> Option<Self> {
        let byte = |shift: i32| u8::try_from((value >> shift) & 0xff).ok();
        let player = ((value >> PLAYER_SHIFT) & 0x7f) - 1;
        let payload = value & PAYLOAD_MASK;
        Some(match kind {
            1 => Self::ObjectiveComplete { activity, group: byte(8)?, index: byte(0)?, player },
            2 => Self::ObjectivesFailed { activity, group: byte(0)?, player },
            3 => Self::Won { activity, process: payload != 0, player },
            4 => Self::Lost { activity, result: i32::from((payload & 0xffff) as u16 as i16), player },
            _ => return None,
        })
    }

    pub fn activity(self) -> u64 {
        match self {
            Self::ObjectiveComplete { activity, .. } | Self::ObjectivesFailed { activity, .. } | Self::Won { activity, .. } | Self::Lost { activity, .. } => activity,
        }
    }

    /// The campaign slot the decision is about; negative for none (the sender's).
    pub fn player(self) -> i32 {
        match self {
            Self::ObjectiveComplete { player, .. } | Self::ObjectivesFailed { player, .. } | Self::Won { player, .. } | Self::Lost { player, .. } => player,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Outcome;

    #[test]
    fn every_outcome_survives_the_wire() {
        for outcome in [
            Outcome::ObjectiveComplete { activity: 0x1234_5678_9abc_def0, group: 1, index: 3, player: 0 },
            Outcome::ObjectiveComplete { activity: 2, group: 0, index: 255, player: 7 },
            Outcome::ObjectivesFailed { activity: 7, group: 2, player: 1 },
            Outcome::ObjectivesFailed { activity: 7, group: 2, player: -1 },
            Outcome::Won { activity: u64::MAX, process: true, player: 3 },
            Outcome::Won { activity: 9, process: false, player: -1 },
            Outcome::Lost { activity: 1, result: -2, player: 1 },
            Outcome::Lost { activity: 1, result: 1, player: 0 },
        ] {
            let (kind, activity, value) = outcome.to_wire();
            assert_eq!(Outcome::from_wire(kind, activity, value), Some(outcome));
        }
        assert_eq!(Outcome::from_wire(9, 0, 0), None);
        // A value without a player byte names no player: the sender's.
        assert_eq!(Outcome::from_wire(2, 5, 3), Some(Outcome::ObjectivesFailed { activity: 5, group: 3, player: -1 }));
    }
}
