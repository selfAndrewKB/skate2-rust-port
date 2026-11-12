#!/usr/bin/env python3
"""The synchronization inventory: every field of a pedestrian's state and of the world stream,
each marked streamed, derived on the receiver, or local-only, generated from the code so a new
field shows up unclassified rather than forgotten.

    python3 tools/coop/sync_inventory.py            # prints the table
    python3 tools/coop/sync_inventory.py --write    # rewrites docs/coop/plan/sync-inventory.md

The classification itself (the CLASSES table below) is the reviewed part: a field the structs
have and the table does not is reported as UNCLASSIFIED and fails the check."""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STREAM = ROOT / 'crates/skate2-core/src/coop/world_stream.rs'
ENTITY = ROOT / 'crates/skate2-core/src/living_world/pedestrians/living_world_pedestrian_entity.rs'
OUT = ROOT / 'docs/coop/plan/sync-inventory.md'

# How each PedestrianConstruction field relates to the wire. Classes:
#   streamed  - travels in the owner's run (names the wire field)
#   derived   - the receiver recomputes it from streamed fields
#   identity  - fixed at spawn from the descriptor (travels once, then equal by construction)
#   local     - this console's own; never equal across consoles by design (the reason says why)
#   plumbing  - allocation, handles, caches: no gameplay state
CLASSES = {
    'id': ('local', 'the local entity id; the world id names it across consoles'),
    'allocations': ('plumbing', 'memory store slots'),
    'authority': ('plumbing', 'graph authority handle'),
    'core': ('streamed', 'transform (position, yaw), velocity, entity key, model key, recipe id, colours'),
    'rain_data': ('plumbing', 'spatial index item'),
    'model': ('identity', 'the descriptor\'s model key (M6)'),
    'entity_category_key': ('identity', 'from the entity key'),
    'challenge_owner': ('local', 'set on the spawner\'s copy; a story character is its spawner\'s, streamed as a pedestrian with its entry\'s identity in the descriptor (M14; every console\'s own M5 to M13)'),
    'hand_prop_attributes': ('local', 'hand props do not travel'),
    'hand_prop_usage': ('local', 'hand props do not travel'),
    'positioning_test_start_height': ('plumbing', 'spawn-time ground test'),
    'target_iterators': ('local', 'navigation targets; a puppet navigates nowhere (M6)'),
    'current_waypoint': ('local', 'navigation; a puppet navigates nowhere'),
    'contained_props': ('local', 'props carried; do not travel'),
    'attrib_cache': ('plumbing', 'attribute cache'),
    'resources': ('plumbing', 'resources'),
    'mesh_navigation': ('local', 'navigation; a puppet navigates nowhere'),
    'physics_navigation': ('local', 'navigation; a puppet navigates nowhere'),
    'state': ('local', 'AI state: wants, knowledge, moods, timers, chaser; a puppet perceives nothing (M5); flags[1] bit 2 (down) travels as KNOCKED_DOWN'),
    'big_black': ('local', 'Big Black is each console\'s own (M5)'),
    'security_guard': ('local', 'a guard chases only its own console\'s skater (M5); its kind travels'),
    'navigation_rig_enabled': ('local', 'navigation'),
    'navigation_smoothing_enabled': ('local', 'navigation'),
    'navigation_rig_override': ('local', 'navigation'),
    'renderable': ('plumbing', 'presentation'),
    'adjust_world_collisions': ('local', 'collision adjust'),
    'ignore_nav_mesh_in_ground_test': ('local', 'ground test'),
    'nav_rig_ignore_entity': ('local', 'navigation'),
    'ignore_collision_target': ('local', 'collision'),
    'ignore_avoid_target': ('local', 'navigation'),
    'chasee': ('local', 'the chase is a fact of the player, not of the pedestrian (M5)'),
    'chaser_should_block': ('local', 'chase'),
    'chaser_block_position': ('local', 'chase'),
    'patrol_entity_of_interest': ('local', 'guard patrol'),
    'patrol_zone_parameters': ('local', 'guard patrol'),
    'patrol_zone_start_point': ('local', 'guard patrol'),
    'patrol_point_of_interest': ('local', 'guard patrol'),
    'patrol_returning': ('local', 'guard patrol'),
    'interacting_object': ('local', 'DMO interaction; props travel on their own (M5)'),
    'held_hand_prop': ('local', 'hand props do not travel'),
    'last_frame_hand_position': ('local', 'hand props do not travel'),
    'plugin': ('local', 'waypoint plugin (spectators)'),
    'plugin_removal_requested': ('local', 'waypoint plugin'),
    'intent_queue': ('derived', 'a puppet\'s base locomotion intents from the streamed velocity (M6); packets (collisions) from the streamed flags'),
    'navigation_intent': ('local', 'navigation'),
    'perception': ('local', 'a puppet perceives nothing (M5)'),
    'state_out': ('streamed', 'state (the speech/audio event) travels as motion; the receiver does not apply it'),
    'collision_info': ('streamed', 'force and direction travel in flags'),
    'navigation_modifiers': ('local', 'navigation'),
    'animation': ('derived', 'the motion graph plays the derived intents and the held graph\'s behaviours (M16); clip phase is each console\'s own'),
    'nav_rig': ('local', 'navigation'),
    'collision': ('derived', 'colliding and required_to_begin_colliding set from the streamed KNOCKED_DOWN, STANDING_HIT and COLLIDING flags; nothing predicted on a puppet since M16'),
    'post_adjust': ('local', 'pose adjust'),
    'head_tracking': ('local', 'head tracking looks at the local skater'),
    'spatial_adjust': ('local', 'pose adjust'),
    'graph': ('streamed', 'the owner\'s current state travels (EntityState::graph, format 5) and a puppet\'s graph is held at it every tick (M16); its own selection only without one'),
    'ai_bindings': ('plumbing', 'graph bindings'),
    'behaviours': ('derived', 'begun and ended by the held graph as the owner\'s are (M16)'),
    'puppet': ('derived', 'the receiver\'s drive: the owner\'s last sample'),
}

FIELD = re.compile(r'^\s*pub(?:\(crate\))?\s+([a-z_][a-z0-9_]*)\s*:')


def struct_fields(path, name):
    text = path.read_text(encoding='utf-8')
    start = text.index(f'struct {name}')
    body = text[start:]
    body = body[body.index('{') + 1:]
    depth, end = 1, 0
    for i, ch in enumerate(body):
        depth += (ch == '{') - (ch == '}')
        if depth == 0:
            end = i
            break
    fields, doc = [], []
    for line in body[:end].splitlines():
        s = line.strip()
        if s.startswith('///'):
            doc.append(s[3:].strip())
            continue
        m = FIELD.match(line)
        if m:
            fields.append((m.group(1), ' '.join(doc)))
            doc = []
        elif not s.startswith('//'):
            doc = []
    return fields


def main():
    wire = {name: [f for f, _ in struct_fields(STREAM, name)] for name in ('EntityState', 'Descriptor', 'PropState', 'Contact')}
    entity = struct_fields(ENTITY, 'PedestrianConstruction')
    lines = ['# The synchronization inventory', '',
             'Generated by `tools/coop/sync_inventory.py` from `coop/world_stream.rs` and',
             '`living_world/pedestrians/living_world_pedestrian_entity.rs`; the classes are the reviewed part',
             '(a field the structs have and the script does not know fails the check). Front page:',
             '[README](../README.md). The audit that measures it: `tools/coop/sync_audit.py`.', '',
             '## What travels (the world stream, format 5)', '']
    for name, fields in wire.items():
        lines.append(f'- `{name}`: ' + ', '.join(f'`{f}`' for f in fields))
    lines += ['', '## A pedestrian\'s state (`PedestrianConstruction`)', '',
              '| Field | Class | Why |', '| --- | --- | --- |']
    unclassified = []
    counts = {}
    for field, _ in entity:
        cls, why = CLASSES.get(field, ('UNCLASSIFIED', 'not in the inventory: classify it'))
        counts[cls] = counts.get(cls, 0) + 1
        if cls == 'UNCLASSIFIED':
            unclassified.append(field)
        lines.append(f'| `{field}` | {cls} | {why} |')
    stale = [f for f in CLASSES if f not in {n for n, _ in entity}]
    lines += ['', '## Counts', '', ', '.join(f'{k} {v}' for k, v in sorted(counts.items()))]
    if stale:
        lines += ['', f'Classified but gone from the struct: {", ".join(stale)}']
    text = '\n'.join(lines) + '\n'
    if '--write' in sys.argv:
        OUT.write_text(text, encoding='utf-8')
        print(f'wrote {OUT}')
    else:
        print(text)
    if unclassified:
        print(f'UNCLASSIFIED: {", ".join(unclassified)}', file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
