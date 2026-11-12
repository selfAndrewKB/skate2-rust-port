//! Co-op fork: a pedestrian another console owns, driven here as a puppet. Its owner's world
//! stream (`coop::world_stream`) gives it a target transform at the stream's cadence; between
//! runs it carries on at the owner's velocity. The entity is otherwise a real pedestrian in the
//! census (its collision bodies follow the target as every pedestrian's do), so the local
//! skater, scoring and scripts see it as they see any pedestrian.

use crate::living_world_rain::geometry::Transform;

/// How many ticks a puppet carries on at its last velocity before it stands where it is: past
/// the owner's keepalive (M17: 20 runs of 3 ticks), since a sample that would say the same is
/// not sent.
const COAST_TICKS: u32 = 72;
/// Over how many ticks a sample's correction (its position and facing) is spread, instead of
/// taken in one step (M16, M19).
const BLEND_TICKS: u32 = 3;
/// A correction this large is a teleport, a knockdown's throw or a respawn: taken at once.
const SNAP_DISTANCE: f32 = 2.0;
/// A tick whose step strays from the walk by more than this counts as a stutter (M19's meter).
const STUTTER_METRES: f32 = 0.01;

#[derive(Clone, Debug)]
pub(crate) struct PuppetDrive {
    /// The owner's last transform and velocity, and the ticks since they arrived.
    pub(crate) target: Transform,
    pub(crate) velocity: [f32; 4],
    pub(crate) age: u32,
    /// The owner's last flags (`coop::world_stream::flags`).
    pub(crate) flags: u8,
    /// A full collision the local skater made with this puppet, waiting to be reported to its
    /// owner: force, direction, the skater's velocity and angular velocity at the hit.
    pub(crate) pending_contact: Option<(u8, u8, [f32; 3], [f32; 3])>,
    /// Whether the entity was in or entering its colliding behaviour at the last graph gate:
    /// the graph is ticked once more after the collision ends, so its exit (the behaviour's
    /// `End`, which clears the knocked-down flag and releases the collision's intent packet)
    /// runs. Without it a puppet stayed knocked down for good (2026-11-11).
    pub(crate) was_colliding: bool,
    /// M16: the state the owner's AI graph is in, from its last sample, which this puppet's
    /// graph is held at (`PedestrianGraph::update_delegate_held`); `None` before the first
    /// sample, or once a held update failed here (`held_failed`: the old gate from then on).
    pub(crate) graph: Option<u16>,
    pub(crate) held_failed: bool,
    /// M16: the facing turned to over the blend: the facing shown when the sample arrived and
    /// the sample's own.
    yaw_from: f32,
    yaw_to: f32,
    /// M19: the gap between where the puppet would have gone on this tick (its last position
    /// carried on at its last velocity) and where the new sample says it is: faded out over the
    /// blend, so the walk goes on unbroken through every sample.
    offset: [f32; 3],
    /// The position and facing shown last tick, and that tick's length.
    last_position: [f32; 3],
    last_yaw: f32,
    last_dt: f32,
    /// M19's meter: the ticks stepped, the summed stray of each tick's step from the walk (the
    /// displacement the velocity gives), and the ticks that strayed past `STUTTER_METRES`.
    pub(crate) stutter: Stutter,
    /// M19: the owner's hand prop this puppet waits to take while its model is not resident
    /// here (traced once per prop), 0 for none.
    pub(crate) prop_waiting: u64,
}

/// M19: how much a puppet's steps strayed from its walk, since the meter was last read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Stutter {
    pub(crate) ticks: u32,
    pub(crate) stray: f32,
    pub(crate) over: u32,
}

impl PuppetDrive {
    pub(crate) fn new(target: Transform) -> Self {
        let yaw = crate::coop::world_stream::yaw_of(&target);
        let at = [target[3][0], target[3][1], target[3][2]];
        Self {
            target, velocity: [0.; 4], age: 0, flags: 0, pending_contact: None, was_colliding: false, graph: None, held_failed: false,
            yaw_from: yaw, yaw_to: yaw, offset: [0.; 3], last_position: at, last_yaw: yaw, last_dt: 1. / 60., stutter: Stutter::default(), prop_waiting: 0,
        }
    }

    /// The velocity the drive carries the puppet on at, at an age.
    fn carried_velocity(&self, age: u32) -> [f32; 3] {
        if age < COAST_TICKS { [self.velocity[0], self.velocity[1], self.velocity[2]] } else { [0.; 3] }
    }

    /// The owner's next sample. The puppet goes on from where it would have been this tick,
    /// and the gap to the sample fades over the blend (a gap past `SNAP_DISTANCE` is taken at
    /// once).
    pub(crate) fn arrive(&mut self, target: Transform, velocity: [f32; 3], flags: u8) {
        let carried = self.carried_velocity(self.age);
        let here = [0, 1, 2].map(|lane| self.last_position[lane] + carried[lane] * self.last_dt);
        let sample = [target[3][0], target[3][1], target[3][2]];
        let offset = [0, 1, 2].map(|lane| here[lane] - sample[lane]);
        let gap = offset.iter().map(|value| value * value).sum::<f32>().sqrt();
        self.offset = if gap > SNAP_DISTANCE { [0.; 3] } else { offset };
        self.yaw_from = self.last_yaw;
        self.yaw_to = crate::coop::world_stream::yaw_of(&target);
        self.target = target;
        self.velocity = [velocity[0], velocity[1], velocity[2], 0.];
        self.age = 0;
        self.flags = flags;
    }

    /// The share of the blend still to come at an age: two thirds on the arrival tick, then a
    /// third, then none (the step on the arrival tick already moves a third of the way).
    fn remaining(age: u32) -> f32 {
        1. - ((age + 1).min(BLEND_TICKS) as f32) / BLEND_TICKS as f32
    }

    /// The velocity the puppet's animation walks at: the owner's last, or none once it has
    /// coasted and stands.
    pub(crate) fn locomotion_velocity(&self) -> [f32; 4] {
        if self.age < COAST_TICKS { self.velocity } else { [0.; 4] }
    }

    /// Whether the entity is in or entering its colliding behaviour, which its own graph
    /// plays even as a puppet (a knockdown's animation).
    pub(crate) fn colliding(collision: Option<&crate::pedestrian_collision::PedestrianCollision>) -> bool {
        collision.is_some_and(|collision| collision.required_to_begin_colliding || collision.colliding)
    }

    /// The transform the entity should reach this tick: the last sample carried on at its
    /// velocity (held where the coast ends), plus what is left of the sample's correction.
    /// M19: until M19 the step stood still on a sample's arrival tick and caught up over the
    /// next three (0, 0.7, 1.3 and 2 times the walk), and snapped back the whole coast when a
    /// sample was 72 ticks late: the stutter the player saw.
    pub(crate) fn step(&mut self, dt: f32) -> Transform {
        let coasted = self.age.min(COAST_TICKS) as f32 * dt;
        let remaining = Self::remaining(self.age);
        let position = [0, 1, 2].map(|lane| self.target[3][lane] + self.velocity[lane] * coasted + self.offset[lane] * remaining);
        let turn = (self.yaw_to - self.yaw_from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        let yaw = self.yaw_from + turn * (1. - remaining);
        // The meter: this tick's step against the walk the velocity gives.
        let walk = self.carried_velocity(self.age);
        let stray = [0, 1, 2].map(|lane| position[lane] - self.last_position[lane] - walk[lane] * dt).iter().map(|value| value * value).sum::<f32>().sqrt();
        self.stutter.ticks += 1;
        self.stutter.stray += stray;
        self.stutter.over += u32::from(stray > STUTTER_METRES);
        self.last_position = position;
        self.last_yaw = yaw;
        self.last_dt = dt;
        self.age = self.age.saturating_add(1);
        let turned = crate::coop::world_stream::transform(position, yaw);
        [turned[0], turned[1], turned[2], [position[0], position[1], position[2], self.target[3][3]]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1. / 60.;

    fn at(x: f32) -> Transform {
        crate::coop::world_stream::transform([x, 0., 0.], 0.)
    }

    /// An owner walking steadily, sampled every `every` ticks with no error: the puppet's every
    /// step after the first sample is the walk's own (the old blend stood still on each
    /// arrival tick and doubled up after).
    fn walk(every: u32, ticks: u32) -> Vec<f32> {
        let speed = 1.5;
        let mut drive = PuppetDrive::new(at(0.));
        drive.arrive(at(0.), [speed, 0., 0.], 0);
        let mut steps = Vec::new();
        let mut last = 0.;
        for tick in 0..ticks {
            if tick > 0 && tick % every == 0 {
                drive.arrive(at(speed * DT * tick as f32), [speed, 0., 0.], 0);
            }
            let x = drive.step(DT)[3][0];
            if tick > 0 {
                steps.push(x - last);
            }
            last = x;
        }
        steps
    }

    #[test]
    fn a_steady_walk_steps_evenly_through_every_sample() {
        for every in [3, 7, 60] {
            for step in walk(every, 240) {
                assert!((step - 1.5 * DT).abs() < 1e-4, "every {every}: a step of {step} m against {} m", 1.5 * DT);
            }
        }
    }

    #[test]
    fn a_correction_fades_over_the_blend() {
        let mut drive = PuppetDrive::new(at(0.));
        drive.arrive(at(0.), [0., 0., 0.], 0);
        drive.step(DT);
        // The owner is 5 cm further on than the puppet thought.
        drive.arrive(at(0.05), [0., 0., 0.], 0);
        let a = drive.step(DT)[3][0];
        let b = drive.step(DT)[3][0];
        let c = drive.step(DT)[3][0];
        let d = drive.step(DT)[3][0];
        assert!((a - 0.05 / 3.).abs() < 1e-5 && (b - 0.1 / 3.).abs() < 1e-5, "{a} {b}");
        assert!((c - 0.05).abs() < 1e-6 && (d - 0.05).abs() < 1e-6, "{c} {d}");
        assert_eq!(drive.stutter.over, 3, "a third of 5 cm a tick, 1.7 cm, on each of the three ticks of the blend");
    }

    #[test]
    fn a_teleport_is_taken_at_once() {
        let mut drive = PuppetDrive::new(at(0.));
        drive.arrive(at(0.), [0., 0., 0.], 0);
        drive.step(DT);
        drive.arrive(at(30.), [0., 0., 0.], 0);
        assert!((drive.step(DT)[3][0] - 30.).abs() < 1e-5);
    }

    #[test]
    fn a_late_sample_holds_at_the_coast_and_never_snaps_back() {
        let mut drive = PuppetDrive::new(at(0.));
        drive.arrive(at(0.), [1.5, 0., 0.], 0);
        let mut last = 0.;
        for _ in 0..200 {
            let x = drive.step(DT)[3][0];
            assert!(x >= last - 1e-6, "went back from {last} to {x}");
            last = x;
        }
        assert!((last - 1.5 * DT * COAST_TICKS as f32).abs() < 1e-4);
    }
}
