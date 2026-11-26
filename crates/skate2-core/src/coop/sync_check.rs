//! Co-op fork (M23): the campaign round's sync check, the original's `Sk8::Net::SyncChecker`
//! (`docs/online/lockstep.md` §9) over the port's facts lockstep.
//!
//! **The original.** After every simulated frame `SyncChecker::GenerateChecksum` hashes the
//! world (`Simulation::GetSyncLog`: the tick, the random seed, the commands, every entity, the
//! physics world, the movable objects) to one byte, and the GameSync of frame N carries the
//! byte of frame N − 45; `DoSyncChecking` compares it with the console's own as the frame is
//! released and a difference kicks the player ("kicked for unknown reason").
//!
//! **Here.** Only the campaign round (the scripts) runs in lockstep, so the round's state after
//! a tick is what every console must agree on. It is hashed in [`Part`]s, so a difference names
//! the part that diverged first rather than one byte for everything: the frames the scripts
//! took in (a difference there is the network's, everything else the simulation's), the
//! activities' graphs, the objectives, the timers and the scripts' vars. Every
//! [`CHECK_INTERVAL`] ticks the round's next facts carry its parts at the newest such tick it
//! ran (16 bits each), named by the tick, so no frame-offset convention ties the two consoles;
//! a console compares a player's check against its own record of that tick when it runs the
//! tick the check came with. A difference is reported, not kicked: the bench and play want to
//! know where the games parted, and the round goes on.

use std::collections::{BTreeMap, VecDeque};

/// The parts of the round's state, each hashed on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The frames the scripts took in at the tick (each player's facts as the round runs them).
    Inputs,
    /// Each activity's graph: the current state and whether it runs; the main activity.
    Activities,
    /// Each objective group and objective: their states, points and counters.
    Objectives,
    /// The challenge timers.
    Timers,
    /// The activities' vars tables in the challenge's Lua state.
    Vars,
}

/// How many parts a check carries.
pub const PARTS: usize = 5;

impl Part {
    pub const ALL: [Self; PARTS] = [Self::Inputs, Self::Activities, Self::Objectives, Self::Timers, Self::Vars];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Inputs => "inputs",
            Self::Activities => "activities",
            Self::Objectives => "objectives",
            Self::Timers => "timers",
            Self::Vars => "vars",
        }
    }
}

/// A check goes with the facts every this many round ticks (four a second at 60 Hz).
pub const CHECK_INTERVAL: u32 = 15;
/// The checkpoints a console keeps its own parts of: a player's check may name a tick this far
/// behind the tick it came with (its lead after a long stall).
const KEPT_CHECKPOINTS: usize = 240;
/// The ticks whose full parts a console keeps, for the window it prints at a difference.
const KEPT_TICKS: usize = CHECK_INTERVAL as usize * 2;

/// FNV-1a, 32 bits: the round's hashes are alike on every console that holds the same state.
#[derive(Clone, Copy, Debug)]
pub struct Fnv(u32);

impl Default for Fnv {
    fn default() -> Self {
        Self(0x811c_9dc5)
    }
}

impl Fnv {
    pub fn bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0 ^ u32::from(byte)).wrapping_mul(0x0100_0193);
        }
    }
    pub fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }
    pub fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }
    pub fn i32(&mut self, value: i32) {
        self.bytes(&value.to_le_bytes());
    }
    pub fn bool(&mut self, value: bool) {
        self.bytes(&[u8::from(value)]);
    }
    /// A float by its bits, with the two zeros alike and every NaN one value.
    pub fn f32(&mut self, value: f32) {
        let bits = if value == 0.0 {
            0
        } else if value.is_nan() {
            0x7fc0_0000
        } else {
            value.to_bits()
        };
        self.u32(bits);
    }
    pub fn str(&mut self, value: &str) {
        self.u32(u32::try_from(value.len()).unwrap_or(u32::MAX));
        self.bytes(value.as_bytes());
    }
    pub fn option<T>(&mut self, value: Option<T>, mut each: impl FnMut(&mut Self, T)) {
        self.bool(value.is_some());
        if let Some(value) = value {
            each(self, value);
        }
    }
    #[must_use]
    pub const fn finish(self) -> u32 {
        self.0
    }
}

/// The round's parts after a tick, in [`Part::ALL`]'s order.
pub type Parts = [u32; PARTS];

/// The 16 bits of a part a check carries.
#[must_use]
pub const fn wire_part(part: u32) -> u16 {
    (part ^ (part >> 16)) as u16
}

/// What a comparison found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Compared {
    /// The player's parts at the tick are this console's.
    Agrees,
    /// They differ in these parts (a bit per part, [`Part::ALL`]'s order); the first difference
    /// with this player, or the first after it agreed again.
    Differs { mask: u8, first: bool },
    /// A difference again, the player not having agreed since.
    StillDiffers { mask: u8 },
    /// This console holds no record of the tick (the round here started later, or the check
    /// is older than what it keeps).
    Unknown,
}

/// The names of the parts in `mask`.
#[must_use]
pub fn part_names(mask: u8) -> String {
    let names: Vec<&str> = Part::ALL.iter().enumerate().filter(|(index, _)| mask & (1 << index) != 0).map(|(_, part)| part.name()).collect();
    names.join("+")
}

/// One player's standing with this console.
#[derive(Clone, Copy, Debug, Default)]
struct Peer {
    checked: u32,
    differed: u32,
    /// The parts that differed at the player's last check, none while it agrees.
    differs: u8,
    /// The first checkpoint the player differed at, and its parts.
    first: Option<(u32, u8)>,
}

/// The round's sync check (`Sk8::Net::SyncChecker`).
#[derive(Debug, Default)]
pub struct SyncCheck {
    /// This console's parts at its checkpoints, oldest first.
    checkpoints: VecDeque<(u32, Parts)>,
    /// Its parts at its last ticks, oldest first.
    ticks: VecDeque<(u32, Parts)>,
    /// The newest checkpoint not yet carried by facts.
    unsent: Option<(u32, Parts)>,
    peers: BTreeMap<u64, Peer>,
}

impl SyncCheck {
    /// The round ran `tick` and its state hashed to `parts`.
    pub fn record(&mut self, tick: u32, parts: Parts) {
        if self.ticks.len() == KEPT_TICKS {
            self.ticks.pop_front();
        }
        self.ticks.push_back((tick, parts));
        if tick % CHECK_INTERVAL == 0 {
            if self.checkpoints.len() == KEPT_CHECKPOINTS {
                self.checkpoints.pop_front();
            }
            self.checkpoints.push_back((tick, parts));
            self.unsent = Some((tick, parts));
        }
    }

    /// The check the next facts carry: the newest checkpoint not yet sent.
    pub fn take_unsent(&mut self) -> Option<(u32, [u16; PARTS])> {
        self.unsent.take().map(|(tick, parts)| (tick, parts.map(wire_part)))
    }

    /// This console's parts at `tick`, if it keeps them.
    #[must_use]
    pub fn parts_at(&self, tick: u32) -> Option<Parts> {
        self.checkpoints.iter().chain(&self.ticks).find(|(at, _)| *at == tick).map(|(_, parts)| *parts)
    }

    /// A player's check of `tick`, against this console's own.
    pub fn compare(&mut self, user_id: u64, tick: u32, theirs: [u16; PARTS]) -> Compared {
        let Some(ours) = self.checkpoints.iter().find(|(at, _)| *at == tick).map(|(_, parts)| *parts) else { return Compared::Unknown };
        let mask = ours.iter().zip(theirs).enumerate().filter(|(_, (ours, theirs))| wire_part(**ours) != *theirs).fold(0_u8, |mask, (index, _)| mask | (1 << index));
        let peer = self.peers.entry(user_id).or_default();
        peer.checked += 1;
        if mask == 0 {
            peer.differs = 0;
            return Compared::Agrees;
        }
        peer.differed += 1;
        let again = peer.differs != 0;
        peer.differs = mask;
        if again {
            return Compared::StillDiffers { mask };
        }
        let first = peer.first.is_none();
        if first {
            peer.first = Some((tick, mask));
        }
        Compared::Differs { mask, first }
    }

    /// Each player's checks so far: (user id, checked, differed, the first difference).
    #[must_use]
    pub fn standings(&self) -> Vec<(u64, u32, u32, Option<(u32, u8)>)> {
        self.peers.iter().map(|(&user_id, peer)| (user_id, peer.checked, peer.differed, peer.first)).collect()
    }

    /// The ticks this console keeps whole, oldest first, for the window printed at a difference.
    pub fn kept_ticks(&self) -> impl Iterator<Item = &(u32, Parts)> {
        self.ticks.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(seed: u32) -> Parts {
        [seed, seed + 1, seed + 2, seed + 3, seed + 4]
    }

    #[test]
    fn a_checkpoint_goes_once_with_the_next_facts() {
        let mut check = SyncCheck::default();
        check.record(14, parts(1));
        assert_eq!(check.take_unsent(), None);
        check.record(15, parts(2));
        check.record(16, parts(3));
        assert_eq!(check.take_unsent(), Some((15, parts(2).map(wire_part))));
        assert_eq!(check.take_unsent(), None);
    }

    #[test]
    fn a_difference_names_its_parts_and_the_first_one_is_kept() {
        let mut ours = SyncCheck::default();
        let mut theirs = SyncCheck::default();
        for tick in 0..=45 {
            let mut state = parts(tick * 10);
            ours.record(tick, state);
            if tick >= 31 {
                state[3] ^= 0x55;
            }
            theirs.record(tick, state);
        }
        let check = |tick: u32| (tick, theirs.parts_at(tick).expect("kept").map(wire_part));
        let (tick, sent) = check(15);
        assert_eq!(ours.compare(7, tick, sent), Compared::Agrees);
        let (tick, sent) = check(30);
        assert_eq!(ours.compare(7, tick, sent), Compared::Agrees);
        let (tick, sent) = check(45);
        assert_eq!(ours.compare(7, tick, sent), Compared::Differs { mask: 1 << 3, first: true });
        assert_eq!(part_names(1 << 3), "timers");
        assert_eq!(ours.compare(7, tick, sent), Compared::StillDiffers { mask: 1 << 3 });
        assert_eq!(ours.standings(), vec![(7, 4, 2, Some((45, 1 << 3)))]);
        assert_eq!(ours.compare(7, 999, sent), Compared::Unknown);
    }

    #[test]
    fn agreeing_again_makes_the_next_difference_new_but_not_the_first() {
        let mut check = SyncCheck::default();
        check.record(0, parts(0));
        check.record(15, parts(1));
        let mut wrong = parts(0).map(wire_part);
        wrong[0] ^= 1;
        assert_eq!(check.compare(1, 0, wrong), Compared::Differs { mask: 1, first: true });
        assert_eq!(check.compare(1, 15, parts(1).map(wire_part)), Compared::Agrees);
        assert_eq!(check.compare(1, 0, wrong), Compared::Differs { mask: 1, first: false });
    }

    #[test]
    fn the_hash_treats_the_zeros_and_the_nans_alike() {
        let hash = |value: f32| {
            let mut fnv = Fnv::default();
            fnv.f32(value);
            fnv.finish()
        };
        assert_eq!(hash(0.0), hash(-0.0));
        assert_eq!(hash(f32::NAN), hash(-f32::NAN));
        assert_ne!(hash(1.0), hash(-1.0));
    }
}
