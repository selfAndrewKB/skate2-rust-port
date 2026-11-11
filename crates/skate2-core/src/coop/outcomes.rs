//! Co-op fork (M9, the competing kinds): what a script decides about its own player, carried as
//! a note on that player's facts (`skate2_net::protocol::Outcome`) and applied to that player's
//! copy of the objectives on every console at the note's tick, the deciding console's included.
//!
//! In a campaign every console runs every mission script against its own AI skaters
//! ([`crate::challenge::players::script_slot`]), so a script that reads its AI's run (S.K.A.T.E.,
//! the jams, the contests) decides for its own player alone: an objective forced complete, the
//! objectives all failed, the mission won or lost. The ledger's "the one who did it" kind,
//! as the prompts: the decision is not applied where it was made but travels, and the rules
//! (`coop::rules`) settle the mission from every player's copy. A script's decision that follows
//! the shared objective completion (`Challenge.CompleteChallenge` once the copies are complete,
//! the same tick on every console) is applied where it is made, as before.

/// One decision of a script about its own player, for the activity named by its challenge key.
/// A script may hold several groups of one activity for its player (`MakeObjectives` and
/// `MakeObjectivesForAI`): `group` is the group's ordinal among the player's groups of the
/// activity, in creation order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// `Objectives.ForceObjectiveComplete(group, index)` of the player's copy.
    ObjectiveComplete { activity: u64, group: u8, index: u8 },
    /// `Objectives.FailAllObjectiveStatus(group)` of the player's copy.
    ObjectivesFailed { activity: u64, group: u8 },
    /// `Challenge.CompleteChallenge(activity, processResults)` by a script while the activity's
    /// copies had not completed it under its rule: the player won on its own terms.
    Won { activity: u64, process: bool },
    /// `Challenge.LoseChallenge(activity)`: the player lost, with the result code the native
    /// gives (`end_challenge_unwon`).
    Lost { activity: u64, result: i32 },
}

impl Outcome {
    pub fn to_wire(self) -> (u8, u64, i32) {
        match self {
            Self::ObjectiveComplete { activity, group, index } => (1, activity, i32::from(group) << 8 | i32::from(index)),
            Self::ObjectivesFailed { activity, group } => (2, activity, i32::from(group)),
            Self::Won { activity, process } => (3, activity, i32::from(process)),
            Self::Lost { activity, result } => (4, activity, result),
        }
    }

    pub fn from_wire(kind: u8, activity: u64, value: i32) -> Option<Self> {
        let byte = |shift: i32| u8::try_from((value >> shift) & 0xff).ok();
        Some(match kind {
            1 => Self::ObjectiveComplete { activity, group: byte(8)?, index: byte(0)? },
            2 => Self::ObjectivesFailed { activity, group: byte(0)? },
            3 => Self::Won { activity, process: value != 0 },
            4 => Self::Lost { activity, result: value },
            _ => return None,
        })
    }

    pub fn activity(self) -> u64 {
        match self {
            Self::ObjectiveComplete { activity, .. } | Self::ObjectivesFailed { activity, .. } | Self::Won { activity, .. } | Self::Lost { activity, .. } => activity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Outcome;

    #[test]
    fn every_outcome_survives_the_wire() {
        for outcome in [
            Outcome::ObjectiveComplete { activity: 0x1234_5678_9abc_def0, group: 1, index: 3 },
            Outcome::ObjectiveComplete { activity: 2, group: 0, index: 255 },
            Outcome::ObjectivesFailed { activity: 7, group: 2 },
            Outcome::Won { activity: u64::MAX, process: true },
            Outcome::Won { activity: 9, process: false },
            Outcome::Lost { activity: 1, result: -2 },
        ] {
            let (kind, activity, value) = outcome.to_wire();
            assert_eq!(Outcome::from_wire(kind, activity, value), Some(outcome));
        }
        assert_eq!(Outcome::from_wire(9, 0, 0), None);
    }
}
