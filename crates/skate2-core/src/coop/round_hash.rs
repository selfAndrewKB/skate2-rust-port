//! Co-op fork (M23): the campaign round's state after a tick, hashed in the sync check's parts
//! (`super::sync_check`), the port's `Simulation::GetSyncLog` for the scripts.
//!
//! Only what every console's scripts hold alike goes in: no handle, id or table number a
//! console allocates in its own order (an objective group's id, a timer's id, a Lua table's
//! number), no HUD or presentation state (each console's own time), no local player flag. An
//! activity a timer belongs to is named by its key, a table by its contents, and a collection
//! whose order is allocation order is hashed as a set.

use super::sync_check::{Fnv, PARTS, Part, Parts};
use crate::challenge::lua::{Lua, TableId, Value};
use crate::challenge::mission::MissionFrame;
use crate::challenge::system::{ChallengeSystem, World};

/// Nested tables hashed below an activity's vars table at most this deep.
const TABLE_DEPTH: u32 = 4;

/// The round's parts after a tick, but for [`Part::Inputs`] (the frames, hashed by [`inputs`]
/// before the scripts take them), left zero.
#[must_use]
pub fn round_parts(system: &ChallengeSystem) -> Parts {
    let world = system.world();
    let mut parts = [0; PARTS];
    for (index, part) in Part::ALL.into_iter().enumerate() {
        parts[index] = match part {
            Part::Inputs => 0,
            Part::Activities => activities(world),
            Part::Objectives => objectives(world),
            Part::Timers => timers(world),
            Part::Vars => vars(&system.challenge, world),
        };
    }
    parts
}

/// The frames as every console runs them: without what only the owner's console has of its
/// own frame (the local flag, its pushes), as `SKATE2_TRACE_ONLINE_INPUTS` hashes them.
#[must_use]
pub fn inputs(frames: &[MissionFrame]) -> u32 {
    let mut fnv = Fnv::default();
    for frame in frames {
        fnv.str(&shared_text(frame));
    }
    fnv.finish()
}

/// A frame's text without its owner-only fields.
#[must_use]
pub fn shared_text(frame: &MissionFrame) -> String {
    format!("{:?}", MissionFrame { local: false, is_pushing: false, is_physically_pushing: false, is_new_auto_pump: false, ..frame.clone() })
}

/// Each activity's graph (its current state's key, whether it runs) in creation order, which is
/// the career's activity table's on every console; the main activity's key.
fn activities(world: &World) -> u32 {
    let mut fnv = Fnv::default();
    let activities = &world.activities.activities;
    fnv.option(world.activities.main.and_then(|main| activities.get(main)), |fnv, activity| fnv.u64(activity.key));
    for activity in activities {
        fnv.u64(activity.key);
        fnv.option(activity.graph.current.and_then(|state| activity.graph.states.get(state)), |fnv, state| fnv.u64(state.key));
        fnv.bool(activity.graph.running);
    }
    fnv.finish()
}

/// The objective groups as a set (their ids and order are allocation's), each by its challenge,
/// player and objectives' states and counters.
fn objectives(world: &World) -> u32 {
    let mut groups: Vec<u32> = world
        .objectives
        .groups
        .iter()
        .map(|group| {
            let mut fnv = Fnv::default();
            fnv.u64(group.challenge);
            fnv.i32(group.player);
            fnv.bool(group.evaluate);
            fnv.u32(group.points_requirement);
            for &trick in &group.completed_unique_tricks {
                fnv.u64(trick);
            }
            for objective in &group.objectives {
                fnv.u64(objective.key);
                fnv.i32(objective.player);
                fnv.u32(objective.state as u32);
                fnv.u32(objective.total_points);
                fnv.u32(objective.last_sequence_points);
                fnv.u32(objective.num_pushes);
                fnv.bool(objective.in_scoring_boundary);
            }
            fnv.finish()
        })
        .collect();
    groups.sort_unstable();
    let mut fnv = Fnv::default();
    for group in groups {
        fnv.u32(group);
    }
    fnv.finish()
}

/// The timers as a set (their ids are allocation's), each by its ticks, kind, pause, expiry,
/// owning activity's key and subject.
fn timers(world: &World) -> u32 {
    let activities = &world.activities.activities;
    let mut timers: Vec<u32> = world
        .timers
        .sync_view()
        .map(|(timer, owner, subject)| {
            let mut fnv = Fnv::default();
            fnv.i32(timer.ticks);
            fnv.u32(timer.kind as u32);
            fnv.bool(timer.paused);
            fnv.option(owner.and_then(|owner| activities.get(owner)), |fnv, activity| fnv.u64(activity.key));
            fnv.option(subject, Fnv::i32);
            fnv.finish()
        })
        .collect();
    timers.sort_unstable();
    let mut fnv = Fnv::default();
    for timer in timers {
        fnv.u32(timer);
    }
    fnv.finish()
}

/// Each activity's vars table (`registry[Name]`, `PushActivityVars`) by its contents, in
/// activity order; an activity with none hashes as nil.
fn vars(lua: &Lua<World>, world: &World) -> u32 {
    let registry = lua.table(lua.registry());
    let mut fnv = Fnv::default();
    for activity in &world.activities.activities {
        fnv.u64(activity.key);
        fnv.u32(value_hash(lua, &registry.get_str(&activity.name), TABLE_DEPTH));
    }
    fnv.finish()
}

/// A Lua value by its contents: a table as the set of its entries (its traversal order is its
/// insertion history), a function or userdata by its type alone.
fn value_hash<H>(lua: &Lua<H>, value: &Value, depth: u32) -> u32 {
    let mut fnv = Fnv::default();
    match value {
        Value::Nil => fnv.bytes(&[0]),
        Value::Boolean(value) => {
            fnv.bytes(&[1]);
            fnv.bool(*value);
        }
        Value::Number(value) => {
            fnv.bytes(&[2]);
            fnv.f32(*value);
        }
        Value::String(value) => {
            fnv.bytes(&[3]);
            fnv.bytes(value.as_bytes());
        }
        Value::Table(table) => {
            fnv.bytes(&[4]);
            if depth > 0 {
                fnv.u32(table_hash(lua, *table, depth - 1));
            }
        }
        Value::Function(_) => fnv.bytes(&[5]),
        Value::UserData(_) | Value::LightUserData(_) => fnv.bytes(&[6]),
    }
    fnv.finish()
}

fn table_hash<H>(lua: &Lua<H>, table: TableId, depth: u32) -> u32 {
    let table = lua.table(table);
    let mut entries = Vec::new();
    let mut key = Value::Nil;
    while let Ok(Some((next, value))) = table.next(&key) {
        let mut fnv = Fnv::default();
        fnv.u32(value_hash(lua, &next, depth));
        fnv.u32(value_hash(lua, &value, depth));
        entries.push(fnv.finish());
        key = next;
    }
    entries.sort_unstable();
    let mut fnv = Fnv::default();
    for entry in entries {
        fnv.u32(entry);
    }
    fnv.finish()
}

/// The part's contents as text, a line per field, for the field dump: what the bench diffs
/// between two seats' logs when the part differs.
#[must_use]
pub fn dump(system: &ChallengeSystem, part: Part) -> Vec<String> {
    let world = system.world();
    let activities = &world.activities.activities;
    match part {
        Part::Inputs => Vec::new(),
        Part::Activities => activities
            .iter()
            .map(|activity| {
                let state = activity.graph.current.and_then(|state| activity.graph.states.get(state)).map_or("-", |state| state.name.as_str());
                format!("{} {:016x} state {state} running {}", activity.name, activity.key, activity.graph.running)
            })
            .collect(),
        Part::Objectives => {
            let mut lines: Vec<String> = world
                .objectives
                .groups
                .iter()
                .flat_map(|group| {
                    group.objectives.iter().map(move |objective| {
                        format!(
                            "{:016x} player {} objective {:016x} {:?} points {} last {} pushes {} boundary {}",
                            group.challenge, group.player, objective.key, objective.state, objective.total_points, objective.last_sequence_points, objective.num_pushes, objective.in_scoring_boundary
                        )
                    })
                })
                .collect();
            lines.sort();
            lines
        }
        Part::Timers => {
            let mut lines: Vec<String> = world
                .timers
                .sync_view()
                .map(|(timer, owner, subject)| {
                    let owner = owner.and_then(|owner| activities.get(owner)).map_or("-", |activity| activity.name.as_str());
                    format!("{owner} {:?} ticks {} paused {} subject {subject:?}", timer.kind, timer.ticks, timer.paused)
                })
                .collect();
            lines.sort();
            lines
        }
        Part::Vars => {
            let lua = &system.challenge;
            let registry = lua.table(lua.registry());
            let mut lines = Vec::new();
            for activity in activities {
                let Some(table) = registry.get_str(&activity.name).as_table() else { continue };
                let table = lua.table(table);
                let mut entries = Vec::new();
                let mut key = Value::Nil;
                while let Ok(Some((next, value))) = table.next(&key) {
                    entries.push(format!("{}.{} = {}", activity.name, value_text(&next), value_text(&value)));
                    key = next;
                }
                entries.sort();
                lines.extend(entries);
            }
            lines
        }
    }
}

fn value_text(value: &Value) -> String {
    match value {
        Value::Nil => "nil".into(),
        Value::Boolean(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => String::from_utf8_lossy(value.as_bytes()).into_owned(),
        other => other.type_name().into(),
    }
}

/// `SKATE2_DIAGNOSTIC_DESYNC`: a divergence planted in this console's state, the first activity
/// with a vars table getting `coopPlantedDivergence = 1`; the activity's name, if one had vars.
pub fn plant_divergence(system: &mut ChallengeSystem) -> Option<String> {
    let lua = &mut system.challenge;
    let registry = lua.registry();
    let (name, table) = system.world.activities.activities.iter().find_map(|activity| lua.table(registry).get_str(&activity.name).as_table().map(|table| (activity.name.clone(), table)))?;
    lua.table_mut(table).set_str("coopPlantedDivergence", Value::Number(1.0));
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(lua: &mut Lua<()>, entries: &[(&str, f32)]) -> Value {
        let table = lua.create_table();
        for &(key, value) in entries {
            lua.table_mut(table).set_str(key, Value::Number(value));
        }
        Value::Table(table)
    }

    #[test]
    fn a_table_hashes_by_its_contents_whatever_its_order_or_number() {
        let mut lua = Lua::<()>::new();
        let first = table(&mut lua, &[("bails", 2.0), ("timer", 30.0), ("stage", 1.0)]);
        let second = table(&mut lua, &[("stage", 1.0), ("bails", 2.0), ("timer", 30.0)]);
        let other = table(&mut lua, &[("stage", 1.0), ("bails", 3.0), ("timer", 30.0)]);
        assert_ne!(first.as_table(), second.as_table());
        assert_eq!(value_hash(&lua, &first, TABLE_DEPTH), value_hash(&lua, &second, TABLE_DEPTH));
        assert_ne!(value_hash(&lua, &first, TABLE_DEPTH), value_hash(&lua, &other, TABLE_DEPTH));
    }

    #[test]
    fn a_nested_table_counts_and_a_function_is_its_type_alone() {
        let mut lua = Lua::<()>::new();
        let inner = table(&mut lua, &[("x", 1.0)]);
        let changed = table(&mut lua, &[("x", 2.0)]);
        let outer = |lua: &mut Lua<()>, inner: &Value| {
            let table = lua.create_table();
            lua.table_mut(table).set_str("inner", inner.clone());
            Value::Table(table)
        };
        let (a, b) = (outer(&mut lua, &inner), outer(&mut lua, &changed));
        assert_ne!(value_hash(&lua, &a, TABLE_DEPTH), value_hash(&lua, &b, TABLE_DEPTH));
        assert_eq!(value_hash(&lua, &Value::Function(crate::challenge::lua::FunctionId(1)), 1), value_hash(&lua, &Value::Function(crate::challenge::lua::FunctionId(2)), 1));
    }
}
