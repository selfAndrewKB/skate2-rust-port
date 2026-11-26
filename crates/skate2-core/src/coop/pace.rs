//! Co-op fork (M22): the campaign round's pace, the original's online pace control
//! (`docs/online/lockstep.md` §8) over the port's facts lockstep.
//!
//! **Input rate.** `OnlineNetwork::UpdateInputRate` 0x8278e4b8 trims a console's timer by up to
//! 4 % so its input does not run far ahead of what its peers let it simulate. The port's
//! campaign skater runs on its own console and only the round (the scripts) waits for every
//! player's facts, so "sent and not yet simulated" is the round's lead: the local facts
//! produced and not yet run. A console whose round trails its own skater (it started first, or
//! its peer stalled) slows; one with headroom speeds up; the gap a round starts with closes.
//!
//! **Stall.** `SimController::IsSimulationStalled`: no round tick for `kSimulationStallTime`
//! (1.0 s; here 60 simulation ticks) while the round runs; `StateOnlineGame::UpdateChyron` 0x82748858 then shows "waiting
//! for players" until ticks flow again.

use std::time::Duration;

/// `kMaxSyncCommands`: the frames a console may send ahead of its own simulation.
const MAX_SYNC_COMMANDS: f32 = 45.0;
/// The target's floor, a third of the window (`45 × 0.333`), and its ceiling.
const LEAST_TARGET: f32 = MAX_SYNC_COMMANDS * 0.333;
const MOST_TARGET: f32 = 44.0;
/// The sample's range: 4 % slower to 4 % faster.
const SLOWEST: f32 = 0.96;
const FASTEST: f32 = 1.04;
/// `rate = 0.9 rate + 0.1 sample`.
const SAMPLE_WEIGHT: f32 = 0.1;
/// `UpdatePing`'s exponential mean, per update.
const PING_WEIGHT: f32 = 0.001;
/// The timer's baseline (`SetTimerHz(60)`).
pub const BASE_HZ: i32 = 60;
/// `kSimulationStallTime`'s default (1.0 s) in simulation ticks at the baseline.
pub const STALL_TICKS: u32 = 60;

/// The input rate and what set it, for the trace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paced {
    pub hz: i32,
    /// The local facts the round has not run yet; none when the round runs alone.
    pub lead: Option<u32>,
    /// `free / target` before the clamp.
    pub ratio: f32,
    pub target: f32,
    pub ping_ms: f32,
}

/// `UpdateInputRate`'s state.
#[derive(Clone, Copy, Debug)]
pub struct Pace {
    rate: f32,
    /// The mean round trip to the server, in milliseconds; none until the first ping returns.
    ping_ms: Option<f32>,
    hz: i32,
    /// The last update had another player pacing the round with the local skater.
    paced: bool,
}

impl Default for Pace {
    fn default() -> Self {
        Self { rate: 1.0, ping_ms: None, hz: BASE_HZ, paced: false }
    }
}

impl Pace {
    /// The timer rate for the 60 Hz baseline.
    #[must_use]
    pub const fn hz(&self) -> i32 {
        self.hz
    }

    /// Whether the last update had another player pacing the round with the local skater: alone,
    /// the timer is the game's own (`SetTimerHz(60)`, or the PC port's Online Speed).
    #[must_use]
    pub const fn paced(&self) -> bool {
        self.paced
    }

    /// One update: `lead` is the local facts produced and not yet run while other players pace
    /// the round with the local skater, none otherwise (alone, the timer runs at exactly 60 Hz);
    /// `rtt` is the last ping's round trip. The new rate, when the timer changes.
    pub fn update(&mut self, lead: Option<u32>, rtt: Option<Duration>) -> Option<Paced> {
        if let Some(rtt) = rtt {
            let ms = rtt.as_secs_f32() * 1000.0;
            // PC port: the mean starts at the first ping, not at zero.
            self.ping_ms = Some(self.ping_ms.map_or(ms, |mean| mean + PING_WEIGHT * (ms - mean)));
        }
        let ping_ms = self.ping_ms.unwrap_or(0.0);
        self.paced = lead.is_some();
        let target = (ping_ms * 60.0 / 1000.0 + 0.5).clamp(LEAST_TARGET, MOST_TARGET);
        let (ratio, hz) = if let Some(lead) = lead {
            #[expect(clippy::cast_precision_loss, reason = "a lead of facts is far under 2^24")]
            let free = MAX_SYNC_COMMANDS - lead as f32;
            let ratio = free / target;
            self.rate = (1.0 - SAMPLE_WEIGHT) * self.rate + SAMPLE_WEIGHT * ratio.clamp(SLOWEST, FASTEST);
            #[expect(clippy::cast_possible_truncation, reason = "the rate stays within 0.96..1.04")]
            let hz = (self.rate * 60.0 + 0.5) as i32;
            (ratio, hz)
        } else {
            self.rate = 1.0;
            (1.0, BASE_HZ)
        };
        (hz != self.hz).then(|| {
            self.hz = hz;
            Paced { hz, lead, ratio, target, ping_ms }
        })
    }
}

/// Whether the "waiting for players" notice opens or closes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    /// No round tick ran for [`STALL_TICKS`].
    Open,
    /// Ticks run again after the round waited this many simulation ticks.
    Close(u32),
}

/// `IsSimulationStalled` over the round's ticks, counted in this console's simulation ticks: its
/// own load or hitch stops both clocks, so only a wait on the other players counts.
#[derive(Clone, Copy, Debug, Default)]
pub struct StallWatch {
    waited: u32,
    shown: bool,
}

impl StallWatch {
    /// The notice is up.
    #[must_use]
    pub const fn shown(&self) -> bool {
        self.shown
    }

    /// One simulation tick of a running round: whether a round tick ran in it.
    pub fn update(&mut self, ran: bool) -> Option<Notice> {
        if ran {
            let waited = std::mem::take(&mut self.waited);
            return std::mem::take(&mut self.shown).then_some(Notice::Close(waited));
        }
        self.waited = self.waited.saturating_add(1);
        (!self.shown && self.waited >= STALL_TICKS).then(|| {
            self.shown = true;
            Notice::Open
        })
    }

    /// The round stopped running (a sync point, its end): the wait starts again when it runs.
    pub fn pause(&mut self) -> Option<Notice> {
        let waited = std::mem::take(&mut self.waited);
        std::mem::take(&mut self.shown).then_some(Notice::Close(waited))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(pace: &mut Pace, lead: Option<u32>, rtt: Option<Duration>) -> i32 {
        for _ in 0..200 {
            pace.update(lead, rtt);
        }
        pace.hz()
    }

    #[test]
    fn a_round_far_behind_its_skater_slows_to_58_and_one_with_headroom_runs_62() {
        // The probe's host: its round 122 ticks behind its skater. The joiner: 12.
        assert_eq!(settle(&mut Pace::default(), Some(122), None), 58);
        assert_eq!(settle(&mut Pace::default(), Some(12), None), 62);
        // At the target (45 − 15 = 30 ahead, low ping) the rate holds at 60.
        assert_eq!(settle(&mut Pace::default(), Some(30), None), 60);
    }

    #[test]
    fn a_long_ping_raises_the_target() {
        // 600 ms: target 36.5, so 12 ahead (62 Hz on a near link) is 33 free, under the
        // target: slower; 9 ahead is 36 free, just under: 59.
        assert_eq!(settle(&mut Pace::default(), Some(12), Some(Duration::from_millis(600))), 58);
        assert_eq!(settle(&mut Pace::default(), Some(9), Some(Duration::from_millis(600))), 59);
    }

    #[test]
    fn alone_the_timer_runs_at_sixty_at_once() {
        let mut pace = Pace::default();
        settle(&mut pace, Some(122), None);
        let paced = pace.update(None, None).expect("back to 60");
        assert_eq!((paced.hz, paced.lead), (BASE_HZ, None));
        assert_eq!(pace.update(Some(122), None), None, "the smoothing starts again from 60");
    }

    #[test]
    fn the_rate_moves_smoothly() {
        let mut pace = Pace::default();
        let mut changes = Vec::new();
        for _ in 0..100 {
            if let Some(paced) = pace.update(Some(200), None) {
                changes.push(paced.hz);
            }
        }
        assert_eq!(changes, [59, 58], "one hertz at a time");
    }

    #[test]
    fn the_notice_opens_after_a_second_without_a_tick_and_closes_when_ticks_flow() {
        let mut watch = StallWatch::default();
        assert_eq!(watch.update(true), None);
        for _ in 1..STALL_TICKS {
            assert_eq!(watch.update(false), None);
        }
        assert_eq!(watch.update(false), Some(Notice::Open));
        assert_eq!(watch.update(false), None, "opened once");
        assert_eq!(watch.update(true), Some(Notice::Close(STALL_TICKS + 1)));
        assert_eq!(watch.update(true), None);
        // A short wait (the round refilling its buffer) shows nothing.
        for _ in 0..10 {
            watch.update(false);
        }
        assert_eq!(watch.update(true), None);
    }

    #[test]
    fn a_paused_round_starts_its_wait_again() {
        let mut watch = StallWatch::default();
        for _ in 0..STALL_TICKS {
            watch.update(false);
        }
        assert_eq!(watch.pause(), Some(Notice::Close(STALL_TICKS)));
        for _ in 1..STALL_TICKS {
            assert_eq!(watch.update(false), None, "the wait starts when the round runs again");
        }
        assert_eq!(watch.update(false), Some(Notice::Open));
    }
}
