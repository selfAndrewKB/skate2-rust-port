//! Co-op fork: a pedestrian another console owns, driven here as a puppet. Its owner's world
//! stream (`coop::world_stream`) gives it a target transform at the stream's cadence; between
//! runs it carries on at the owner's velocity. The entity is otherwise a real pedestrian in the
//! census (its collision bodies follow the target as every pedestrian's do), so the local
//! skater, scoring and scripts see it as they see any pedestrian.

use crate::living_world_rain::geometry::Transform;

/// How many ticks a puppet carries on at its last velocity before it stands where it is.
const COAST_TICKS: u32 = 30;

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
}

impl PuppetDrive {
    pub(crate) fn new(target: Transform) -> Self {
        Self { target, velocity: [0.; 4], age: 0, flags: 0, pending_contact: None, was_colliding: false }
    }

    /// The owner's next sample.
    pub(crate) fn arrive(&mut self, target: Transform, velocity: [f32; 3], flags: u8) {
        self.target = target;
        self.velocity = [velocity[0], velocity[1], velocity[2], 0.];
        self.age = 0;
        self.flags = flags;
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
        self.age = self.age.saturating_add(1);
        target
    }
}
