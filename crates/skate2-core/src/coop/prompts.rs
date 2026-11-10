//! The prompts a pad answers in the career (a dialog's Yes or No, a script popup's option, a
//! sign-up press, the several-challenges choice, a poster's accept or decline, Skate 101's and
//! the unlock overlay's continue, the phone's service purchase and its cancel), as the ledger's
//! "the one who did it" kind: in a campaign the
//! press on one console is not applied there but queued as a note on that player's next facts,
//! and every console (the presser's included) applies it at the tick the note arrives, so the
//! scripts take the same answer at the same tick everywhere. The wire form is
//! `skate2_net::protocol::Prompt`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptAnswer {
    /// The progression's Yes/No dialog, or a script's yes/no popup: `cMsgHUDYes`.
    Yes,
    /// `cMsgHUDNo`.
    No,
    /// A script's generic popup's option.
    Option(i32),
    /// Skate 101's continue.
    Skate101Continue,
    /// The unlock overlay's continue.
    UnlockOverlayContinue,
    /// The sign-up HUD's D-pad up (the prompt taken) and its release.
    SignUpDpadUp,
    SignUpDpadReleased,
    /// The several-challenges popup's choice, or its cancel.
    SignUpChoice(Option<i32>),
    /// The sign-up poster's accept and decline.
    PosterAccept,
    PosterDecline,
    /// The phone's service bought (`eServiceType`: 0 security, 1 uncapping, 2 draining), M5:
    /// the shared cash pays once and every console's scripts start the service.
    Service(i32),
    /// The phone's "cancel security" while Big Black is hired.
    ServiceCancel,
}

impl PromptAnswer {
    pub fn to_wire(self) -> (u8, i32) {
        match self {
            Self::Yes => (1, 0),
            Self::No => (2, 0),
            Self::Option(option) => (3, option),
            Self::Skate101Continue => (4, 0),
            Self::UnlockOverlayContinue => (5, 0),
            Self::SignUpDpadUp => (6, 0),
            Self::SignUpDpadReleased => (7, 0),
            Self::SignUpChoice(choice) => (8, choice.unwrap_or(-1)),
            Self::PosterAccept => (9, 0),
            Self::PosterDecline => (10, 0),
            Self::Service(service) => (11, service),
            Self::ServiceCancel => (12, 0),
        }
    }

    pub fn from_wire(kind: u8, value: i32) -> Option<Self> {
        Some(match kind {
            1 => Self::Yes,
            2 => Self::No,
            3 => Self::Option(value),
            4 => Self::Skate101Continue,
            5 => Self::UnlockOverlayContinue,
            6 => Self::SignUpDpadUp,
            7 => Self::SignUpDpadReleased,
            8 => Self::SignUpChoice((value >= 0).then_some(value)),
            9 => Self::PosterAccept,
            10 => Self::PosterDecline,
            11 => Self::Service(value),
            12 => Self::ServiceCancel,
            _ => return None,
        })
    }

    /// The bench's names (`SKATE2_DIAGNOSTIC_COOP_PROMPT`).
    pub fn parse(text: &str) -> Option<Self> {
        Some(match text.trim() {
            "yes" => Self::Yes,
            "no" => Self::No,
            "skate101" => Self::Skate101Continue,
            "unlock" => Self::UnlockOverlayContinue,
            "signup" => Self::SignUpDpadUp,
            "release" => Self::SignUpDpadReleased,
            "accept" => Self::PosterAccept,
            "decline" => Self::PosterDecline,
            "cancel" => Self::SignUpChoice(None),
            "service_cancel" => Self::ServiceCancel,
            other => {
                if let Some(option) = other.strip_prefix("option:") {
                    Self::Option(option.parse().ok()?)
                } else if let Some(choice) = other.strip_prefix("choice:") {
                    Self::SignUpChoice(Some(choice.parse().ok()?))
                } else if let Some(service) = other.strip_prefix("service:") {
                    Self::Service(service.parse().ok()?)
                } else {
                    return None;
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_answer_round_trips_the_wire_and_its_name() {
        let all = [
            PromptAnswer::Yes,
            PromptAnswer::No,
            PromptAnswer::Option(2),
            PromptAnswer::Skate101Continue,
            PromptAnswer::UnlockOverlayContinue,
            PromptAnswer::SignUpDpadUp,
            PromptAnswer::SignUpDpadReleased,
            PromptAnswer::SignUpChoice(Some(1)),
            PromptAnswer::SignUpChoice(None),
            PromptAnswer::PosterAccept,
            PromptAnswer::PosterDecline,
            PromptAnswer::Service(0),
            PromptAnswer::Service(2),
            PromptAnswer::ServiceCancel,
        ];
        for answer in all {
            let (kind, value) = answer.to_wire();
            assert_eq!(PromptAnswer::from_wire(kind, value), Some(answer));
        }
        assert_eq!(PromptAnswer::parse("choice:1"), Some(PromptAnswer::SignUpChoice(Some(1))));
        assert_eq!(PromptAnswer::parse("option:3"), Some(PromptAnswer::Option(3)));
        assert_eq!(PromptAnswer::parse("service:0"), Some(PromptAnswer::Service(0)));
        assert_eq!(PromptAnswer::parse("service_cancel"), Some(PromptAnswer::ServiceCancel));
        assert_eq!(PromptAnswer::parse("maybe"), None);
        assert_eq!(PromptAnswer::from_wire(99, 0), None);
    }
}
