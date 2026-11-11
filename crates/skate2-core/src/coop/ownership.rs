//! Who simulates a living-world entity. One rule per entity kind, one funnel that names every
//! decision (entity, old owner, new owner, reason), so a wrong owner is never a mystery.
//!
//! M1 knows one rule: the console whose census spawned an entity owns it (`Rule::Spawner`).
//! M5 adds the story characters a challenge places (Slappy outside his shop, Reda at the
//! flip-trick locator): every console spawns and simulates its own copy from the same scripts,
//! and none is streamed (`Rule::EachConsole`), because every reference to one is that console's
//! (its character token, its cull keep list, its marquee AI greeting the local skater) and no
//! script reads its existence into the facts. Nearest-player ownership with hysteresis and
//! migration is M6's (`design.md` §5).

use super::trace;

/// The console that simulates an entity: this one, or another by its slot in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Local,
    Remote(u8),
}

/// A kind of living-world entity the ownership table knows. The wire carries the streamed ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    /// An ambient pedestrian of the census.
    Pedestrian = 0,
    /// A pedestrian a challenge placed (`challenge_owner` set): a story character, or in profile
    /// free skate a no-skate-zone guard.
    StoryCharacter = 1,
    /// A security guard of the census (`security_guard` set): streamed as a pedestrian.
    SecurityGuard = 2,
    /// A dynamic object with a locator (a challenge prop, a movable box or rail): named across
    /// consoles by its locator id, streamed in the run's prop list.
    Prop = 3,
}

impl Kind {
    /// The kinds the run's entity list carries (a story character never travels; a prop has
    /// its own list).
    pub fn from_wire(kind: u8) -> Option<Self> {
        match kind {
            0 => Some(Self::Pedestrian),
            2 => Some(Self::SecurityGuard),
            _ => None,
        }
    }
}

/// How a kind's owner is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// The console whose census spawned it, for as long as it lives.
    Spawner,
    /// Every console's own: each spawns and simulates its copy, and none travels.
    EachConsole,
    /// The console of the player it chases. A guard chases only the skater of the console
    /// that simulates it (a remote skater is a replay, not an actor), so until M6's migration
    /// that is its spawner; its puppets elsewhere perceive and chase nobody.
    Chasee,
    /// The console whose skater touched it last: each touch claims it with a higher claim
    /// number, the highest claim wins everywhere, a tie the lower slot.
    LastToucher,
}

/// The rule for a kind. One table, read here and nowhere else.
pub fn rule(kind: Kind) -> Rule {
    match kind {
        Kind::Pedestrian => Rule::Spawner,
        Kind::StoryCharacter => Rule::EachConsole,
        Kind::SecurityGuard => Rule::Chasee,
        Kind::Prop => Rule::LastToucher,
    }
}

/// Whether a claim of `claim` by `slot` beats the known one.
pub fn claim_wins(known: Option<(u32, u8)>, claim: u32, slot: u8) -> bool {
    match known {
        None => true,
        Some((known_claim, known_slot)) => claim > known_claim || (claim == known_claim && slot < known_slot),
    }
}

/// The owner an entity gets under its kind's rule.
pub fn owner(kind: Kind, spawner: Owner) -> Owner {
    match rule(kind) {
        Rule::Spawner | Rule::Chasee | Rule::LastToucher => spawner,
        Rule::EachConsole => Owner::Local,
    }
}

/// Whether the owner streams a kind to the others.
pub fn streamed(kind: Kind) -> bool {
    match rule(kind) {
        Rule::Spawner | Rule::Chasee | Rule::LastToucher => true,
        Rule::EachConsole => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_names_every_kind() {
        assert_eq!(owner(Kind::Pedestrian, Owner::Remote(1)), Owner::Remote(1));
        assert_eq!(owner(Kind::SecurityGuard, Owner::Remote(1)), Owner::Remote(1));
        assert_eq!(owner(Kind::StoryCharacter, Owner::Remote(1)), Owner::Local);
        assert!(streamed(Kind::Pedestrian) && streamed(Kind::SecurityGuard) && !streamed(Kind::StoryCharacter));
        assert_eq!(Kind::from_wire(0), Some(Kind::Pedestrian));
        assert_eq!(Kind::from_wire(1), None, "a story character never travels");
        assert_eq!(Kind::from_wire(2), Some(Kind::SecurityGuard), "a guard travels as its kind");
        assert!(claim_wins(None, 1, 1));
        assert!(claim_wins(Some((1, 0)), 2, 1));
        assert!(!claim_wins(Some((2, 0)), 2, 1), "a tie goes to the lower slot");
        assert!(claim_wins(Some((2, 1)), 2, 0));
        assert!(!claim_wins(Some((2, 0)), 2, 0), "the owner's own continuing samples do not win: the receiver passes them as the owner's (coop_world), not as claims");
    }
}

/// The funnel: every ownership decision passes here and is traced.
pub fn decide(kind: Kind, id: u32, old: Option<Owner>, new: Owner, reason: &str) -> Owner {
    if old != Some(new) {
        trace(format_args!("ownership: {kind:?} {id:08x} {old:?} -> {new:?} ({reason})"));
    }
    new
}
