//! Who simulates a living-world entity. One rule per entity kind, one funnel that names every
//! decision (entity, old owner, new owner, reason), so a wrong owner is never a mystery.
//!
//! M1 knows one rule: the console whose census spawned an entity owns it (`Rule::Spawner`).
//! Nearest-player ownership with hysteresis and migration is M6's (`design.md` §5).

use super::trace;

/// The console that simulates an entity: this one, or another by its slot in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Local,
    Remote(u8),
}

/// A kind of living-world entity the world stream carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Pedestrian = 0,
}

impl Kind {
    pub fn from_wire(kind: u8) -> Option<Self> {
        match kind {
            0 => Some(Self::Pedestrian),
            _ => None,
        }
    }
}

/// How a kind's owner is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// The console whose census spawned it, for as long as it lives.
    Spawner,
}

/// The rule for a kind. One table, read here and nowhere else.
pub fn rule(kind: Kind) -> Rule {
    match kind {
        Kind::Pedestrian => Rule::Spawner,
    }
}

/// The owner an entity gets under its kind's rule.
pub fn owner(kind: Kind, spawner: Owner) -> Owner {
    match rule(kind) {
        Rule::Spawner => spawner,
    }
}

/// The funnel: every ownership decision passes here and is traced.
pub fn decide(kind: Kind, id: u32, old: Option<Owner>, new: Owner, reason: &str) -> Owner {
    if old != Some(new) {
        trace(format_args!("ownership: {kind:?} {id:08x} {old:?} -> {new:?} ({reason})"));
    }
    new
}
