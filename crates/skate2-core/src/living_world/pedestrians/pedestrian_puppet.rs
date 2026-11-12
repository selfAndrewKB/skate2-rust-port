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
/// Over how many ticks a puppet turns to a sample's facing (the stream's cadence), instead of
/// snapping to it and holding it (M16).
const YAW_SLEW_TICKS: u32 = 3;

#[derive(Clone, Debug)]
pub(crate) struct PuppetDrive {
    /// The owner's last transform and velocity, and the ticks since they arrived.
    pub(crate) target: Transform,
    pub(crate) velocity: [f32; 4],
    pub(crate) age: u32,
    /// The owner's last flags (`coop::world_stream::flags`).
    pub(crate) flags: u8,
    /// A full collision the local skater made with this puppet, resolved here as a prediction
    /// and waiting to be reported to the owner: force, direction, the skater's velocity and
    /// angular velocity at the hit.
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
    /// M16: the facing turned to over the stream's cadence: the yaw the last sample arrived at
    /// and the sample's own.
    yaw_from: f32,
    yaw_to: f32,
    /// M19: where the puppet stood when the last sample arrived, and where it stood last tick:
    /// each sample's correction is spread over the cadence instead of taken in one step.
    position_from: [f32; 3],
    last_position: [f32; 3],
}

impl PuppetDrive {
    pub(crate) fn new(target: Transform) -> Self {
        let yaw = crate::coop::world_stream::yaw_of(&target);
        let at = [target[3][0], target[3][1], target[3][2]];
        Self { target, velocity: [0.; 4], age: 0, flags: 0, pending_contact: None, was_colliding: false, graph: None, held_failed: false, yaw_from: yaw, yaw_to: yaw, position_from: at, last_position: at }
    }

    /// The owner's next sample.
    pub(crate) fn arrive(&mut self, target: Transform, velocity: [f32; 3], flags: u8) {
        self.yaw_from = self.yaw();
        self.yaw_to = crate::coop::world_stream::yaw_of(&target);
        self.position_from = self.last_position;
        self.target = target;
        self.velocity = [velocity[0], velocity[1], velocity[2], 0.];
        self.age = 0;
        self.flags = flags;
    }

    /// The facing this tick: from the last sample's reached facing toward the new sample's,
    /// over the stream's cadence, the shorter way round.
    fn yaw(&self) -> f32 {
        let t = (self.age.min(YAW_SLEW_TICKS) as f32) / YAW_SLEW_TICKS as f32;
        let turn = (self.yaw_to - self.yaw_from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        self.yaw_from + turn * t
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
    /// velocity for a while, then held.
    pub(crate) fn step(&mut self, dt: f32) -> Transform {
        let mut target = self.target;
        if self.age < COAST_TICKS {
            let t = dt * self.age as f32;
            for lane in 0..3 {
                target[3][lane] += self.velocity[lane] * t;
            }
        }
        // M16: the facing turns over the cadence rather than snapping (the audit's 3 to 6 % of
        // samples beyond 5° were the held facing of a turning owner). M19: so does the position,
        // from where the puppet stood at the sample's arrival to the sample carried on: a sample
        // the send-on-error rule sent is one the extrapolation missed by 5 cm or more, and a
        // 5 cm step every few ticks was the lurch the player saw.
        let yaw = self.yaw();
        let t = (self.age.min(YAW_SLEW_TICKS) as f32) / YAW_SLEW_TICKS as f32;
        let position = [
            self.position_from[0] + (target[3][0] - self.position_from[0]) * t,
            self.position_from[1] + (target[3][1] - self.position_from[1]) * t,
            self.position_from[2] + (target[3][2] - self.position_from[2]) * t,
        ];
        self.last_position = position;
        self.age = self.age.saturating_add(1);
        let turned = crate::coop::world_stream::transform(position, yaw);
        [turned[0], turned[1], turned[2], [position[0], position[1], position[2], target[3][3]]]
    }
}
