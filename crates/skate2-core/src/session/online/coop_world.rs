//! Co-op fork: the campaign's shared living world on the session (route D, `docs/coop/design.md`
//! §3.6). Each tick of a campaign this console:
//!
//! - sends a world run every few ticks: the pedestrians it owns near its skater (every one its
//!   own census spawned; M1's rule, `coop::ownership`), the ones it stopped owning, and the
//!   contacts its skater made with the others' puppets;
//! - applies the runs it received: a puppet pedestrian per streamed entity (spawned through the
//!   census's own spawn, driven by `PuppetDrive`), its owner's knockdowns played here, puppets
//!   culled when their owner stops streaming them;
//! - applies the contacts the others reported on its own pedestrians, through the same funnel a
//!   local hit takes (`record_accepted_full_collision`), so the hit becomes real on the owner
//!   and streams back to everyone as the owner's state.
//!
//! Every decision is traced under `SKATE2_TRACE_COOP`; the stream's cost is measured and
//! traced every ten seconds.

use super::{SelectedLiveError, SelectedLiveSession, error};
use crate::coop::ownership::{self, Kind, Owner};
use crate::coop::trace;
use crate::coop::world_stream::{self as stream, Contact, Descriptor, EntityState, PropState, WorldRun, flags};
use crate::living_world_grind_services::dmo_models::NULL_ASSET_ID;
use crate::pedestrian_graph::colliding::{CollisionDirection, CollisionForce};
use crate::pedestrian_puppet::PuppetDrive;
use skate2_net::frames::INTERVAL;
use skate2_net::protocol::MAX_WORLD;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// How far from the local skater an owned pedestrian is streamed (metres).
const STREAM_RADIUS: f32 = 80.;
/// A descriptor goes with an entity's first run and again every this many of its runs.
const DESCRIBE_EVERY: u32 = 20;
/// A puppet whose owner has not streamed it for this many ticks is culled.
const STALE_TICKS: u32 = 180;
/// A despawn is announced in this many runs, as runs are not acknowledged.
const DESPAWN_RUNS: u8 = 3;
/// A spawn request that found no model or no room is dropped after this many ticks.
const SPAWN_PATIENCE: u32 = 1200;
/// The measurement trace's period in ticks.
const MEASURE_TICKS: u32 = 600;

#[derive(Debug)]
struct Puppet {
    /// The local entity id of the puppet pedestrian.
    entity: u32,
    owner: u8,
    /// The local tick its last sample arrived.
    updated: u32,
    /// The local tick a contact was sent to the owner and not yet confirmed by its flags.
    awaiting: Option<u32>,
    knocked_down: bool,
}

#[derive(Debug)]
struct Spawn {
    owner: u8,
    descriptor: Descriptor,
    state: EntityState,
    since: u32,
}

#[derive(Debug, Default)]
struct Stats {
    runs: u32,
    bytes: u64,
    entities: u64,
    props: u64,
    contacts_sent: u32,
    contacts_applied: u32,
    confirmed: u32,
    since: u32,
}

/// A prop (a dynamic object with a locator) the ownership table knows: who owns it by the
/// last touch, with what claim, and this console's entity for it.
#[derive(Debug)]
struct PropOwner {
    owner: Owner,
    claim: u32,
    /// The tick this console's skater last touched it, while it is this console's.
    touched: u32,
    /// The last sampled position, so a resting prop is not sent every run.
    sent: Option<[f32; 3]>,
    runs: u32,
}

/// Props at rest are repeated every this many runs, so a receiver that missed a run or
/// joined late still gets them.
const RESTING_REPEAT: u32 = 20;
/// A prop is sampled this long after its last touch even at rest.
const TOUCH_SAMPLES: u32 = 120;

/// The session's shared-world state, one per campaign.
#[derive(Debug, Default)]
pub(in super::super) struct CoopWorld {
    /// The props the ownership table knows, by locator id (M5).
    props: BTreeMap<u64, PropOwner>,
    /// The owners' prop samples received, applied at the next tick.
    prop_updates: Vec<PropState>,
    nudged: bool,
    puppets: BTreeMap<u32, Puppet>,
    spawns: BTreeMap<u32, Spawn>,
    /// The samples received for known puppets, applied at the next tick.
    updates: Vec<EntityState>,
    despawns: Vec<u32>,
    /// Contacts the others reported on this console's own entities.
    received_contacts: Vec<Contact>,
    /// This console's entities described at least once, and how many runs each has been in.
    announced: BTreeMap<u32, u32>,
    /// This console's entities gone from its roster, with the runs left to say so.
    gone: BTreeMap<u32, u8>,
    /// This console's own story characters (never streamed), each decided once.
    own: BTreeSet<u32>,
    bad_runs: u32,
    shoved: bool,
    stats: Stats,
}

/// `SKATE2_DIAGNOSTIC_COOP_SHOVE=<tick>`: at that tick the nearest puppet is knocked down here
/// as if the local skater had hit it, and the contact goes to its owner, so the bench proves
/// the round trip without steering a skater into a pedestrian.
fn diagnostic_shove_tick() -> Option<u32> {
    static TICK: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *TICK.get_or_init(|| std::env::var("SKATE2_DIAGNOSTIC_COOP_SHOVE").ok().and_then(|tick| tick.parse().ok()))
}

/// `SKATE2_DIAGNOSTIC_COOP_NUDGE=<tick>`: at that tick the nearest prop with a locator within
/// 30 m is given a velocity here as a push would, and counts as touched by this console's
/// skater, so the bench proves a prop's claim and its stream without steering a skater into it.
fn diagnostic_nudge_tick() -> Option<u32> {
    static TICK: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *TICK.get_or_init(|| std::env::var("SKATE2_DIAGNOSTIC_COOP_NUDGE").ok().and_then(|tick| tick.parse().ok()))
}

/// `SKATE2_DIAGNOSTIC_COOP_TELEPORT=<tick>:<x>,<y>,<z>[;<tick>:<x>,<y>,<z>...]`: at each tick the
/// local skater is put there (the bench moves its seats into a populated street or through a
/// mission's volumes; the career's own diagnostics run only under the career demo).
fn diagnostic_teleports() -> &'static [(u32, [f32; 3])] {
    static SPEC: std::sync::OnceLock<Vec<(u32, [f32; 3])>> = std::sync::OnceLock::new();
    SPEC.get_or_init(|| {
        let Ok(spec) = std::env::var("SKATE2_DIAGNOSTIC_COOP_TELEPORT") else { return Vec::new() };
        spec.split(';')
            .filter_map(|entry| {
                let (at, position) = entry.split_once(':')?;
                let values: Vec<f32> = position.split(',').filter_map(|v| v.trim().parse().ok()).collect();
                let [x, y, z] = values[..] else { return None };
                Some((at.trim().parse().ok()?, [x, y, z]))
            })
            .collect()
    })
}

fn direction_of(flags: u8) -> CollisionDirection {
    match (flags & flags::DIRECTION_MASK) >> flags::DIRECTION_SHIFT {
        0 => CollisionDirection::Front,
        1 => CollisionDirection::Back,
        2 => CollisionDirection::Left,
        _ => CollisionDirection::Right,
    }
}

fn contact_direction(direction: u8) -> CollisionDirection {
    direction_of(direction << flags::DIRECTION_SHIFT)
}

fn distance_squared(a: [f32; 4], b: [f32; 4]) -> f32 {
    (0..3).map(|lane| (a[lane] - b[lane]) * (a[lane] - b[lane])).sum()
}

impl<T, C> SelectedLiveSession<T, C> {
    /// Another member's world run, from the connection's events: sorted into samples for known
    /// puppets, spawns for new entities, despawns and the contacts on this console's own.
    pub(in super::super) fn online_world_event(&mut self, user_id: u64, tick: u32, data: &[u8]) {
        let Some(game) = self.online.game.as_ref().filter(|game| game.entered) else { return };
        if !self.online_campaign() {
            return;
        }
        let Some(owner) = game.players.iter().find(|player| player.user_id == user_id).map(|player| player.slot) else { return };
        let local_slot = game.local_slot;
        let run = match stream::decode(data) {
            Ok(run) => run,
            Err(reason) => {
                let coop = &mut self.online.coop;
                coop.bad_runs += 1;
                if coop.bad_runs == 1 {
                    trace(format_args!("world run from slot {owner} at {tick} refused: {reason}"));
                }
                return;
            }
        };
        let coop = &mut self.online.coop;
        for entity in run.entities {
            if stream::owner_slot(entity.id) != owner {
                continue;
            }
            if coop.puppets.contains_key(&entity.id) {
                coop.updates.push(entity);
            } else if let Some(descriptor) = entity.descriptor.clone() {
                let since = self.completed_frames;
                coop.spawns.entry(entity.id).and_modify(|spawn| spawn.state = entity.clone()).or_insert(Spawn { owner, descriptor, state: entity, since });
            }
        }
        for prop in run.props {
            let known = coop.props.get(&prop.locator).map(|known| (known.claim, match known.owner { Owner::Local => local_slot, Owner::Remote(slot) => slot }));
            if !ownership::claim_wins(known, prop.claim, owner) {
                continue;
            }
            let old = coop.props.get(&prop.locator).map(|known| known.owner);
            if old != Some(Owner::Remote(owner)) {
                ownership::decide(Kind::Prop, prop.locator as u32, old, Owner::Remote(owner), &format!("its skater touched it last (claim {})", prop.claim));
            }
            coop.props.insert(prop.locator, PropOwner { owner: Owner::Remote(owner), claim: prop.claim, touched: 0, sent: None, runs: 0 });
            coop.prop_updates.push(prop);
        }
        coop.despawns.extend(run.despawns.into_iter().filter(|&id| stream::owner_slot(id) == owner));
        coop.received_contacts.extend(run.contacts.into_iter().filter(|contact| stream::owner_slot(contact.id) == local_slot));
    }

    /// One completed tick of a campaign: the received runs applied, the contacts applied, and
    /// this console's own run sent at the stream's cadence.
    pub(in super::super) fn coop_world_tick(&mut self) -> Result<(), SelectedLiveError> {
        let Some(game) = self.online.game.as_ref().filter(|game| game.entered) else { return Ok(()) };
        if !self.online_campaign() || self.map_travel.is_some() {
            return Ok(());
        }
        let local_slot = game.local_slot;
        let tick = self.completed_frames;
        for &(at, position) in diagnostic_teleports() {
            if at == tick {
                trace(format_args!("diagnostic: teleport to {position:?} at {tick}: {}", self.diagnostic_teleport_local_player(position)));
            }
        }
        let mut coop = std::mem::take(&mut self.online.coop);
        let sent = self
            .inner
            .try_with_selected_queued_frame(|mut frame, ground, _| -> Result<Option<(Vec<u8>, usize)>, String> {
                let Some(recipes) = frame.living_world_recipes.as_mut() else { return Ok(None) };
                let cvars = frame.cvars;
                let world = &mut *frame.world;
                let living_owner = Arc::clone(world.grind_services.living_world());
                let (centre, _) = crate::simulation_assembly_backend::constraint_session::dmo_living_world::local_skater_census_centre(frame.actor)
                    .map_err(|e| format!("coop world: skater centre: {e:?}"))?;

                // 1. The owners' samples drive the puppets; an owner's knockdown plays here.
                {
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let roster = living.pedestrian_entities.as_deref_mut().ok_or("pedestrian EntityMan absent")?;
                    for sample in coop.updates.drain(..) {
                        let Some(puppet) = coop.puppets.get_mut(&sample.id) else { continue };
                        let Some(pedestrian) = roster.get_mut(puppet.entity) else {
                            trace(format_args!("puppet {:08x}: its entity {:#x} is gone from the roster", sample.id, puppet.entity));
                            coop.puppets.remove(&sample.id);
                            continue;
                        };
                        let Some(drive) = pedestrian.puppet.as_mut() else { continue };
                        drive.arrive(stream::transform(sample.position, sample.yaw), sample.velocity, sample.flags);
                        puppet.updated = tick;
                        let knocked = sample.flags & flags::KNOCKED_DOWN != 0;
                        if knocked && !puppet.knocked_down {
                            puppet.knocked_down = true;
                            if puppet.awaiting.take().is_some() {
                                coop.stats.confirmed += 1;
                                trace(format_args!("puppet {:08x}: the owner confirmed the knockdown at {tick}", sample.id));
                            } else if !pedestrian.collision.as_ref().is_some_and(|collision| collision.colliding) {
                                let direction = direction_of(sample.flags);
                                pedestrian.record_accepted_full_collision(CollisionForce::Knockdown, direction, cvars)?;
                                trace(format_args!("puppet {:08x}: knocked down by its owner ({direction:?}) at {tick}", sample.id));
                            }
                        } else if !knocked {
                            if puppet.knocked_down {
                                trace(format_args!("puppet {:08x}: up again at {tick}", sample.id));
                            }
                            puppet.knocked_down = false;
                        }
                    }

                    // 2. The others' contacts on this console's own pedestrians, through the
                    // same funnel a local hit takes.
                    for contact in coop.received_contacts.drain(..) {
                        let entity = stream::local_id(contact.id);
                        let Some(pedestrian) = roster.get_mut(entity) else {
                            trace(format_args!("contact on {:08x}: no such entity here", contact.id));
                            continue;
                        };
                        let force = if contact.force == 0 { CollisionForce::Knockdown } else { CollisionForce::Standing };
                        let direction = contact_direction(contact.direction);
                        pedestrian.record_accepted_full_collision(force, direction, cvars)?;
                        coop.stats.contacts_applied += 1;
                        trace(format_args!("contact on {:08x} from another skater: {force:?} {direction:?} at {:.1} m/s applied at {tick}",
                            contact.id, (contact.velocity[0].powi(2) + contact.velocity[1].powi(2) + contact.velocity[2].powi(2)).sqrt()));
                    }
                }

                // 2b. The owners' prop samples: the local object of each locator becomes a
                // puppet driven to the sample (M5); this console's skater's touches claim.
                {
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let objects = living.dmo_objects.as_deref_mut().ok_or("DMO roster absent")?;
                    let ids = objects.ids();
                    let by_locator: BTreeMap<u64, u32> = ids
                        .iter()
                        .filter_map(|&id| objects.object(id).filter(|object| object.locator_id != NULL_ASSET_ID && !object.is_static()).map(|object| (object.locator_id, id)))
                        .collect();
                    for sample in coop.prop_updates.drain(..) {
                        let Some(&id) = by_locator.get(&sample.locator) else { continue };
                        let Some(object) = objects.object_mut(id) else { continue };
                        if !object.coop_puppet {
                            object.coop_puppet = true;
                            trace(format_args!("prop {:016x} (entity {id:#x}): a puppet here from {tick}, driven to its owner's samples", sample.locator));
                        }
                        let owner = coop.props.get_mut(&sample.locator).expect("a sample of a known prop");
                        owner.runs += 1;
                        if owner.runs % 60 == 1 {
                            let here = [object.transform[12], object.transform[13], object.transform[14]];
                            let gap = (0..3).map(|lane| (here[lane] - sample.position[lane]).powi(2)).sum::<f32>().sqrt();
                            trace(format_args!("prop {:016x}: sample {} at {:?}, the puppet at {here:?}, {gap:.2} m behind, at {tick}", sample.locator, owner.runs, sample.position));
                        }
                        object.target_transform = stream::transform_of(sample.position, sample.rotation);
                        object.velocity = [sample.velocity[0], sample.velocity[1], sample.velocity[2], 0.];
                        object.touched = false;
                    }
                    if let Some(at) = diagnostic_nudge_tick()
                        && tick >= at
                        && !coop.nudged
                    {
                        let nearest = by_locator
                            .values()
                            .filter_map(|&id| {
                                let object = objects.object(id)?;
                                let position = [object.transform[12], object.transform[13], object.transform[14], 1.];
                                Some((distance_squared(position, centre), id))
                            })
                            .filter(|(d2, _)| *d2 <= 30. * 30.)
                            .min_by(|a, b| a.0.total_cmp(&b.0));
                        if let Some((d2, id)) = nearest {
                            coop.nudged = true;
                            let object = objects.object_mut(id).expect("listed");
                            object.coop_puppet = false;
                            object.touched = true;
                            world.try_set_dmo_part_velocity(&object.body, 0, [0., 3., 4.])?;
                            trace(format_args!("diagnostic: nudged prop {:016x} (entity {id:#x}) at {:.1} m at {tick}: a touch by this console's skater", object.locator_id, d2.sqrt()));
                        } else if tick == at {
                            trace(format_args!("diagnostic: no prop within 30 m to nudge at {tick}"));
                        }
                    }
                    for (&locator, &id) in &by_locator {
                        let Some(object) = objects.object_mut(id) else { continue };
                        if !object.touched {
                            continue;
                        }
                        object.touched = false;
                        let known = coop.props.get(&locator);
                        if known.is_some_and(|known| known.owner == Owner::Local) {
                            coop.props.get_mut(&locator).expect("known").touched = tick;
                            continue;
                        }
                        let claim = known.map_or(1, |known| known.claim + 1);
                        ownership::decide(Kind::Prop, locator as u32, known.map(|known| known.owner), ownership::owner(Kind::Prop, Owner::Local), &format!("this console's skater touched it last (claim {claim})"));
                        object.coop_puppet = false;
                        coop.props.insert(locator, PropOwner { owner: Owner::Local, claim, touched: tick, sent: None, runs: 0 });
                    }
                }

                // 3. New entities of the others spawn here as puppets, through the census's spawn.
                let pending: Vec<u32> = coop.spawns.keys().copied().collect();
                for id in pending {
                    let spawn = coop.spawns.get_mut(&id).expect("listed");
                    if tick.saturating_sub(spawn.since) > SPAWN_PATIENCE {
                        trace(format_args!("puppet {id:08x} of slot {}: no spawn in {SPAWN_PATIENCE} ticks, dropped", spawn.owner));
                        coop.spawns.remove(&id);
                        continue;
                    }
                    let loaded = recipes.load_living_world_model(world, cvars, spawn.descriptor.model_key).map_err(|e| format!("coop world: model load: {e:?}"))?;
                    if !loaded {
                        continue;
                    }
                    let transform = stream::transform(spawn.state.position, spawn.state.yaw);
                    let look = Some((spawn.descriptor.model_key, spawn.descriptor.recipe_id));
                    let spawned = recipes.spawn_census_pedestrian_as(world, cvars, ground, spawn.descriptor.entity_key, look, transform, false, false, true, true, false, false)?;
                    let Some(entity) = spawned else { continue };
                    let owner = spawn.owner;
                    {
                        let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                        let pedestrian = living.pedestrian_entities.as_deref_mut().and_then(|p| p.get_mut(entity)).ok_or("spawned puppet absent")?;
                        let mut drive = PuppetDrive::new(transform);
                        drive.arrive(transform, spawn.state.velocity, spawn.state.flags);
                        pedestrian.puppet = Some(drive);
                        if let Some(core) = pedestrian.core.as_mut() {
                            core.colour_indices = spawn.descriptor.colours;
                            core.allow_culling = false;
                        }
                    }
                    ownership::decide(Kind::Pedestrian, id, None, Owner::Remote(owner), "its census spawned it");
                    trace(format_args!("puppet {id:08x} of slot {owner} spawned as entity {entity:#x} (entity key {:016x}, model {:016x}) at {tick}",
                        spawn.descriptor.entity_key, spawn.descriptor.model_key));
                    coop.puppets.insert(id, Puppet { entity, owner, updated: tick, awaiting: None, knocked_down: spawn.state.flags & flags::KNOCKED_DOWN != 0 });
                    coop.spawns.remove(&id);
                }

                // 4. Puppets their owner despawned or stopped streaming go.
                let mut culls: Vec<(u32, &'static str)> = coop.despawns.drain(..).map(|id| (id, "its owner despawned it")).collect();
                for (&id, puppet) in &coop.puppets {
                    if tick.saturating_sub(puppet.updated) > STALE_TICKS {
                        culls.push((id, "its owner stopped streaming it"));
                    }
                }
                for (id, reason) in culls {
                    let Some(puppet) = coop.puppets.remove(&id) else { continue };
                    recipes.cull_census_pedestrian(world, cvars, puppet.entity)?;
                    trace(format_args!("puppet {id:08x} of slot {}: culled at {tick}, {reason}", puppet.owner));
                }

                // 5. The diagnostic shove: the nearest puppet, hit as a skater would hit it.
                if let Some(at) = diagnostic_shove_tick()
                    && tick >= at
                    && !coop.shoved
                {
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let roster = living.pedestrian_entities.as_deref_mut().ok_or("pedestrian EntityMan absent")?;
                    let nearest = coop
                        .puppets
                        .iter()
                        .filter_map(|(&id, puppet)| {
                            let core = roster.get(puppet.entity)?.core.as_ref()?;
                            Some((distance_squared(core.transform[3], centre), id, puppet.entity))
                        })
                        .min_by(|a, b| a.0.total_cmp(&b.0));
                    if let Some((d2, id, entity)) = nearest {
                        coop.shoved = true;
                        let pedestrian = roster.get_mut(entity).expect("listed");
                        pedestrian.record_accepted_full_collision(CollisionForce::Knockdown, CollisionDirection::Front, cvars)?;
                        if let Some(drive) = pedestrian.puppet.as_mut() {
                            drive.pending_contact = Some((CollisionForce::Knockdown as u32 as u8, CollisionDirection::Front as u32 as u8, [0., 0., 6.], [0., 0., 0.]));
                        }
                        trace(format_args!("diagnostic: shoved puppet {id:08x} at {:.1} m: knockdown predicted here at {tick}, contact to its owner", d2.sqrt()));
                    } else if tick == at {
                        trace(format_args!("diagnostic: no puppet to shove at {tick}"));
                    }
                }

                // 6. This console's own run, at the stream's cadence.
                if tick % INTERVAL != 0 {
                    return Ok(None);
                }
                let mut run = WorldRun::default();
                let mut owned = Vec::new();
                let mut guards = BTreeSet::new();
                let roster_size;
                {
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let roster = living.pedestrian_entities.as_deref_mut().ok_or("pedestrian EntityMan absent")?;
                    let present: BTreeSet<u32> = roster.ids().into_iter().collect();
                    roster_size = present.len();
                    for &id in &present {
                        let pedestrian = roster.get(id).expect("listed");
                        if pedestrian.puppet.is_some() {
                            continue;
                        }
                        // A pedestrian a challenge placed is every console's own (M5).
                        if pedestrian.challenge_owner.is_some() {
                            if coop.own.insert(id) {
                                ownership::decide(Kind::StoryCharacter, id, None, ownership::owner(Kind::StoryCharacter, Owner::Local), "a challenge placed it: every console's own, not streamed");
                            }
                            continue;
                        }
                        let Some(core) = pedestrian.core.as_ref() else { continue };
                        if pedestrian.security_guard.is_some() {
                            guards.insert(id);
                        }
                        let d2 = distance_squared(core.transform[3], centre);
                        if d2 > STREAM_RADIUS * STREAM_RADIUS {
                            continue;
                        }
                        let mut entity_flags = 0;
                        if pedestrian.state.as_ref().is_some_and(|state| state.flags[1] & 4 != 0) {
                            entity_flags |= flags::KNOCKED_DOWN;
                        }
                        if pedestrian.collision.as_ref().is_some_and(|collision| collision.colliding) {
                            entity_flags |= flags::COLLIDING;
                            if pedestrian.collision_info.force == CollisionForce::Standing {
                                entity_flags |= flags::STANDING_HIT;
                            }
                        }
                        entity_flags |= ((pedestrian.collision_info.direction as u32 as u8) << flags::DIRECTION_SHIFT) & flags::DIRECTION_MASK;
                        let runs = coop.announced.get(&id).copied();
                        let describe = runs.is_none_or(|runs| runs % DESCRIBE_EVERY == 0);
                        owned.push((d2, EntityState {
                            id: stream::world_id(local_slot, id),
                            kind: Kind::Pedestrian,
                            position: [core.transform[3][0], core.transform[3][1], core.transform[3][2]],
                            yaw: stream::yaw_of(&core.transform),
                            velocity: [core.velocity[0], core.velocity[1], core.velocity[2]],
                            flags: entity_flags,
                            motion: pedestrian.state_out.state.min(255) as u8,
                            descriptor: describe.then(|| Descriptor {
                                entity_key: core.entity_key,
                                model_key: core.model_key,
                                recipe_id: core.recipe_id,
                                colours: core.colour_indices,
                            }),
                        }));
                    }
                    // The contacts this skater made with the others' puppets.
                    for (&id, puppet) in &mut coop.puppets {
                        let Some(contact) = roster.get_mut(puppet.entity).and_then(|p| p.puppet.as_mut()).and_then(|drive| drive.pending_contact.take()) else { continue };
                        let (force, direction, velocity, omega) = contact;
                        run.contacts.push(Contact { id, force, direction, velocity, omega });
                        puppet.awaiting = Some(tick);
                        coop.stats.contacts_sent += 1;
                        trace(format_args!("contact on puppet {id:08x} of slot {}: force {force} direction {direction}, reported to its owner at {tick}", puppet.owner));
                    }
                    // Entities announced before and gone from the roster are despawned to the others.
                    coop.own.retain(|id| present.contains(id));
                    let gone: Vec<u32> = coop.announced.keys().filter(|id| !present.contains(id)).copied().collect();
                    for id in gone {
                        coop.announced.remove(&id);
                        coop.gone.insert(id, DESPAWN_RUNS);
                        trace(format_args!("own pedestrian {:08x} left the roster: despawn announced", stream::world_id(local_slot, id)));
                    }
                }
                owned.sort_by(|a, b| a.0.total_cmp(&b.0));
                run.entities = owned.into_iter().map(|(_, entity)| entity).collect();
                // 6b. This console's props: every one it owns, sampled while it moves, for a
                // while after a touch, and now and then at rest.
                {
                    let living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let objects = living.dmo_objects.as_deref().ok_or("DMO roster absent")?;
                    let ids = objects.ids();
                    for (&locator, owner) in coop.props.iter_mut() {
                        if owner.owner != Owner::Local || run.props.len() >= stream::MOST_PROPS {
                            continue;
                        }
                        let Some(object) = ids.iter().find_map(|&id| objects.object(id).filter(|object| object.locator_id == locator)) else { continue };
                        let position = [object.transform[12], object.transform[13], object.transform[14]];
                        let velocity = [object.velocity[0], object.velocity[1], object.velocity[2]];
                        let speed2 = velocity.iter().map(|v| v * v).sum::<f32>();
                        let moved = owner.sent.is_none_or(|sent| (0..3).map(|lane| (position[lane] - sent[lane]).powi(2)).sum::<f32>() > 1e-4);
                        owner.runs += 1;
                        if !(moved || speed2 > 1e-4 || tick.saturating_sub(owner.touched) <= TOUCH_SAMPLES || owner.runs % RESTING_REPEAT == 0) {
                            continue;
                        }
                        owner.sent = Some(position);
                        run.props.push(PropState { locator, claim: owner.claim, position, rotation: stream::quaternion_of(&object.transform), velocity, resting: speed2 <= 1e-4 });
                    }
                }
                run.despawns = coop.gone.keys().map(|&id| stream::world_id(local_slot, id)).collect();
                coop.gone.retain(|_, left| {
                    *left -= 1;
                    *left > 0
                });
                let (bytes, included) = stream::encode(&run, MAX_WORLD);
                for entity in &run.entities[..included] {
                    let id = stream::local_id(entity.id);
                    let runs = coop.announced.entry(id).or_insert_with(|| {
                        if guards.contains(&id) {
                            ownership::decide(Kind::SecurityGuard, entity.id, None, ownership::owner(Kind::SecurityGuard, Owner::Local), "this census spawned it; it chases this console's skater only");
                        } else {
                            ownership::decide(Kind::Pedestrian, entity.id, None, Owner::Local, "this census spawned it");
                        }
                        0
                    });
                    *runs += 1;
                }
                if included < run.entities.len() {
                    trace(format_args!("world run at {tick}: {} of {} owned pedestrians fit {MAX_WORLD} bytes", included, run.entities.len()));
                }
                let stats = &mut coop.stats;
                stats.runs += 1;
                stats.bytes += bytes.len() as u64;
                stats.entities += included as u64;
                stats.props += run.props.len() as u64;
                if tick.saturating_sub(stats.since) >= MEASURE_TICKS {
                    let seconds = f64::from(tick.saturating_sub(stats.since)) / 60.;
                    let per_entity = if stats.entities == 0 { 0. } else { stats.bytes as f64 / seconds / (stats.entities as f64 / f64::from(stats.runs.max(1))) };
                    trace(format_args!(
                        "world stream at {tick}: {} runs, {} entity samples, {} prop samples, {} B ({:.0} B/s, {:.0} B/s per streamed pedestrian), {} owned in range of {} pedestrians here, {} puppets, {} props owned of {} known, contacts sent {} applied {} confirmed {}",
                        stats.runs, stats.entities, stats.props, stats.bytes, stats.bytes as f64 / seconds, per_entity, run.entities.len(), roster_size, coop.puppets.len(),
                        coop.props.values().filter(|prop| prop.owner == Owner::Local).count(), coop.props.len(),
                        stats.contacts_sent, stats.contacts_applied, stats.confirmed
                    ));
                    *stats = Stats { since: tick, ..Default::default() };
                }
                Ok(Some((bytes, included)))
            })
            .map_err(|e| error(format!("{e:?}")))?
            .map_err(error)?;
        self.online.coop = coop;
        if let Some((bytes, _)) = sent
            && let Some(client) = self.online.client.as_ref()
        {
            client.send_world(tick, bytes);
        }
        Ok(())
    }
}
