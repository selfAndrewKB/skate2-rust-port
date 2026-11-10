//! The world stream's wire format: one run is one tick of the living-world entities a console
//! owns, plus the entities it stopped owning and the contacts its skater made with others'
//! entities. Runs are self-contained (every value absolute, a descriptor repeated now and
//! then), so a lost run costs nothing but a moment; they travel unreliably in
//! `ClientMessage::World` and must fit `MAX_WORLD` bytes.
//!
//! Format 1 (M1). Units: positions 1/256 m, velocities 1/64 m/s, yaw 1/65536 of a turn.
//! A change of meaning raises `FORMAT`; a receiver drops a run of another format.

use super::ownership::Kind;

pub const FORMAT: u8 = 1;

/// A world id names an entity across consoles: the owner's slot in the game (4 bits) over
/// the owner's local entity id (its type nibble and 24-bit counter).
pub fn world_id(slot: u8, local_id: u32) -> u32 {
    (u32::from(slot & 0xf) << 28) | (local_id & 0x0fff_ffff)
}
pub fn owner_slot(id: u32) -> u8 {
    (id >> 28) as u8
}
pub fn local_id(id: u32) -> u32 {
    id & 0x0fff_ffff
}

/// Entity flags.
pub mod flags {
    /// The entity is down (a pedestrian's knockdown).
    pub const KNOCKED_DOWN: u8 = 0x01;
    /// The entity is in its colliding behaviour (down or a standing hit).
    pub const COLLIDING: u8 = 0x02;
    /// Bits 2..3: the direction the last hit came from (`CollisionDirection` as a number).
    pub const DIRECTION_SHIFT: u8 = 2;
    pub const DIRECTION_MASK: u8 = 0x0c;
    /// The last hit was a standing one, not a knockdown.
    pub const STANDING_HIT: u8 = 0x10;
}

/// What a receiver needs to spawn its copy; sent with an entity's first appearance and
/// repeated now and then, since runs are not acknowledged.
#[derive(Clone, Debug, PartialEq)]
pub struct Descriptor {
    pub entity_key: u64,
    pub model_key: u64,
    pub recipe_id: u64,
    pub colours: [u8; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityState {
    pub id: u32,
    pub kind: Kind,
    pub position: [f32; 3],
    /// Radians about the up axis, `atan2(forward.x, forward.z)`.
    pub yaw: f32,
    pub velocity: [f32; 3],
    pub flags: u8,
    /// The owner's motion state (`PedStateOut::state`), for a puppet's animation later.
    pub motion: u8,
    pub descriptor: Option<Descriptor>,
}

/// A local skater's contact with another console's entity, for its owner to apply.
#[derive(Clone, Debug, PartialEq)]
pub struct Contact {
    pub id: u32,
    /// `CollisionForce` as a number: 0 knockdown, 1 standing.
    pub force: u8,
    /// `CollisionDirection` as a number.
    pub direction: u8,
    pub velocity: [f32; 3],
    pub omega: [f32; 3],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldRun {
    pub despawns: Vec<u32>,
    pub contacts: Vec<Contact>,
    pub entities: Vec<EntityState>,
}

fn varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}
fn zigzag(out: &mut Vec<u8>, value: i64) {
    varint(out, ((value << 1) ^ (value >> 63)) as u64);
}
fn quantized(value: f32, scale: f32, limit: i64) -> i64 {
    let scaled = (value * scale).round();
    if scaled.is_nan() {
        return 0;
    }
    (scaled as i64).clamp(-limit, limit)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn u8(&mut self) -> Result<u8, &'static str> {
        let byte = *self.bytes.get(self.at).ok_or("world run cut short")?;
        self.at += 1;
        Ok(byte)
    }
    fn varint(&mut self) -> Result<u64, &'static str> {
        let mut value = 0_u64;
        for shift in (0..64).step_by(7) {
            let byte = self.u8()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err("world run varint too long")
    }
    fn zigzag(&mut self) -> Result<i64, &'static str> {
        let raw = self.varint()?;
        Ok(((raw >> 1) as i64) ^ -((raw & 1) as i64))
    }
    fn u64(&mut self) -> Result<u64, &'static str> {
        let mut bytes = [0; 8];
        for byte in &mut bytes {
            *byte = self.u8()?;
        }
        Ok(u64::from_le_bytes(bytes))
    }
    fn count(&mut self, most: usize) -> Result<usize, &'static str> {
        let count = self.varint()?;
        if count > most as u64 {
            return Err("world run count too large");
        }
        Ok(count as usize)
    }
}

const MOST_DESPAWNS: usize = 256;
const MOST_CONTACTS: usize = 64;
const MOST_ENTITIES: usize = 256;
const POSITION_SCALE: f32 = 256.;
const VELOCITY_SCALE: f32 = 64.;

fn write_vector(out: &mut Vec<u8>, vector: [f32; 3], scale: f32, limit: i64) {
    for lane in vector {
        zigzag(out, quantized(lane, scale, limit));
    }
}
fn read_vector(r: &mut Reader<'_>, scale: f32) -> Result<[f32; 3], &'static str> {
    Ok([r.zigzag()? as f32 / scale, r.zigzag()? as f32 / scale, r.zigzag()? as f32 / scale])
}

fn write_entity(out: &mut Vec<u8>, entity: &EntityState) {
    varint(out, u64::from(entity.id));
    out.push(entity.kind as u8);
    out.push(entity.flags);
    out.push(entity.motion);
    write_vector(out, entity.position, POSITION_SCALE, i64::from(i32::MAX));
    let turns = entity.yaw / std::f32::consts::TAU;
    let yaw = ((turns - turns.floor()) * 65536.).round() as u32 & 0xffff;
    varint(out, u64::from(yaw));
    write_vector(out, entity.velocity, VELOCITY_SCALE, i64::from(i16::MAX));
    match &entity.descriptor {
        Some(descriptor) => {
            out.push(1);
            out.extend_from_slice(&descriptor.entity_key.to_le_bytes());
            out.extend_from_slice(&descriptor.model_key.to_le_bytes());
            out.extend_from_slice(&descriptor.recipe_id.to_le_bytes());
            out.extend_from_slice(&descriptor.colours);
        }
        None => out.push(0),
    }
}

/// Codes a run within `budget` bytes: the despawns and contacts always (they are few and must
/// arrive), then the entities in order until the budget is reached. Returns the bytes and how
/// many entities went in.
pub fn encode(run: &WorldRun, budget: usize) -> (Vec<u8>, usize) {
    let mut out = vec![FORMAT];
    let despawns = &run.despawns[..run.despawns.len().min(MOST_DESPAWNS)];
    varint(&mut out, despawns.len() as u64);
    for &id in despawns {
        varint(&mut out, u64::from(id));
    }
    let contacts = &run.contacts[..run.contacts.len().min(MOST_CONTACTS)];
    varint(&mut out, contacts.len() as u64);
    for contact in contacts {
        varint(&mut out, u64::from(contact.id));
        out.push(contact.force);
        out.push(contact.direction);
        write_vector(&mut out, contact.velocity, VELOCITY_SCALE, i64::from(i16::MAX));
        write_vector(&mut out, contact.omega, VELOCITY_SCALE, i64::from(i16::MAX));
    }
    let count_at = out.len();
    out.push(0);
    let mut included = 0_usize;
    for entity in run.entities.iter().take(MOST_ENTITIES) {
        let mut record = Vec::new();
        write_entity(&mut record, entity);
        // The count is one byte while it stays under 128 entities; past that a run is far over
        // any budget a datagram takes.
        if out.len() + record.len() > budget || included == 127 {
            break;
        }
        out.extend_from_slice(&record);
        included += 1;
    }
    out[count_at] = included as u8;
    (out, included)
}

pub fn decode(bytes: &[u8]) -> Result<WorldRun, &'static str> {
    let mut r = Reader { bytes, at: 0 };
    if r.u8()? != FORMAT {
        return Err("world run of another format");
    }
    let mut run = WorldRun::default();
    for _ in 0..r.count(MOST_DESPAWNS)? {
        run.despawns.push(r.varint()? as u32);
    }
    for _ in 0..r.count(MOST_CONTACTS)? {
        run.contacts.push(Contact {
            id: r.varint()? as u32,
            force: r.u8()?,
            direction: r.u8()?,
            velocity: read_vector(&mut r, VELOCITY_SCALE)?,
            omega: read_vector(&mut r, VELOCITY_SCALE)?,
        });
    }
    for _ in 0..r.count(MOST_ENTITIES)? {
        let id = r.varint()? as u32;
        let kind = Kind::from_wire(r.u8()?).ok_or("world run entity kind")?;
        let flags = r.u8()?;
        let motion = r.u8()?;
        let position = read_vector(&mut r, POSITION_SCALE)?;
        let yaw = (r.varint()? & 0xffff) as f32 / 65536. * std::f32::consts::TAU;
        let velocity = read_vector(&mut r, VELOCITY_SCALE)?;
        let descriptor = match r.u8()? {
            0 => None,
            1 => {
                let entity_key = r.u64()?;
                let model_key = r.u64()?;
                let recipe_id = r.u64()?;
                let colours = [r.u8()?, r.u8()?];
                Some(Descriptor { entity_key, model_key, recipe_id, colours })
            }
            _ => return Err("world run descriptor flag"),
        };
        run.entities.push(EntityState { id, kind, position, yaw, velocity, flags, motion, descriptor });
    }
    if r.at != bytes.len() {
        return Err("world run has trailing bytes");
    }
    Ok(run)
}

/// The yaw of a living-world transform (rows: right, up, forward, translation).
pub fn yaw_of(transform: &[[f32; 4]; 4]) -> f32 {
    transform[2][0].atan2(transform[2][2])
}

/// A living-world transform standing upright at a position, facing a yaw.
pub fn transform(position: [f32; 3], yaw: f32) -> [[f32; 4]; 4] {
    let (sin, cos) = yaw.sin_cos();
    [[cos, 0., -sin, 0.], [0., 1., 0., 0.], [sin, 0., cos, 0.], [position[0], position[1], position[2], 1.]]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> WorldRun {
        WorldRun {
            despawns: vec![world_id(1, 0x0200_0007)],
            contacts: vec![Contact { id: world_id(2, 0x0200_0001), force: 0, direction: 3, velocity: [6.5, -0.25, 1.], omega: [0., 2., 0.] }],
            entities: vec![
                EntityState {
                    id: world_id(0, 0x0200_0003),
                    kind: Kind::Pedestrian,
                    position: [1491.8145, 109.711, -743.305],
                    yaw: 2.5,
                    velocity: [1.25, 0., -0.5],
                    flags: flags::KNOCKED_DOWN | flags::COLLIDING | (2 << flags::DIRECTION_SHIFT),
                    motion: 8,
                    descriptor: Some(Descriptor { entity_key: 0x1234, model_key: 0xabcd_ef01_2345_6789, recipe_id: 7, colours: [3, 250] }),
                },
                EntityState {
                    id: world_id(0, 0x0200_0004),
                    kind: Kind::Pedestrian,
                    position: [-2000., 0., 2000.],
                    yaw: -3.0,
                    velocity: [0., 0., 0.],
                    flags: 0,
                    motion: 0,
                    descriptor: None,
                },
            ],
        }
    }

    #[test]
    fn a_run_round_trips_within_its_quantization() {
        let run = run();
        let (bytes, included) = encode(&run, 1024);
        assert_eq!(included, 2);
        let back = decode(&bytes).expect("decodes");
        assert_eq!(back.despawns, run.despawns);
        assert_eq!(back.contacts.len(), 1);
        assert_eq!(back.contacts[0].id, run.contacts[0].id);
        assert_eq!((back.contacts[0].force, back.contacts[0].direction), (0, 3));
        for lane in 0..3 {
            assert!((back.contacts[0].velocity[lane] - run.contacts[0].velocity[lane]).abs() <= 1. / 64.);
        }
        assert_eq!(back.entities.len(), 2);
        for (a, b) in back.entities.iter().zip(&run.entities) {
            assert_eq!((a.id, a.kind, a.flags, a.motion, &a.descriptor), (b.id, b.kind, b.flags, b.motion, &b.descriptor));
            for lane in 0..3 {
                assert!((a.position[lane] - b.position[lane]).abs() <= 1. / 256., "{:?} vs {:?}", a.position, b.position);
                assert!((a.velocity[lane] - b.velocity[lane]).abs() <= 1. / 64.);
            }
            let turn = (a.yaw - b.yaw).rem_euclid(std::f32::consts::TAU);
            assert!(turn < 0.001 || turn > std::f32::consts::TAU - 0.001, "{} vs {}", a.yaw, b.yaw);
        }
    }

    #[test]
    fn the_budget_cuts_entities_and_never_the_rest() {
        let run = run();
        let (bytes, included) = encode(&run, 40);
        assert_eq!(included, 0);
        let back = decode(&bytes).expect("decodes");
        assert_eq!(back.despawns, run.despawns);
        assert_eq!(back.contacts.len(), 1);
        assert!(back.entities.is_empty());
        let (bytes, included) = encode(&run, 80);
        assert_eq!(included, 1, "{} bytes", bytes.len());
    }

    #[test]
    fn a_bad_run_is_refused() {
        let (bytes, _) = encode(&run(), 1024);
        assert!(decode(&bytes[..bytes.len() - 1]).is_err());
        let mut other = bytes.clone();
        other[0] = FORMAT + 1;
        assert!(decode(&other).is_err());
        let mut trailing = bytes;
        trailing.push(0);
        assert!(decode(&trailing).is_err());
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn ids_and_transforms_round_trip() {
        let id = world_id(3, 0x0200_0042);
        assert_eq!((owner_slot(id), local_id(id)), (3, 0x0200_0042));
        for yaw in [0., 1., -2.5, 3.1] {
            let t = transform([1., 2., 3.], yaw);
            let turn = (yaw_of(&t) - yaw).rem_euclid(std::f32::consts::TAU);
            assert!(turn < 1e-5 || turn > std::f32::consts::TAU - 1e-5);
            assert_eq!(t[3], [1., 2., 3., 1.]);
        }
    }

    #[test]
    fn one_pedestrian_costs_this_much() {
        let run = run();
        let (with, _) = encode(&WorldRun { entities: vec![run.entities[0].clone()], ..Default::default() }, 1024);
        let (without, _) = encode(&WorldRun { entities: vec![run.entities[1].clone()], ..Default::default() }, 1024);
        // The budget arithmetic in docs/coop/bench.md rests on these.
        assert!(with.len() <= 56, "{} bytes with a descriptor", with.len());
        assert!(without.len() <= 28, "{} bytes without", without.len());
    }
}
