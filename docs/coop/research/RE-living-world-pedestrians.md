# RE: the living world's pedestrians (read 2026-11-10)

What a pedestrian is in the port, how it is spawned, moved, hit and knocked down, as M1 had to
drive one from another console. Paths are under `crates/skate2-core/src/`. From the code at
commit `2b80ee7`.

## Spawning

- Per tick, `SelectedLivingWorldDmo::try_update1` (`session/living_world/selected_living_world_dmo.rs:917`)
  resets collision flags, updates ModelMan, waypoints and RAIN, runs `try_update_census` and
  `try_evaluate_population` (:819): culls first (DMOs, pedestrians, vehicles, props), then
  spawns, one category per tick in a round robin of four (`component_index`).
- The census centre and velocity are the local skater's only (`local_skater_census_centre`).
- `evaluate` (`living_world/population/living_world_census_pedestrian.rs:326`): the circle from
  the skater's speed, culls, then spawns if `do_spawn && !banned && enable_spawn_pedestrians &&
  census.enable & 0x20`; `evaluative_spawn` (:207) returns early when `ghost || network`; the
  initial spawn makes 6000 attempts for up to 600 in an 8-80 m ring, a normal tick 2 attempts
  for 1. `spawn` (:93) gates on security zones, `census.enable & 0x80` (`IsLWSpawnEnabled`),
  the recipe's model being resident (`contains_model`), then the count limit
  (`max_pedestrians`, 12 on the bench) and calls `request_spawn(PedestrianSpawnInfo { entity_key,
  model_key, recipe_id, transform, initial, allow_culling, force_spawn, allow_world_collisions,
  load_model, ignore_navmesh_ground, allow_hand_prop })`.
- Outside the census: `SelectedRecipeFrame::spawn_census_pedestrian(world, cvars, ground,
  entity_key, transform, initial, allow_culling, force_spawn, allow_world_collisions,
  ignore_navmesh_ground, allow_hand_prop)` (`selected_living_world_pedestrian_census.rs`) and the
  fork's `cull_census_pedestrian`. Models load through
  `SelectedRecipeFrame::load_living_world_model(world, cvars, key)`.
- Banned in game modes 2|4|5 (`try_update_census`, :785); the `0x80` bit is set at
  `try_reset_game` from `!network_game && !is_network_game && !ghost`, and
  `try_is_network_game()` is `game_info.online_game || is_network_game`: a joined online game
  turns the census off (what M1 found and gated for a campaign).
- RNG: the Manager's `NativeSkaterRandom` seeded from `rw_math::global_next()` (wall clock);
  the global generator is also drawn for census target groups, colours, hand props and a
  collision Bernoulli. Nothing reseeds at `try_reset_game`.
- The career's continue point ([1491.8, 109.7, -743.3]) spawned no pedestrian in 1700 census
  calls offline; the waterfront boot spawn ([786, -58, 882]) fills 12 in 20 s.

## The entity

- `PedestrianConstruction` (`living_world/pedestrians/living_world_pedestrian_entity.rs:61`):
  `id`, `core: Option<CommonEntityCore>`, `model`, `state: Option<PedestrianStateStorage>`,
  `security_guard`, `big_black`, `intent_queue`, `state_out: PedStateOut`, `collision_info`,
  `animation`, `nav_rig`, `collision: Option<PedestrianCollision>`, `graph: PedestrianGraph`,
  `behaviours`, and the fork's `puppet: Option<PuppetDrive>`.
- `CommonEntityCore` (`living_world/entities/living_world_entity_core.rs:14`): `transform`
  (actual), `target_transform` (next), `velocity`, `entity_key`, `model_key`, `recipe_id`,
  `colour_indices`, `current_losim`, `allow_culling`. `update_transform` derives the velocity
  from actual to target and copies the target in; it runs for every pedestrian in
  `try_update_post_transforms` (`selected_living_world_dmo.rs:1221`), after the collision bodies
  were driven toward the target (`update_collision_models` → `try_drive_living_world_collision_body`:
  velocity-driven, never teleported).
- Transforms are `[[f32;4];4]`, rows right, up, forward, translation.
- Ids: one global allocator for every living-world kind, `(type << 24) | counter`, pedestrians
  type 2; monotonic per process, never stable across machines.
- State: `PedestrianStateStorage.flags[1] & 4` knocked down, `& 8` ignore skater collisions,
  `flags[0] & 2` attempting takedown; `PedestrianGraph { current, last }` the AI node;
  `PedStateOut { state, length, topic, target_greet_character, flags }` the motion output.
- No kinematic or puppet mode existed; the replay is presentation-only
  (`rendering/presentation_restore_living_world.rs`).

## Knockdown

- The one funnel: `LivingWorldGrindServices::pedestrian_resolve_contacts`
  (`pedestrian_collision_contacts.rs:387`): every contact of a pedestrian body (world and skater
  bodies, the Skate 3 contacts added) is classified by `determine_collision_type` into Ignore,
  AdjustWorld, AdjustVehicle, AdjustDmoInteraction, Light or Full, then
  `resolve_classified_collision`.
- Full: `resolve_full_collision` (:290) reads the skater body's Part 0 velocity and angular
  velocity, compares with the entity's max linear/angular and the selected animation's
  thresholds, derives Front/Back/Left/Right from the contact normal, and ends in
  `record_accepted_full_collision(force, direction, cvars)` (`pedestrian_graph_colliding.rs:284`),
  which sets `collision_info`, `required_to_begin_colliding` and the recovery flags. The fork
  records a puppet's full collision there as the contact to report.
- `CollidingBehaviour::begin` (`pedestrian_graph_colliding.rs:125`) plays the knockdown or
  standing hit through intent stages, sets `flags[1] |= 4` for a knockdown, a recovery timer from
  RECOVERY_TIME, and ends with `colliding = true`; `update`/`update_recovering`/`end` get the
  pedestrian up again. No ragdoll; no despawn timer; culling is distance only.
- Other producers bypass the classifier: the Skate 3 shove (`pedestrian_collision_shove.rs:58`)
  and takedowns.

## Tick order

Inside `SelectedPlayerTick::try_step_input` (`session/selected_player_tick.rs:525`):
`try_update1` (census) → `try_update2` (AI, execution, targets) → `try_update_animation` →
shove and Skate 3 contacts → physics → `try_resolve_contact_spy_lists` (the knockdown funnel) →
`try_sync_animation` → `try_update_post_transforms`. Only after the tick completes does the
facade run `try_update_career → record_replay_frame → online_tick → update_online_dmos`
(`selected_session_public_facade.rs:~3671`), so the online tick sees the whole living-world tick.
