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
//! M6: every entity has one world id for life (its first spawner's slot and local id) and a
//! table entry here naming its owner and epoch; an owned pedestrian a peer is nearer to by a
//! margin for a second is handed over (`HANDOFF`): this console's copy becomes a puppet, the
//! record names the new owner with the epoch raised, and the new owner promotes its puppet
//! (its graph and navigation take over where the puppet stood).
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
/// M17: an owned pedestrian's sample goes out when the receiver's extrapolation of the last
/// one (its position carried on at its velocity) would be off by more than this, or its facing
/// by more than `SEND_YAW`, or its flags, motion or graph state changed; else only every
/// `KEEPALIVE_RUNS` runs.
const SEND_POSITION: f32 = 0.05;
const SEND_YAW: f32 = 2.0_f32 * std::f32::consts::PI / 180.;
const KEEPALIVE_RUNS: u32 = 20;
/// A puppet whose owner has not streamed it for this many ticks is culled.
const STALE_TICKS: u32 = 180;
/// A despawn is announced in this many runs, as runs are not acknowledged.
const DESPAWN_RUNS: u8 = 3;
/// A spawn request that found no model or no room is dropped after this many ticks.
const SPAWN_PATIENCE: u32 = 1200;
/// The measurement trace's period in ticks.
const MEASURE_TICKS: u32 = 600;
/// A peer must be nearer an owned pedestrian than this console by this many metres...
const HANDOFF_MARGIN: f32 = 15.;
/// ...for this many ticks before the pedestrian is handed over.
const HANDOFF_TICKS: u32 = 60;

/// An entity the table knows, by its world id: this console's copy, who owns it, and its epoch.
#[derive(Debug)]
struct Tracked {
    /// The local entity id of this console's copy (a puppet unless the owner is this console).
    entity: u32,
    owner: Owner,
    epoch: u32,
    kind: Kind,
    /// The local tick its last sample arrived (a puppet's).
    updated: u32,
    /// The local tick a contact was sent to the owner and not yet confirmed by its flags.
    awaiting: Option<u32>,
    knocked_down: bool,
    /// M20: the owner's knockdown, not yet played on this puppet because the puppet was still
    /// in a collision of its own (its owner's standing hit, played a sample late): played on
    /// the first tick the puppet is free, its standing hit ended for it.
    knockdown_due: Option<CollisionDirection>,
    /// How many runs this console has streamed it in (an owned one's).
    runs: u32,
    /// How many ticks a peer has been nearer by the margin (an owned one's), and which.
    nearer: (u32, u8),
    /// How many samples in a row this puppet has been beyond the position tolerance.
    behind: u32,
    /// M17: the last sample sent of an owned pedestrian, and the tick it went out, for the
    /// send-on-error rule.
    sent: Option<Sent>,
}

/// M17: what the last sample sent said, and when.
#[derive(Clone, Copy, Debug)]
struct Sent {
    tick: u32,
    position: [f32; 3],
    yaw: f32,
    velocity: [f32; 3],
    flags: u8,
    motion: u8,
    graph: u16,
    /// M20: the hash of the intents last sent, and how many more samples name them.
    intents: u32,
    names_left: u8,
    runs: u32,
    /// M19: the hand prop last described, so a change sends a descriptor at once.
    hand_prop: u64,
}

/// M20: an owner's tick against this console's: the least difference seen this window and the
/// last (a 300-tick window), so a sample's lateness is its difference past that.
#[derive(Debug, Clone, Copy)]
struct Clock {
    least: i64,
    least_before: i64,
    window: u32,
}

impl Clock {
    /// A second: the least is over this window and the last, so a step in the owner's tick
    /// against this console's (a hitch its loop never caught up) reads as lateness for at most
    /// two seconds, while a second holds many punctual runs.
    const WINDOW: u32 = 60;

    fn new(difference: i64, local: u32) -> Self {
        Self { least: difference, least_before: difference, window: local / Self::WINDOW }
    }

    /// A run's difference seen at a local tick: the lateness of its samples.
    fn late(&mut self, difference: i64, local: u32) -> u32 {
        let window = local / Self::WINDOW;
        if window != self.window {
            self.least_before = if window == self.window + 1 { self.least } else { difference };
            self.least = difference;
            self.window = window;
        }
        self.least = self.least.min(difference);
        let punctual = self.least.min(self.least_before);
        u32::try_from(difference - punctual).unwrap_or(0)
    }
}

/// M20: the velocity a pedestrian moves at, from its last positions; `None` for too short a
/// trail or one that jumped (a respawn, a teleport).
fn measured_velocity(trail: &std::collections::VecDeque<[f32; 3]>) -> Option<[f32; 3]> {
    if trail.len() < 4 {
        return None;
    }
    let (first, last) = (trail.front()?, trail.back()?);
    let seconds = (trail.len() - 1) as f32 / 60.;
    let velocity = [0, 1, 2].map(|lane| (last[lane] - first[lane]) / seconds);
    (velocity.iter().map(|value| value * value).sum::<f32>() < 100.).then_some(velocity)
}

/// M20: a stage's intent names as text, for the world stream.
fn intent_texts(stage: &[crate::play_animation::FastString30]) -> Vec<String> {
    stage.iter().map(|name| name.decode().unwrap_or_default()).collect()
}

/// M20: how many samples name a change of intents (a run lost leaves the others).
const NAME_SAMPLES: u8 = 3;

/// M20: the intent lists a console keeps by hash at most (a pedestrian's vocabulary is a few
/// dozen; this bounds a peer that sends nonsense).
const MOST_INTENT_LISTS: usize = 4096;

/// How many ticks of a pedestrian's positions its measured velocity spans (M20): a stride's
/// share, so the root motion's surge within a step averages out.
const TRAIL_TICKS: usize = 8;

#[derive(Debug)]
struct Spawn {
    owner: u8,
    descriptor: Descriptor,
    state: EntityState,
    since: u32,
    /// The record handed the entity to this console: promote the puppet once spawned.
    promote: bool,
}

/// The live audit (the bridge's receiving end): each sample measured against the puppet it
/// drives before it is applied, so divergence shows in play on any two machines, not only in
/// two logs on one. Tolerances as `tools/coop/sync_audit.py`'s: position 0.5 m, yaw 5°, speed
/// 0.5 m/s; flags exact.
#[derive(Debug, Default)]
struct LiveAudit {
    samples: u64,
    position_ok: u64,
    yaw_ok: u64,
    speed_ok: u64,
    flags_ok: u64,
    worst_position: (f32, u32),
    worst_yaw: (f32, u32),
}
const AUDIT_POSITION: f32 = 0.5;
const AUDIT_YAW_DEGREES: f32 = 5.;
const AUDIT_SPEED: f32 = 0.5;
/// A puppet this many samples in a row beyond the position tolerance is traced once.
const AUDIT_BEHIND_SAMPLES: u32 = 20;
/// A knockdown contact sent to the owner is awaited this many ticks before the owner's "up"
/// counts as its refusal (the stream's round trip is a few ticks; the owner's own knockdown
/// plays at once).
const CONFIRM_GRACE_TICKS: u32 = 60;

#[derive(Debug, Default)]
struct Stats {
    runs: u32,
    bytes: u64,
    entities: u64,
    props: u64,
    audit: LiveAudit,
    contacts_sent: u32,
    contacts_applied: u32,
    confirmed: u32,
    /// Predicted knockdowns ended by the owner's "up" before any confirmation.
    refused: u32,
    /// Records received and ignored for an epoch older than the table's.
    stale: u32,
    /// M17: samples the send-on-error rule withheld.
    withheld: u64,
    /// M20: intent lists named in samples sent; puppet samples whose intents were applied, and
    /// those whose hash had no names heard yet.
    intents_named: u64,
    intents_known: u64,
    intents_unknown: u64,
    /// M20: the lateness of the samples received (ticks), its sum, worst and count past 3.
    late_samples: u64,
    late_sum: u64,
    late_worst: u32,
    late_over: u64,
    /// M19's baseline: this console's own pedestrians' jerk, measured as the puppets' is.
    own_ticks: u64,
    own_jerk: f64,
    own_over: u64,
    own_over_flat: u64,
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
    /// While it is another console's: the tick its last sample arrived, whether that sample
    /// had it moving, and how many touches here have waited on it (M10.1).
    sampled: u32,
    moving: bool,
    waited: u32,
}

/// A touch here does not take a prop whose owner's latest sample, this recent, still has it
/// moving: two consoles' actors leaning on one prop would otherwise claim it back and forth
/// (the M9 S.K.A.T.E., each console's own pro: twelve claims of one rail in forty frames, a
/// puppet up to 4.7 m behind). The owner's samples come every third tick while a prop moves.
const CONTEST_TICKS: u32 = 30;

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
    /// The nudges done, an index into `diagnostic_nudges`.
    nudged: usize,
    /// Every entity the table knows, by world id (M6): this console's own and the puppets.
    entities: BTreeMap<u32, Tracked>,
    /// This console's local entity ids and their world ids.
    locals: BTreeMap<u32, u32>,
    spawns: BTreeMap<u32, Spawn>,
    /// The samples received for known puppets, applied at the next tick.
    /// The others' samples to apply, each with its lateness in ticks (M20).
    updates: Vec<(EntityState, u32)>,
    /// M20: each owner's clock against this console's (the least of `local tick - its tick`
    /// over this window and the last, so drift is followed), for a sample's lateness.
    clocks: BTreeMap<u8, Clock>,
    despawns: Vec<u32>,
    /// Puppets handed to this console, promoted at the next tick.
    promotions: Vec<u32>,
    /// This console's own entities another console claimed with a higher epoch, demoted at
    /// the next tick.
    demotions: Vec<u32>,
    /// Contacts the others reported on this console's own entities.
    received_contacts: Vec<Contact>,
    /// Entities handed over, with the runs left to say so (the record names the new owner).
    handoffs: BTreeMap<u32, u8>,
    /// This console's entities gone from its roster, with the runs left to say so.
    gone: BTreeMap<u32, u8>,
    /// The others' story characters refused here (this console's own copy of the entry stands,
    /// the lower slot's): their world ids, until their despawn.
    refused: BTreeSet<u32>,
    /// M19's baseline meter: this console's own pedestrians' last position and step.
    own_steps: BTreeMap<u32, ([f32; 3], [f32; 3])>,
    /// M20: this console's own pedestrians' last positions (`TRAIL_TICKS` of them): the velocity
    /// their samples carry is the one they move at, not their navigation's (a pedestrian stopped
    /// to talk or wait kept a navigation velocity of up to half a metre a second while it stood,
    /// and its puppet walked in place to it: the player, 2026-11-12, and the M19 census dumps).
    own_trail: BTreeMap<u32, std::collections::VecDeque<[f32; 3]>>,
    /// M20: every intent list heard from the others, by its hash.
    intent_names: BTreeMap<u32, Vec<crate::play_animation::FastString30>>,
    bad_runs: u32,
    shoved: bool,
    stats: Stats,
}

/// `SKATE2_TRACE_COOP_CENSUS=1`: every 60 round ticks, one line per pedestrian this console
/// knows (world id, owner and epoch, position, yaw, speed, flags, model, graph behaviour,
/// moods, down), the same ticks on every seat, so `tools/coop/sync_audit.py` can pair the
/// seats' lines by world id and say per field what agrees and what is each console's own
/// (`docs/coop/plan/sync-inventory.md`). Per-pedestrian detail, so under its own switch.
fn census_trace_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("SKATE2_TRACE_COOP_CENSUS").is_some())
}
/// The census dump's period in round ticks.
const CENSUS_EVERY: u32 = 60;

/// `SKATE2_DIAGNOSTIC_COOP_SHOVE=<tick>`: at that tick the nearest puppet is knocked down here
/// as if the local skater had hit it, and the contact goes to its owner, so the bench proves
/// the round trip without steering a skater into a pedestrian.
fn diagnostic_shove_tick() -> Option<u32> {
    static TICK: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *TICK.get_or_init(|| std::env::var("SKATE2_DIAGNOSTIC_COOP_SHOVE").ok().and_then(|tick| tick.parse().ok()))
}

/// `SKATE2_DIAGNOSTIC_COOP_NUDGE=<tick>[,<tick>...]`: at each tick the nearest prop with a
/// locator within 30 m is given a velocity here as a push would, and counts as touched by this
/// console's skater, so the bench proves a prop's claim and its stream without steering a
/// skater into it; `SKATE2_DIAGNOSTIC_COOP_NUDGE_ROUND` counts in round ticks instead, so two
/// seats' nudges can be timed against each other.
fn diagnostic_nudges() -> (&'static [u32], bool) {
    static NUDGES: std::sync::OnceLock<(Vec<u32>, bool)> = std::sync::OnceLock::new();
    let (ticks, round) = NUDGES.get_or_init(|| {
        let parse = |ticks: String| {
            let mut ticks = ticks.split(',').filter_map(|tick| tick.trim().parse().ok()).collect::<Vec<u32>>();
            ticks.sort_unstable();
            ticks
        };
        match std::env::var("SKATE2_DIAGNOSTIC_COOP_NUDGE_ROUND") {
            Ok(ticks) => (parse(ticks), true),
            Err(_) => (std::env::var("SKATE2_DIAGNOSTIC_COOP_NUDGE").ok().map_or_else(Vec::new, parse), false),
        }
    });
    (ticks, *round)
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

/// A pedestrian's state on the wire (`coop::world_stream::flags`): down, colliding, the hit's
/// force and direction.
/// M19: the hand prop a pedestrian holds or is about to (its collection key), 0 for none.
fn hand_prop_of(pedestrian: &crate::living_world_pedestrian_entity::PedestrianConstruction) -> u64 {
    let pending = pedestrian.state.as_ref().is_some_and(|state| state.flags[3] & 2 != 0);
    if pedestrian.held_hand_prop.is_none() && !pending {
        return 0;
    }
    pedestrian.hand_prop_attributes.as_ref().and_then(|attributes| attributes.collection_key()).unwrap_or(0)
}

/// M19: a puppet's prop thrown away on its owner's word (the puppet's own throws are held back).
fn release_as_owner(
    pedestrian: &mut crate::living_world_pedestrian_entity::PedestrianConstruction,
    cvars: &crate::input_cvar_runtime::InputCvarReferences,
) -> Result<(), String> {
    if let Some(drive) = pedestrian.puppet.as_mut() {
        drive.releasing = true;
    }
    let thrown = pedestrian.throw_hand_prop_away(cvars);
    if let Some(drive) = pedestrian.puppet.as_mut() {
        drive.releasing = false;
    }
    thrown
}

/// M19: whether a hand prop's model is resident here (the hand-prop pass's own lookup: the
/// `hand_props` row's model, its `dmo_models` row's asset). The pass clears a pending prop whose
/// model is not, so a puppet waits for it instead of setting it pending again at every
/// descriptor (the M19 probe's first run: up to 22 times on one puppet).
fn hand_prop_model_resident(
    models: &crate::living_world_grind_services::dmo_models::DmoModelMan,
    vault: &Arc<crate::attribute_runtime::AttributeRuntimeVault>,
    hand_prop: u64,
) -> bool {
    let word = |data: Option<_>| -> Option<u64> {
        let data: crate::attribute_runtime::AttributeRuntimeData = data?;
        Some(u64::from_be_bytes(data.bytes().try_into().ok()?))
    };
    let Some(model_key) = vault.instance(0xec8b_7565_46e2_26f6, hand_prop).try_cached_layout_field(40, 8).ok().and_then(word) else { return false };
    let Some(asset) = vault.instance(skate2_formats::attrib::string_hash64("dmo_models"), model_key).try_cached_layout_field(0, 8).ok().and_then(word) else { return false };
    models.model(asset).is_some()
}

/// M19: a puppet's hand prop made the owner's: a prop the owner holds and the puppet does not
/// is set pending (the hand-prop pass spawns and attaches it as it does the census's own) once
/// its model is resident here; one the owner no longer holds is thrown away here.
fn sync_hand_prop(
    pedestrian: &mut crate::living_world_pedestrian_entity::PedestrianConstruction,
    hand_prop: u64,
    vault: &Arc<crate::attribute_runtime::AttributeRuntimeVault>,
    models: &crate::living_world_grind_services::dmo_models::DmoModelMan,
    cvars: &crate::input_cvar_runtime::InputCvarReferences,
    id: u32,
) -> Result<(), String> {
    let here = hand_prop_of(pedestrian);
    if here == hand_prop {
        return Ok(());
    }
    if hand_prop == 0 {
        if pedestrian.held_hand_prop.is_some() {
            release_as_owner(pedestrian, cvars)?;
            trace(format_args!("puppet {id:08x}: its owner let its hand prop go: thrown away here"));
        } else if let Some(state) = pedestrian.state.as_mut() {
            state.flags[3] &= !2;
            pedestrian.hand_prop_attributes = None;
        }
        return Ok(());
    }
    if !hand_prop_model_resident(models, vault, hand_prop) {
        if let Some(drive) = pedestrian.puppet.as_mut().filter(|drive| drive.prop_waiting != hand_prop) {
            drive.prop_waiting = hand_prop;
            trace(format_args!("puppet {id:08x}: hand prop {hand_prop:016x} as its owner's, waiting for its model here"));
        }
        return Ok(());
    }
    if let Some(drive) = pedestrian.puppet.as_mut() {
        drive.prop_waiting = 0;
    }
    if here != 0 && pedestrian.held_hand_prop.is_some() {
        release_as_owner(pedestrian, cvars)?;
    }
    pedestrian.hand_prop_attributes = Some(vault.instance(0xec8b_7565_46e2_26f6, hand_prop));
    if let Some(state) = pedestrian.state.as_mut() {
        state.flags[3] |= 2;
    }
    trace(format_args!("puppet {id:08x}: hand prop {hand_prop:016x} as its owner's, pending here"));
    Ok(())
}

fn entity_flags(pedestrian: &crate::living_world_pedestrian_entity::PedestrianConstruction) -> u8 {
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
    entity_flags | (((pedestrian.collision_info.direction as u32 as u8) << flags::DIRECTION_SHIFT) & flags::DIRECTION_MASK)
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
        // M20: the run's lateness against its owner's punctual runs.
        let local = self.completed_frames;
        let difference = i64::from(local) - i64::from(tick);
        let late = match coop.clocks.get_mut(&owner) {
            Some(clock) => clock.late(difference, local),
            None => {
                coop.clocks.insert(owner, Clock::new(difference, local));
                0
            }
        };
        if !run.entities.is_empty() {
            coop.stats.late_samples += 1;
            coop.stats.late_sum += u64::from(late);
            coop.stats.late_worst = coop.stats.late_worst.max(late);
            coop.stats.late_over += u64::from(late > 3);
        }
        for entity in run.entities {
            let id = entity.id;
            // M20: the names behind an intent hash, kept for every record that follows.
            if let Some(names) = entity.intent_names.as_ref().filter(|_| entity.intents != 0)
                && !coop.intent_names.contains_key(&entity.intents)
                && coop.intent_names.len() < MOST_INTENT_LISTS
            {
                // The names must hash to the hash they came with: a list kept under the wrong
                // hash would play for the whole session.
                let names = names.iter().map(|name| crate::play_animation::FastString30::encode(name)).collect::<Result<Vec<_>, _>>();
                match names {
                    Ok(names) if !names.is_empty() && crate::pedestrian_puppet::intents_hash(&names) == entity.intents => {
                        coop.intent_names.insert(entity.intents, names);
                    }
                    Ok(names) => trace(format_args!("world run from {owner}: intent names {names:?} refused: they do not hash to {:08x}", entity.intents)),
                    Err(error) => trace(format_args!("world run from {owner}: intent names for {:08x} refused: {error:?}", entity.intents)),
                }
            }
            let known = coop.entities.get_mut(&id);
            let stale = known.as_ref().is_some_and(|known| entity.epoch < known.epoch || (entity.epoch == known.epoch && known.owner == Owner::Local));
            if stale {
                // Received and ignored: an older epoch than the table's (counted; the
                // measurement line says how many).
                coop.stats.stale += 1;
                continue;
            }
            if entity.owner == local_slot {
                // A handoff to this console.
                match known {
                    Some(known) => {
                        if known.owner != Owner::Local {
                            known.epoch = entity.epoch;
                            coop.promotions.push(id);
                        }
                    }
                    None => {
                        if let Some(descriptor) = entity.descriptor.clone() {
                            let since = self.completed_frames;
                            coop.spawns.entry(id).and_modify(|spawn| { spawn.state = entity.clone(); spawn.promote = true; }).or_insert(Spawn { owner, descriptor, state: entity, since, promote: true });
                        }
                    }
                }
                continue;
            }
            if coop.refused.contains(&id) {
                continue;
            }
            match known {
                Some(known) => {
                    if known.owner != Owner::Remote(entity.owner) {
                        let old = known.owner;
                        known.owner = ownership::decide(known.kind, id, Some(old), Owner::Remote(entity.owner), &format!("handed over (epoch {})", entity.epoch));
                        known.epoch = entity.epoch;
                        if old == Owner::Local {
                            // This console's copy stops simulating it: a puppet from the next tick.
                            coop.demotions.push(id);
                        }
                    }
                    coop.updates.push((entity, late));
                }
                None => {
                    if let Some(descriptor) = entity.descriptor.clone() {
                        let since = self.completed_frames;
                        coop.spawns.entry(id).and_modify(|spawn| spawn.state = entity.clone()).or_insert(Spawn { owner: entity.owner, descriptor, state: entity, since, promote: false });
                    }
                }
            }
        }
        for prop in run.props {
            let record = coop.props.get(&prop.locator);
            let known = record.map(|known| (known.claim, match known.owner { Owner::Local => local_slot, Owner::Remote(slot) => slot }));
            let same_owner = record.is_some_and(|known| known.owner == Owner::Remote(owner));
            // The owner's continuing stream carries the claim it holds: its samples are the
            // puppet's. (Until M10.1 only a higher claim passed, so a puppet prop took the
            // first sample of each claim and no other: the sample of the touch's own tick,
            // before the prop had moved, and the puppet stood where the original fell.)
            let same = same_owner && record.is_some_and(|known| known.claim == prop.claim);
            if !same && !ownership::claim_wins(known, prop.claim, owner) {
                continue;
            }
            let old = coop.props.get(&prop.locator).map(|known| known.owner);
            if old != Some(Owner::Remote(owner)) {
                ownership::decide(Kind::Prop, prop.locator as u32, old, Owner::Remote(owner), &format!("its skater touched it last (claim {})", prop.claim));
            }
            // The record keeps its sample count across samples (the convergence trace's
            // "every 60th" counted from 1 each sample before).
            let runs = if same { record.map_or(0, |known| known.runs) } else { 0 };
            // A sample that raises the claim is a fresh touch: it counts as moving until the
            // owner's next sample says otherwise (the sender says the same of a touch's own
            // tick, whose cached velocity is the step before's; this covers a sample from before
            // the record knew the claim).
            let moving = !prop.resting || !same;
            // A contest ends when the prop rests or the prop changes hands: the next one traces again.
            let waited = if moving && same_owner { record.map_or(0, |known| known.waited) } else { 0 };
            coop.props.insert(prop.locator, PropOwner { owner: Owner::Remote(owner), claim: prop.claim, touched: 0, sent: None, runs, sampled: self.completed_frames, moving, waited });
            coop.prop_updates.push(prop);
        }
        for id in &run.despawns {
            coop.refused.remove(id);
        }
        coop.despawns.extend(run.despawns.into_iter().filter(|id| coop.entities.get(id).is_some_and(|known| known.owner == Owner::Remote(owner))));
        coop.received_contacts.extend(run.contacts.into_iter().filter(|contact| coop.entities.get(&contact.id).is_some_and(|known| known.owner == Owner::Local)));
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
        // Every player's tester from the facts (the same instant on every console: the
        // processed tick's), for the handoff rule; the local one among them, so the rule
        // never compares a live position with a two-second-old one.
        let (peers, own_facts): (Vec<(u8, [f32; 4])>, Option<[f32; 4]>) = self
            .career
            .as_ref()
            .map(|career| {
                let frames = &career.system.world().skaters;
                let at = |frame: &crate::challenge::mission::MissionFrame| [frame.tester_base[0], frame.tester_base[1], frame.tester_base[2], 1.];
                let peers = frames.iter()
                    .filter(|frame| !frame.local && frame.tester_valid)
                    .filter_map(|frame| Some((u8::try_from(frame.player).ok()?, at(frame))))
                    .collect();
                let own = frames.iter().find(|frame| frame.local && frame.tester_valid).map(at);
                (peers, own)
            })
            .unwrap_or_default();
        let round_tick = self.campaign_round_tick();
        let mut coop = std::mem::take(&mut self.online.coop);
        // M14: the challenge's character entries, for the story characters this console streams
        // (their identities) and the others' it shows (the entries their puppets stand for).
        let story_ids = self.career.as_ref().map(|career| crate::challenge::characters::story_identities(career.system.world())).unwrap_or_default();
        let mut challenge = self.career.as_mut().map(|career| career.system.world_mut());
        let sent = self
            .inner
            .try_with_selected_queued_frame(|mut frame, ground, _| -> Result<Option<(Vec<u8>, usize)>, String> {
                let Some(recipes) = frame.living_world_recipes.as_mut() else { return Ok(None) };
                let cvars = frame.cvars;
                // M19: the attribute vault, for a puppet's hand prop as its owner's.
                let vault = recipes.vault.clone();
                let vault = &vault;
                let world = &mut *frame.world;
                let living_owner = Arc::clone(world.grind_services.living_world());
                let (centre, _) = crate::simulation_assembly_backend::constraint_session::dmo_living_world::local_skater_census_centre(frame.actor)
                    .map_err(|e| format!("coop world: skater centre: {e:?}"))?;
                // The peers' positions for the census's population rule (M6).
                living_owner.lock().map_err(|_| "LivingWorld lock")?.coop_peers = peers.iter().map(|&(_, at)| at).collect();

                // 1. The owners' samples drive the puppets; an owner's knockdown plays here.
                {
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let living = &mut *living;
                    let models = &living.dmo_models;
                    let roster = living.pedestrian_entities.as_deref_mut().ok_or("pedestrian EntityMan absent")?;
                    // 0. Own entities another console claimed become puppets first.
                    for id in std::mem::take(&mut coop.demotions) {
                        let Some(known) = coop.entities.get(&id).filter(|known| known.owner != Owner::Local) else { continue };
                        let Some(pedestrian) = roster.get_mut(known.entity) else { continue };
                        if pedestrian.puppet.is_none()
                            && let Some(core) = pedestrian.core.as_mut()
                        {
                            core.allow_culling = false;
                            let transform = core.transform;
                            let mut drive = PuppetDrive::new(transform);
                            // demoted during or right after a knockdown: the graph owes the exit tick
                            drive.was_colliding = pedestrian.state.as_ref().is_some_and(|state| state.flags[1] & 4 != 0) || PuppetDrive::colliding(pedestrian.collision.as_ref());
                            pedestrian.puppet = Some(drive);
                            trace(format_args!("own pedestrian {id:08x} demoted: another console claimed it at {tick}"));
                        }
                    }
                    for (sample, late) in coop.updates.drain(..) {
                        let Some(puppet) = coop.entities.get_mut(&sample.id).filter(|known| known.owner != Owner::Local) else { continue };
                        let Some(pedestrian) = roster.get_mut(puppet.entity) else {
                            trace(format_args!("puppet {:08x}: its entity {:#x} is gone from the roster", sample.id, puppet.entity));
                            coop.entities.remove(&sample.id);
                            continue;
                        };
                        // The live audit: the puppet's state against the sample about to drive it.
                        if let Some(core) = pedestrian.core.as_ref() {
                            let audit = &mut coop.stats.audit;
                            let position_gap = distance_squared(core.transform[3], [sample.position[0], sample.position[1], sample.position[2], 1.]).sqrt();
                            let yaw_gap = {
                                let turn = (stream::yaw_of(&core.transform) - sample.yaw).rem_euclid(std::f32::consts::TAU);
                                turn.min(std::f32::consts::TAU - turn).to_degrees()
                            };
                            let speed = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                            let speed_gap = (speed([core.velocity[0], core.velocity[1], core.velocity[2]]) - speed(sample.velocity)).abs();
                            let flags_here = entity_flags(pedestrian);
                            audit.samples += 1;
                            audit.position_ok += u64::from(position_gap <= AUDIT_POSITION);
                            audit.yaw_ok += u64::from(yaw_gap <= AUDIT_YAW_DEGREES);
                            audit.speed_ok += u64::from(speed_gap <= AUDIT_SPEED);
                            audit.flags_ok += u64::from(flags_here == sample.flags);
                            if position_gap > audit.worst_position.0 {
                                audit.worst_position = (position_gap, sample.id);
                            }
                            if yaw_gap > audit.worst_yaw.0 {
                                audit.worst_yaw = (yaw_gap, sample.id);
                            }
                            if position_gap > AUDIT_POSITION {
                                puppet.behind += 1;
                                if puppet.behind == AUDIT_BEHIND_SAMPLES {
                                    trace(format_args!("live audit: puppet {:08x} {position_gap:.2} m from its owner's sample for {AUDIT_BEHIND_SAMPLES} samples at {tick} (yaw off {yaw_gap:.0}°, speed off {speed_gap:.2} m/s)", sample.id));
                                }
                            } else {
                                puppet.behind = 0;
                            }
                        }
                        let Some(drive) = pedestrian.puppet.as_mut() else { continue };
                        drive.arrive_late(stream::transform(sample.position, sample.yaw), sample.velocity, sample.flags, late);
                        // M20: the owner's intents, once their names are heard.
                        match coop.intent_names.get(&sample.intents) {
                            Some(names) if sample.intents != 0 => {
                                if drive.owner_intents.as_ref().is_none_or(|(hash, _)| *hash != sample.intents) {
                                    drive.owner_intents = Some((sample.intents, names.clone()));
                                }
                                coop.stats.intents_known += 1;
                            }
                            // A list not named here yet (its run lost): the M6 band from the
                            // owner's speed until it is, not the last list (a walk played on
                            // while the owner stood).
                            _ => {
                                drive.owner_intents = None;
                                coop.stats.intents_unknown += u64::from(sample.intents != 0);
                            }
                        }
                        // M16: the owner's graph state holds this puppet's graph.
                        if !drive.held_failed {
                            drive.graph = (sample.graph != crate::generated_action_load::NULL_INDEX).then_some(sample.graph);
                        }
                        // M19: a descriptor's hand prop, taken or dropped as the owner's.
                        if let Some(descriptor) = sample.descriptor.as_ref() {
                            sync_hand_prop(pedestrian, descriptor.hand_prop, vault, models, cvars, sample.id)?;
                        }
                        puppet.updated = tick;
                        let knocked = sample.flags & flags::KNOCKED_DOWN != 0;
                        // M16: a standing hit the owner took plays here from its flags, as a
                        // knockdown does; this console predicts neither (a puppet's hit is
                        // reported and the owner's word plays, one round trip later).
                        if sample.flags & flags::STANDING_HIT != 0 && !knocked && !pedestrian.collision.as_ref().is_some_and(|collision| collision.colliding) {
                            let direction = direction_of(sample.flags);
                            pedestrian.record_accepted_full_collision(CollisionForce::Standing, direction, cvars)?;
                        }
                        if knocked && !puppet.knocked_down {
                            puppet.knocked_down = true;
                            if puppet.awaiting.take().is_some() {
                                coop.stats.confirmed += 1;
                                trace(format_args!("puppet {:08x}: the owner confirmed the knockdown at {tick}", sample.id));
                            } else {
                                // M20: played below, this tick or once the puppet is free (until
                                // M20 a knockdown that found the puppet colliding was dropped).
                                puppet.knockdown_due = Some(direction_of(sample.flags));
                            }
                        } else if !knocked {
                            puppet.knockdown_due = None;
                            if puppet.knocked_down {
                                trace(format_args!("puppet {:08x}: up again at {tick}", sample.id));
                            }
                            puppet.knocked_down = false;
                            // The owner is up while this copy still plays a knockdown: the owner's
                            // word ends it. The collision is closed as its own completion closes it
                            // (the graph then leaves the colliding state and the behaviour's `End`
                            // clears the knocked-down flag and releases the intent packet). A
                            // prediction the owner never confirmed, past the grace, is a refusal.
                            let down_here = pedestrian.state.as_ref().is_some_and(|state| state.flags[1] & 4 != 0);
                            let awaited_long_enough = puppet.awaiting.is_none_or(|at| tick.saturating_sub(at) > CONFIRM_GRACE_TICKS);
                            if down_here && awaited_long_enough {
                                // M16: whatever the collision's own state (its behaviour may have
                                // been cut short by a held update), the owner's word stands: the
                                // puppet is up, its knocked-down flag cleared, its collision closed.
                                if let Some(collision) = pedestrian.collision.as_mut() {
                                    collision.colliding = false;
                                    collision.required_to_begin_colliding = false;
                                }
                                if let Some(state) = pedestrian.state.as_mut() {
                                    state.flags[1] &= !4;
                                }
                                // The collision's intent packet would park at the queue's front (a puppet
                                // never advances its queue outside a collision) and reject the next hit.
                                pedestrian.clear_all_intents()?;
                                if puppet.awaiting.take().is_some() {
                                    coop.stats.refused += 1;
                                    trace(format_args!("puppet {:08x}: its owner is up, the reported knockdown unconfirmed for {CONFIRM_GRACE_TICKS} ticks: standing up at {tick}", sample.id));
                                } else {
                                    trace(format_args!("puppet {:08x}: its owner is up while this copy lies knocked down: standing up at {tick}", sample.id));
                                }
                            }
                        }
                    }

                    // 1b. M20: the owners' knockdowns due. A puppet still in a standing hit of
                    // its own (the owner's, played a sample late) has it ended (its packet
                    // cleared, so its completion closes it at the next update); the knockdown
                    // is recorded on the first tick it is free. One already down here is the
                    // same knockdown.
                    for (&id, puppet) in coop.entities.iter_mut().filter(|(_, known)| known.owner != Owner::Local) {
                        let Some(direction) = puppet.knockdown_due else { continue };
                        let Some(pedestrian) = roster.get_mut(puppet.entity).filter(|pedestrian| pedestrian.puppet.is_some()) else {
                            puppet.knockdown_due = None;
                            continue;
                        };
                        if pedestrian.state.as_ref().is_some_and(|state| state.flags[1] & 4 != 0) {
                            puppet.knockdown_due = None;
                            continue;
                        }
                        if PuppetDrive::colliding(pedestrian.collision.as_ref()) {
                            pedestrian.clear_all_intents()?;
                            continue;
                        }
                        pedestrian.record_accepted_full_collision(CollisionForce::Knockdown, direction, cvars)?;
                        puppet.knockdown_due = None;
                        trace(format_args!("puppet {id:08x}: knocked down by its owner ({direction:?}) at {tick}"));
                    }

                    // 2. The others' contacts on this console's own pedestrians, through the
                    // same funnel a local hit takes.
                    for contact in coop.received_contacts.drain(..) {
                        let Some(entity) = coop.entities.get(&contact.id).filter(|known| known.owner == Owner::Local).map(|known| known.entity) else {
                            trace(format_args!("contact on {:08x}: not this console's", contact.id));
                            continue;
                        };
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
                        object.touched_by = 0;
                    }
                    let (nudges, on_round_ticks) = diagnostic_nudges();
                    if let Some(&at) = nudges.get(coop.nudged)
                        && let Some(now) = if on_round_ticks { round_tick } else { Some(tick) }
                        && now >= at
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
                            coop.nudged += 1;
                            let object = objects.object_mut(id).expect("listed");
                            // The claim below decides whether the touch takes it (a puppet
                            // its owner is moving stays driven, the push lost).
                            object.touched = true;
                            object.touched_by |= 1;
                            world.try_set_dmo_part_velocity(&object.body, 0, [0., 3., 4.])?;
                            trace(format_args!("diagnostic: nudged prop {:016x} (entity {id:#x}) at {:.1} m at {tick}: a touch by this console's skater", object.locator_id, d2.sqrt()));
                        } else if now == at {
                            trace(format_args!("diagnostic: no prop within 30 m to nudge at {tick}"));
                        }
                    }
                    for (&locator, &id) in &by_locator {
                        let Some(object) = objects.object_mut(id) else { continue };
                        if !object.touched {
                            continue;
                        }
                        object.touched = false;
                        let players = std::mem::take(&mut object.touched_by);
                        let world_push = players & crate::living_world_grind_services::dmo_objects::WORLD_TOUCH != 0;
                        let players = players & !crate::living_world_grind_services::dmo_objects::WORLD_TOUCH;
                        let (old, claim) = coop.props.get(&locator).map_or((None, 1), |known| (Some(known.owner), known.claim + 1));
                        if old == Some(Owner::Local) {
                            coop.props.get_mut(&locator).expect("known").touched = tick;
                            continue;
                        }
                        // M10.1: the owner keeps a prop it is still moving; the touch here waits.
                        if let Some(known) = coop.props.get_mut(&locator).filter(|known| known.moving && tick.saturating_sub(known.sampled) <= CONTEST_TICKS) {
                            known.waited += 1;
                            if known.waited == 1 {
                                trace(format_args!("prop {locator:016x}: touched here (players {players:#b}) while {:?} is moving it (its sample at {}): the touch waits, the puppet stays driven, at {tick}", known.owner, known.sampled));
                            }
                            continue;
                        }
                        ownership::decide(Kind::Prop, locator as u32, old, ownership::owner(Kind::Prop, Owner::Local), &format!("this console's skater touched it last (players {players:#b}{}, claim {claim})", if world_push { ", a car's or a pedestrian's push" } else { "" }));
                        object.coop_puppet = false;
                        coop.props.insert(locator, PropOwner { owner: Owner::Local, claim, touched: tick, sent: None, runs: 0, sampled: 0, moving: false, waited: 0 });
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
                    // M14: another console's story character is offered to the challenge's
                    // character manager first: the entry it stands for may hold this console's
                    // own copy, and the lower slot keeps its own.
                    let story = (spawn.state.kind == Kind::StoryCharacter).then_some(spawn.descriptor.story).flatten();
                    if let (Some(identity), Some(challenge)) = (story, challenge.as_deref_mut())
                        && !crate::challenge::characters::remote_story_offered(challenge, identity, spawn.owner, local_slot)
                    {
                        trace(format_args!("story character {id:08x} of slot {}: refused, this console's own copy of its entry stands (the lower slot's)", spawn.owner));
                        coop.refused.insert(id);
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
                        let living = &mut *living;
                        let models = &living.dmo_models;
                        let pedestrian = living.pedestrian_entities.as_deref_mut().and_then(|p| p.get_mut(entity)).ok_or("spawned puppet absent")?;
                        let mut drive = PuppetDrive::new(transform);
                        drive.arrive(transform, spawn.state.velocity, spawn.state.flags);
                        pedestrian.puppet = Some(drive);
                        if let Some(core) = pedestrian.core.as_mut() {
                            core.colour_indices = spawn.descriptor.colours;
                            core.allow_culling = false;
                        }
                        // M19: the owner's hand prop, spawned here by the hand-prop pass as any
                        // pedestrian's pending prop is.
                        sync_hand_prop(pedestrian, spawn.descriptor.hand_prop, vault, models, cvars, id)?;
                    }
                    let kind = spawn.state.kind;
                    ownership::decide(kind, id, None, Owner::Remote(owner), "its census spawned it");
                    trace(format_args!("puppet {id:08x} of slot {owner} spawned as entity {entity:#x} (entity key {:016x}, model {:016x}, epoch {}) at {tick}",
                        spawn.descriptor.entity_key, spawn.descriptor.model_key, spawn.state.epoch));
                    coop.entities.insert(id, Tracked { entity, owner: Owner::Remote(owner), epoch: spawn.state.epoch, kind, updated: tick, awaiting: None, knocked_down: spawn.state.flags & flags::KNOCKED_DOWN != 0, knockdown_due: None, runs: 0, nearer: (0, 0), behind: 0, sent: None });
                    if let (Some(identity), Some(challenge)) = (story, challenge.as_deref_mut()) {
                        crate::challenge::characters::remote_story_spawned(challenge, identity, owner, entity, spawn.state.position);
                    }
                    if spawn.promote {
                        coop.promotions.push(id);
                    }
                    coop.spawns.remove(&id);
                }

                // 3b. Puppets handed to this console are promoted: their graph and navigation
                // take over where the puppet stood, and this console streams them from now on.
                for id in std::mem::take(&mut coop.promotions) {
                    let Some(known) = coop.entities.get_mut(&id).filter(|known| known.owner != Owner::Local) else { continue };
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let Some(pedestrian) = living.pedestrian_entities.as_deref_mut().and_then(|p| p.get_mut(known.entity)) else { continue };
                    pedestrian.puppet = None;
                    if let Some(core) = pedestrian.core.as_mut() {
                        core.allow_culling = true;
                    }
                    // M20: its intents start clean (ClearAllIntents): the mirrored stage and the
                    // packets its held graph suggested as a puppet go, and the graph input's
                    // completions with them, so its own intent generator starts from idle.
                    pedestrian.clear_all_intents()?;
                    let old = known.owner;
                    known.owner = ownership::decide(known.kind, id, Some(old), Owner::Local, &format!("handed to this console: its player is nearer (epoch {})", known.epoch));
                    known.runs = 0;
                    known.nearer = (0, 0);
                    coop.locals.insert(known.entity, id);
                    trace(format_args!("puppet {id:08x} promoted: entity {:#x} runs its own graph from {tick}", known.entity));
                }

                // 4. Puppets their owner despawned or stopped streaming go.
                let mut culls: Vec<(u32, &'static str)> = coop.despawns.drain(..).map(|id| (id, "its owner despawned it")).collect();
                for (&id, puppet) in coop.entities.iter().filter(|(_, known)| known.owner != Owner::Local) {
                    if tick.saturating_sub(puppet.updated) > STALE_TICKS {
                        culls.push((id, "its owner stopped streaming it"));
                    }
                }
                for (id, reason) in culls {
                    let Some(puppet) = coop.entities.get(&id).filter(|known| known.owner != Owner::Local) else { continue };
                    let (entity, owner, kind) = (puppet.entity, puppet.owner, puppet.kind);
                    coop.entities.remove(&id);
                    recipes.cull_census_pedestrian(world, cvars, entity)?;
                    if kind == Kind::StoryCharacter && let Some(challenge) = challenge.as_deref_mut() {
                        crate::challenge::characters::remote_story_gone(challenge, entity);
                    }
                    trace(format_args!("puppet {id:08x} of {owner:?}: culled at {tick}, {reason}"));
                }

                // 5. The diagnostic shove: the nearest puppet, hit as a skater would hit it.
                if let Some(at) = diagnostic_shove_tick()
                    && tick >= at
                    && !coop.shoved
                {
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let roster = living.pedestrian_entities.as_deref_mut().ok_or("pedestrian EntityMan absent")?;
                    let nearest = coop
                        .entities
                        .iter()
                        .filter(|(_, known)| known.owner != Owner::Local)
                        .filter_map(|(&id, puppet)| {
                            let core = roster.get(puppet.entity)?.core.as_ref()?;
                            Some((distance_squared(core.transform[3], centre), id, puppet.entity))
                        })
                        .min_by(|a, b| a.0.total_cmp(&b.0));
                    if let Some((d2, id, entity)) = nearest {
                        coop.shoved = true;
                        let pedestrian = roster.get_mut(entity).expect("listed");
                        // M16: as a skater's hit, the contact alone goes to the owner; nothing
                        // plays here until the owner's flags and graph state say so.
                        if let Some(drive) = pedestrian.puppet.as_mut() {
                            drive.pending_contact = Some((CollisionForce::Knockdown as u32 as u8, CollisionDirection::Front as u32 as u8, [0., 0., 6.], [0., 0., 0.]));
                        }
                        trace(format_args!("diagnostic: shoved puppet {id:08x} at {:.1} m: contact to its owner at {tick}, its word awaited", d2.sqrt()));
                    } else if tick == at {
                        trace(format_args!("diagnostic: no puppet to shove at {tick}"));
                    }
                }

                // 5b. The census dump for the synchronization audit, at round ticks the
                // seats share.
                if census_trace_on()
                    && let Some(round_tick) = round_tick
                    && round_tick % CENSUS_EVERY == 0
                {
                    let living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let roster = living.pedestrian_entities.as_deref().ok_or("pedestrian EntityMan absent")?;
                    let by_entity: BTreeMap<u32, (u32, &Tracked)> = coop.entities.iter().map(|(&wid, tracked)| (tracked.entity, (wid, tracked))).collect();
                    for id in roster.ids() {
                        let Some(pedestrian) = roster.get(id) else { continue };
                        let Some(core) = pedestrian.core.as_ref() else { continue };
                        let Some(&(wid, tracked)) = by_entity.get(&id) else { continue };
                        let owner = match tracked.owner {
                            Owner::Local => "own".to_owned(),
                            Owner::Remote(slot) => format!("puppet of {slot}"),
                        };
                        let speed = (core.velocity[0].powi(2) + core.velocity[1].powi(2) + core.velocity[2].powi(2)).sqrt();
                        let down = pedestrian.state.as_ref().is_some_and(|state| state.flags[1] & 4 != 0);
                        // M20: the intents its motion graph is given, and how many packets wait.
                        let intents = pedestrian.intent_queue.as_ref().map_or_else(|| "none".to_owned(), |queue| {
                            let names = queue.active_stage().map_or_else(|_| "absent".to_owned(), |stage| {
                                stage.intents().iter().map(|name| name.decode().unwrap_or_else(|_| "?".to_owned())).collect::<Vec<_>>().join("/")
                            });
                            format!("q{} {names}", queue.queued())
                        });
                        let played = pedestrian.animation.as_ref().map_or_else(|| "none".to_owned(), |animation| {
                            animation.motion_intents.names().iter().map(|name| name.decode().unwrap_or_else(|_| "?".to_owned())).collect::<Vec<_>>().join("/")
                        });
                        trace(format_args!(
                            "census {round_tick}: {wid:08x} {owner} e{} {:.2} {:.2} {:.2} yaw {:.1} v {:.2} f{} m{:016x} g{} moods{} {} i {intents}",
                            tracked.epoch, core.transform[3][0], core.transform[3][1], core.transform[3][2],
                            stream::yaw_of(&core.transform).to_degrees(), speed, entity_flags(pedestrian), core.model_key,
                            pedestrian.graph.current, pedestrian.state.as_ref().map_or(0, |state| state.moods.len()),
                            if down { "down" } else { "up" }
                        ));
                        trace(format_args!("census {round_tick}: {wid:08x} plays {played}"));
                    }
                }

                // M19's baseline: this console's own pedestrians' jerk, every tick, measured as
                // the puppets' is (a step against the step before), so the puppets' figure has
                // the original's own to stand against.
                {
                    let living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    if let Some(roster) = living.pedestrian_entities.as_deref() {
                        let mut seen = BTreeSet::new();
                        for id in roster.ids() {
                            let Some(pedestrian) = roster.get(id).filter(|pedestrian| pedestrian.puppet.is_none()) else { continue };
                            let Some(core) = pedestrian.core.as_ref() else { continue };
                            let at = [core.transform[3][0], core.transform[3][1], core.transform[3][2]];
                            seen.insert(id);
                            // M20: the trail the measured velocity is read from; a jump past a
                            // metre (a respawn, a teleport) starts it again.
                            let trail = coop.own_trail.entry(id).or_default();
                            if trail.back().is_some_and(|last| (0..3).map(|lane| (at[lane] - last[lane]).powi(2)).sum::<f32>() > 1.) {
                                trail.clear();
                            }
                            trail.push_back(at);
                            if trail.len() > TRAIL_TICKS {
                                trail.pop_front();
                            }
                            if let Some((last, last_step)) = coop.own_steps.get(&id).copied() {
                                let step = [at[0] - last[0], at[1] - last[1], at[2] - last[2]];
                                let jerk = (0..3).map(|lane| (step[lane] - last_step[lane]).powi(2)).sum::<f32>().sqrt();
                                let flat = ((step[0] - last_step[0]).powi(2) + (step[2] - last_step[2]).powi(2)).sqrt();
                                // A step past 1 m is a respawn or a teleport, not a walk.
                                if step.iter().map(|value| value * value).sum::<f32>() < 1. {
                                    coop.stats.own_ticks += 1;
                                    coop.stats.own_jerk += f64::from(jerk);
                                    coop.stats.own_over += u64::from(jerk > 0.005);
                                    coop.stats.own_over_flat += u64::from(flat > 0.005);
                                }
                                coop.own_steps.insert(id, (at, step));
                            } else {
                                coop.own_steps.insert(id, (at, [0.; 3]));
                            }
                        }
                        coop.own_steps.retain(|id, _| seen.contains(id));
                        coop.own_trail.retain(|id, _| seen.contains(id));
                    }
                }

                // 6. This console's own run, at the stream's cadence.
                if tick % INTERVAL != 0 {
                    return Ok(None);
                }
                let mut run = WorldRun::default();
                let mut owned = Vec::new();
                let roster_size;
                {
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    let roster = living.pedestrian_entities.as_deref_mut().ok_or("pedestrian EntityMan absent")?;
                    let present: BTreeSet<u32> = roster.ids().into_iter().collect();
                    roster_size = present.len();
                    for &id in &present {
                        let pedestrian = roster.get_mut(id).expect("listed");
                        if pedestrian.puppet.is_some() {
                            continue;
                        }
                        // M14: a pedestrian a challenge placed (a story character) streams as
                        // the others do, from the console whose tester box spawned it, wherever
                        // the players are (a mission's character is wanted on every console),
                        // and is never handed over by distance; until M14 it was every
                        // console's own (open item 70).
                        let story = pedestrian.challenge_owner.is_some();
                        let Some(core) = pedestrian.core.as_ref() else { continue };
                        let guard = pedestrian.security_guard.is_some();
                        let d2 = distance_squared(core.transform[3], centre);
                        if d2 > STREAM_RADIUS * STREAM_RADIUS && !story {
                            continue;
                        }
                        // The table entry of an owned pedestrian, made at its first sample.
                        let wid = *coop.locals.entry(id).or_insert_with(|| stream::world_id(local_slot, id));
                        let kind = if story { Kind::StoryCharacter } else if guard { Kind::SecurityGuard } else { Kind::Pedestrian };
                        let tracked = coop.entities.entry(wid).or_insert_with(|| {
                            let reason = if story { "a challenge placed it here: streamed to the others, never handed over" } else if guard { "this census spawned it; it chases this console's skater only" } else { "this census spawned it" };
                            ownership::decide(kind, wid, None, ownership::owner(kind, Owner::Local), reason);
                            Tracked { entity: id, owner: Owner::Local, epoch: 0, kind, updated: tick, awaiting: None, knocked_down: false, knockdown_due: None, runs: 0, nearer: (0, 0), behind: 0, sent: None }
                        });
                        // The handoff rule: a peer nearer by the margin for a second takes it.
                        let position = core.transform[3];
                        let own_d2 = own_facts.map_or(d2, |at| distance_squared(position, at));
                        let nearest_peer = peers.iter().map(|&(slot, at)| (distance_squared(position, at), slot)).min_by(|a, b| a.0.total_cmp(&b.0));
                        match nearest_peer {
                            Some((peer_d2, slot)) if !story && peer_d2.sqrt() + HANDOFF_MARGIN < own_d2.sqrt() && peer_d2 <= STREAM_RADIUS * STREAM_RADIUS => {
                                tracked.nearer = if tracked.nearer.1 == slot { (tracked.nearer.0 + 1, slot) } else { (1, slot) };
                            }
                            _ => tracked.nearer = (0, 0),
                        }
                        if tracked.nearer.0 >= HANDOFF_TICKS {
                            let slot = tracked.nearer.1;
                            tracked.epoch += 1;
                            tracked.owner = ownership::decide(kind, wid, Some(Owner::Local), Owner::Remote(slot), &format!("that player is nearer by {HANDOFF_MARGIN} m for {HANDOFF_TICKS} ticks (epoch {})", tracked.epoch));
                            tracked.updated = tick;
                            tracked.nearer = (0, 0);
                            let transform = core.transform;
                            let velocity = [core.velocity[0], core.velocity[1], core.velocity[2]];
                            let mut drive = PuppetDrive::new(transform);
                            drive.arrive(transform, velocity, 0);
                            pedestrian.puppet = Some(drive);
                            if let Some(core) = pedestrian.core.as_mut() {
                                core.allow_culling = false;
                            }
                            coop.handoffs.insert(wid, DESPAWN_RUNS);
                            continue;
                        }
                        let Some(core) = pedestrian.core.as_ref() else { continue };
                        let entity_flags = entity_flags(pedestrian);
                        let describe = tracked.runs % DESCRIBE_EVERY == 0;
                        // M17: the send-on-error rule. The receiver carries the last sample on
                        // at its velocity (`PuppetDrive::step`), so a sample that says the same
                        // is not sent; a change of flags, motion or graph state, a position or
                        // facing the extrapolation misses, a descriptor due or the keepalive is.
                        let position = [core.transform[3][0], core.transform[3][1], core.transform[3][2]];
                        let yaw = stream::yaw_of(&core.transform);
                        // M20: the velocity it moves at (its navigation's while the trail is short).
                        let velocity = coop.own_trail.get(&id).and_then(measured_velocity).unwrap_or([core.velocity[0], core.velocity[1], core.velocity[2]]);
                        let motion = pedestrian.state_out.state.min(255) as u8;
                        let graph = pedestrian.graph.current;
                        // M20: the intents its motion graph is given, sent by hash; named on a
                        // change and with a descriptor.
                        let stage = pedestrian.intent_queue.as_ref().map(|queue| queue.active_intents()).unwrap_or_default();
                        let intents = if stage.is_empty() { 0 } else { crate::pedestrian_puppet::intents_hash(&stage) };
                        let hand_prop = hand_prop_of(pedestrian);
                        // M19: a hand prop taken or dropped is a descriptor's news, sent at once.
                        let describe = describe || tracked.sent.is_some_and(|last| last.hand_prop != hand_prop);
                        let due = match tracked.sent {
                            None => true,
                            Some(last) => {
                                let dt = tick.saturating_sub(last.tick) as f32 / 60.;
                                let predicted = [last.position[0] + last.velocity[0] * dt, last.position[1] + last.velocity[1] * dt, last.position[2] + last.velocity[2] * dt];
                                let off = (0..3).map(|lane| (position[lane] - predicted[lane]).powi(2)).sum::<f32>().sqrt();
                                let turned = (yaw - last.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                                describe
                                    || tracked.runs.wrapping_sub(last.runs) >= KEEPALIVE_RUNS
                                    || off > SEND_POSITION
                                    || turned.abs() > SEND_YAW
                                    || last.flags != entity_flags
                                    || last.motion != motion
                                    || last.graph != graph
                                    || last.intents != intents
                            }
                        };
                        if !due {
                            tracked.runs += 1;
                            coop.stats.withheld += 1;
                            continue;
                        }
                        // Runs are not acknowledged: a change is named in its first
                        // `NAME_SAMPLES` samples, so one lost run does not leave it unnamed.
                        let changed = tracked.sent.is_none_or(|last| last.intents != intents);
                        let names_left = if changed { NAME_SAMPLES - 1 } else { tracked.sent.map_or(0, |last| last.names_left.saturating_sub(1)) };
                        let name_them = intents != 0 && (describe || changed || tracked.sent.is_some_and(|last| last.names_left > 0));
                        coop.stats.intents_named += u64::from(name_them);
                        tracked.sent = Some(Sent { tick, position, yaw, velocity, flags: entity_flags, motion, graph, intents, names_left, runs: tracked.runs, hand_prop });
                        owned.push((d2, EntityState {
                            id: wid,
                            kind,
                            owner: local_slot,
                            epoch: tracked.epoch,
                            position: [core.transform[3][0], core.transform[3][1], core.transform[3][2]],
                            yaw: stream::yaw_of(&core.transform),
                            velocity,
                            flags: entity_flags,
                            motion: pedestrian.state_out.state.min(255) as u8,
                            graph: pedestrian.graph.current,
                            intents,
                            intent_names: name_them.then(|| intent_texts(&stage)),
                            descriptor: describe.then(|| Descriptor {
                                entity_key: core.entity_key,
                                model_key: core.model_key,
                                recipe_id: core.recipe_id,
                                colours: core.colour_indices,
                                story: story_ids.get(&id).copied(),
                                hand_prop,
                            }),
                        }));
                    }
                    // The handoffs still to say: the record names the new owner, with the
                    // descriptor so a console that never saw the entity spawns it.
                    for (&wid, left) in coop.handoffs.iter_mut() {
                        *left -= 1;
                        let Some(tracked) = coop.entities.get(&wid) else { continue };
                        let Owner::Remote(slot) = tracked.owner else { continue };
                        let Some(pedestrian) = roster.get(tracked.entity) else { continue };
                        let Some(core) = pedestrian.core.as_ref() else { continue };
                        let handoff_stage = pedestrian.intent_queue.as_ref().map(|queue| queue.active_intents()).unwrap_or_default();
                        let handoff_velocity = coop.own_trail.get(&tracked.entity).and_then(measured_velocity).unwrap_or([core.velocity[0], core.velocity[1], core.velocity[2]]);
                        let handoff_intents = if handoff_stage.is_empty() { 0 } else { crate::pedestrian_puppet::intents_hash(&handoff_stage) };
                        owned.push((-1., EntityState {
                            id: wid,
                            kind: tracked.kind,
                            owner: slot,
                            epoch: tracked.epoch,
                            position: [core.transform[3][0], core.transform[3][1], core.transform[3][2]],
                            yaw: stream::yaw_of(&core.transform),
                            velocity: handoff_velocity,
                            flags: entity_flags(pedestrian),
                            motion: pedestrian.state_out.state.min(255) as u8,
                            graph: pedestrian.graph.current,
                            intents: handoff_intents,
                            intent_names: (handoff_intents != 0).then(|| intent_texts(&handoff_stage)),
                            descriptor: Some(Descriptor { entity_key: core.entity_key, model_key: core.model_key, recipe_id: core.recipe_id, colours: core.colour_indices, story: None, hand_prop: hand_prop_of(pedestrian) }),
                        }));
                    }
                    coop.handoffs.retain(|_, left| *left > 0);
                    // The contacts this skater made with the others' puppets.
                    for (&id, puppet) in coop.entities.iter_mut().filter(|(_, known)| known.owner != Owner::Local) {
                        let Some(contact) = roster.get_mut(puppet.entity).and_then(|p| p.puppet.as_mut()).and_then(|drive| drive.pending_contact.take()) else { continue };
                        let (force, direction, velocity, omega) = contact;
                        run.contacts.push(Contact { id, force, direction, velocity, omega });
                        if force == 0 {
                            puppet.awaiting = Some(tick); // a knockdown: the owner's flags confirm or refuse it
                        }
                        coop.stats.contacts_sent += 1;
                        trace(format_args!("contact on puppet {id:08x} of {:?}: force {force} direction {direction}, reported to its owner at {tick}", puppet.owner));
                    }
                    // Owned entities gone from the roster are despawned to the others.
                    let gone: Vec<(u32, u32)> = coop.entities.iter().filter(|(_, known)| known.owner == Owner::Local && !present.contains(&known.entity)).map(|(&wid, known)| (wid, known.entity)).collect();
                    for (wid, entity) in gone {
                        coop.entities.remove(&wid);
                        coop.locals.remove(&entity);
                        coop.gone.insert(wid, DESPAWN_RUNS);
                        trace(format_args!("own pedestrian {wid:08x} left the roster: despawn announced"));
                    }
                    coop.locals.retain(|entity, _| present.contains(entity));
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
                        // A prop touched this tick is not at rest, whatever its cached velocity
                        // (read back from the transforms after the step) says.
                        run.props.push(PropState { locator, claim: owner.claim, position, rotation: stream::quaternion_of(&object.transform), velocity, resting: speed2 <= 1e-4 && owner.touched != tick });
                    }
                }
                run.despawns = coop.gone.keys().copied().collect();
                coop.gone.retain(|_, left| {
                    *left -= 1;
                    *left > 0
                });
                let (bytes, included) = stream::encode(&run, MAX_WORLD);
                for entity in &run.entities[..included] {
                    if let Some(tracked) = coop.entities.get_mut(&entity.id).filter(|known| known.owner == Owner::Local) {
                        tracked.runs += 1;
                    }
                }
                // M17: a sample the budget cut was not sent: the rule sends it next run.
                for entity in &run.entities[included..] {
                    if let Some(tracked) = coop.entities.get_mut(&entity.id).filter(|known| known.owner == Owner::Local) {
                        tracked.sent = None;
                    }
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
                        "world stream at {tick}: {} runs, {} entity samples, {} prop samples, {} B ({:.0} B/s, {:.0} B/s per streamed pedestrian), {} owned in range of {} pedestrians here, {} puppets, {} props owned of {} known, contacts sent {} applied {} confirmed {} refused {}, {} records ignored for an older epoch",
                        stats.runs, stats.entities, stats.props, stats.bytes, stats.bytes as f64 / seconds, per_entity, run.entities.len(), roster_size, coop.entities.values().filter(|known| known.owner != Owner::Local).count(),
                        coop.props.values().filter(|prop| prop.owner == Owner::Local).count(), coop.props.len(),
                        stats.contacts_sent, stats.contacts_applied, stats.confirmed, stats.refused, stats.stale
                    ));
                    trace(format_args!("world stream at {tick}: {} samples withheld by the send-on-error rule (M17)", stats.withheld));
                    trace(format_args!("world stream at {tick}: intents named in {} samples sent; puppet samples with their owner's intents {}, with intents not yet named {}; {} intent lists heard (M20)",
                        stats.intents_named, stats.intents_known, stats.intents_unknown, coop.intent_names.len()));
                    if stats.late_samples > 0 {
                        trace(format_args!("world runs received at {tick}: {} with samples, lateness mean {:.2} ticks, worst {}, {} past 3 ticks (M20)",
                            stats.late_samples, stats.late_sum as f64 / stats.late_samples as f64, stats.late_worst, stats.late_over));
                    }
                    // M19: the puppets' jerk since the last window: each tick's step against the
                    // tick before's, summed over every puppet here.
                    let mut living = living_owner.lock().map_err(|_| "LivingWorld lock")?;
                    if let Some(roster) = living.pedestrian_entities.as_deref_mut() {
                        let (mut ticks, mut stray, mut over, mut over_flat) = (0_u64, 0_f64, 0_u64, 0_u64);
                        for entity in roster.ids() {
                            let Some(drive) = roster.get_mut(entity).and_then(|pedestrian| pedestrian.puppet.as_mut()) else { continue };
                            let meter = std::mem::take(&mut drive.stutter);
                            ticks += u64::from(meter.ticks);
                            stray += f64::from(meter.stray);
                            over += u64::from(meter.over);
                            over_flat += u64::from(meter.over_flat);
                        }
                        if ticks > 0 {
                            trace(format_args!("puppet jerk at {tick}: {ticks} ticks, mean {:.2} mm, {over} past 5 mm ({:.2} %), {over_flat} along the ground (M19)", 1000. * stray / ticks as f64, 100. * over as f64 / ticks as f64));
                        }
                    }
                    let own = &*stats;
                    if own.own_ticks > 0 {
                        trace(format_args!("own pedestrians' jerk at {tick}: {} ticks, mean {:.2} mm, {} past 5 mm ({:.2} %), {} along the ground (M19's baseline)",
                            own.own_ticks, 1000. * own.own_jerk / own.own_ticks as f64, own.own_over, 100. * own.own_over as f64 / own.own_ticks as f64, own.own_over_flat));
                    }
                    let audit = &stats.audit;
                    if audit.samples > 0 {
                        trace(format_args!(
                            "live audit at {tick}: {} samples; position within {AUDIT_POSITION} m {} (worst {:.2} m, {:08x}), yaw within {AUDIT_YAW_DEGREES}° {} (worst {:.0}°, {:08x}), speed within {AUDIT_SPEED} m/s {}, flags equal {}",
                            audit.samples, audit.position_ok, audit.worst_position.0, audit.worst_position.1, audit.yaw_ok, audit.worst_yaw.0, audit.worst_yaw.1, audit.speed_ok, audit.flags_ok
                        ));
                    }
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
        // M17: the bytes sent by channel, cumulative, every measure window.
        if tick % MEASURE_TICKS == 0
            && let Some(client) = self.online.client.as_ref()
        {
            let b = client.channel_bytes();
            trace(format_args!("net bytes at {tick}: frames {}, owned frames {}, world {}, facts {}, looks {}, other {} (cumulative, encoded messages)", b.frames, b.owned_frames, b.world, b.facts, b.looks, b.other));
        }
        Ok(())
    }
}
