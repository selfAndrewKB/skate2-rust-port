# RE: driving living-world entities from outside (read 2026-11-10)

What selects a pedestrian's locomotion, how vehicles, movable props and guards keep their
per-tick state, and what the replay recorder keeps of them: the facts M5 and M6 need to stream
each kind and to drive a puppet without its own AI. Paths are under `crates/skate2-core/src/`,
at commit `0918b45`.

## Pedestrian locomotion

- `PedStateOut::state` is the audio/speech state (72 at birth, 8 on a light hit, guard speech
  events, takedown 71/19, spectate and marquee values), read by the audio only. Its `flags`:
  0x10/0x20 right/left foot down, 0x40 was a player collision, 0x80 stopped. The world stream's
  `EntityState::motion` carries this value today; it is not walk/idle.
- The clips come from intent names: `PedestrianAnimation::motion_intents` (up to four names,
  `count`, `valid`), snapshotted by `update_animation_data` into the motion graph's intents each
  AnimMan update (`animation/graphs/pedestrian_motion_live.rs:22-30`). The graph
  (`pedestrian_motion_data.rs:650+`) has Locomotion children Collision (Knockdown, Standing),
  Takedown, Avoid, Jump, Taunt, Converse, Sit, Strafe, ChaseRun, Shuffle, Idle, Tired, Walk,
  gated by `intent:` attributes.
- Who writes the names: `IntentQueue::update` (`pedestrian_intent_queue.rs:153-172`): with no
  managed packet, `update_base_locomotion` takes the speed from the nav rig
  (`speed_after_clearance_update`, from `core.velocity`) and modifier overrides, and
  `select_speed_mapping` picks the Vault `PedestrianSpeedMapping` (key 0x9c9a4f2cb1cc45de:
  velocity, major, minor, allowed previous) whose velocity is not below it; a turn stage,
  "Strafe", "ForceStop" are added by state. Managed packets (collision, avoid, jump, takedown)
  replace the base stage.
- Root motion: `update_animation_target_transform`
  (`session/living_world/selected_living_world_pedestrian_target.rs:18-82`) composes the target
  from the pose's root delta, spatial adjust, up, `rotate_to_target` from the active stage, and
  the DMO shift; `core.velocity` is derived afterwards from transform to target.
- A knockdown is a managed packet `["Collision", "Knockdown"|"Standing", <direction>]` plus
  recover stages, set by `CollidingBehaviour::begin` once `record_accepted_full_collision` set
  `required_to_begin_colliding`; it depends on the AI graph running
  (`pedestrian_graph_conditions.rs:39-41`).
- Per-tick order inside `try_update2` (`selected_living_world_dmo.rs:1050-1150`): LOS states;
  `update_pedestrian_current_frame` (queries, navigation modifiers, `update_intent` → the intent
  queue); `update_pedestrian_preframe` (threats, wants, chasee, zones, `update_ai_graph` when
  `ai_bindings` is set, `motion_intents.valid = false`, navigation, ground and volume queries);
  then animation over the whole roster; `try_sync_animation` (root motion into the target);
  `try_update_post_transforms` (the fork's puppet override, collision drive, head tracking,
  post-adjust, `core.update_transform`).
- There is no per-entity skip flag in the execution loops; the one gate is
  `entity.ai_bindings.is_some()`. Animation, collision and transforms iterate the full roster
  apart from the execution targets. So a puppet without AI needs: its intent names set from the
  owner (a speed mapping from the streamed velocity, or the owner's names), `valid` kept true,
  the knockdown packet from the owner's flags, and the AI graph and intent queue skipped.

## Vehicles (`living_world/vehicles/`)

- `Vehicle` (`living_world_vehicle_entity.rs:53-83`): `core` (entity type 3), `collision`,
  `graph`, `grab_splines`, `grind_splines`, `motion: VehicleMotion` (speed, acceleration,
  stopping reason, parked, alarming, skidding, backing up, force stopping, being skitched,
  grabbing, skitch helper timer), `route: VehicleRoute` (`lights: u8`: 2 brake, 4 reverse, 8|16
  indicators, 0x80 blink phase), `signals`, `wheels: VehicleWheelState` (target heading,
  radius, spin angle, turn angle), `vision`, `nav_rig`, `characteristics`, `driver`,
  `skater_collision_flags`.
- Per tick (`living_world_vehicle_frame.rs`): LOS states (358) → preframe and the Vehicle.xml
  state graph (380) → `vehicle_update_targets` (route advance, `update_target_transform` from
  road geometry) → ground tests → skitch flags; in post transforms: collisions (the skater body
  driven toward `part_target(core)`, never free), post transforms, `core.update_transform`,
  wheels.
- Census (`living_world_census_vehicle.rs`): `evaluate` culls outside the circle and spawns
  through `request_spawn_vehicle(VehicleSpawnInfo { entity_key, model_key, recipe_id, transform,
  allow_culling, force_spawn })`; `request_cull` ends behaviours, leaves the road network and
  queues the destroy.
- Contact: `vehicle_resolve_contacts` takes the first island contact: a parked vehicle alarms,
  a skater owner sets `colliding_with_skater` and the player bit, a contact ahead sets
  `force_stopping`. Skitch: `physical_player_grab_world.rs:40-117` (`NotifyLocalPlayer` kind 1
  → `local_player_grabbing`, `NotifyGrabbing` → `being_skitched`) and the skitch perception.
- A remote copy needs: transform and velocity, wheels (spin, turn), `route.lights`, `motion`'s
  parked/alarming/skidding/backing/being_skitched, and who skitches.

## Movable props (`living_world/dmos/living_world_dmo_objects.rs`)

- `DynamicObject` (530-584): `entity_id`, `locator_id`, `model`, `transform`,
  `target_transform`, `velocity` (flat `[f32;16]`), `flags` (0x02 condition cull, 0x04 surface
  contact, 0x10 static, 0x20 in use by a player, 0x80 constructed), `skater_collision_flags`,
  `transform_type: Physics | HandProp`, `body: SimulationDmoAssemblyHandle`, `group` (light or
  heavy), `grab_splines`, `grind_splines`, `online_static`.
- A free body's target is Part 0's island transform each tick (`try_update_target_transform`,
  2104); only hand props are driven toward their target (the holding pedestrian's). Body state
  bits: 2 frozen, 4 active (`simulation_rigid_body_activation.rs:10-31`); no sleeping field on
  the object.
- Skater interaction is the physics contact; grabs through `physical_player_grab_world.rs`
  (`ApplyForce`, `NotifyLocalPlayer` kind 2 → `set_in_use_by_player`); hand props through
  `set_hand_prop`, `release_hand_prop_to_physics`.
- A remote copy needs: transform, velocity, body active/frozen, in-use-by, hand prop holder.
  The online free skate's `online_dmos.rs` places the challenge hull's objects static.

## Security guards (`living_world/security/`)

- A guard is a pedestrian of class `SecurityGuard` spawned by the census where a WorldPainter
  region 6 (security areas) exists ("security_nsz01/02", `enable_spawn_security`), with
  `SecurityGuardData { zone, previous_guardee_position, guardee_closing_velocity,
  outside_cull_radius, break_off_forced }`.
- Targets: `TargetableId { kind, id }`: kind 0 a skater (id = player index), 1 a pedestrian.
  Threats come from every Actor in the entity roster within 35 m; the chase target is
  `state.wants[1].target`; `GuardBehaviour::NewChasee` sets `state.chaser.chasee`; the chasee
  keeps `ChaseeState { chasers, group_end_reason }` (a skater's on `actor.prefix().chasee`).
- Warn (`register_warner`), taze (`RegisterTazer`) and the capture message fire only for the
  local player (`is_local_player`/`is_local_actor` through `local_skater_identity`);
  `update_chase_manager` handles the local skater only (zone statistics, capture → jacked,
  respawn). A guard's culling waits on `challenge_system_can_cull(chasee_exists)`.
- "The player it chases" is `state.chaser.chasee` of kind 0: the ownership rule's input.

## The replay recorder's living world

- `RecordedWorld { dynamic_objects, vehicles, pedestrians, traffic_lights }`
  (`session/selected_session_replay.rs:75-80`), filled per tick by `recorded_world_visuals()`;
  pedestrian palettes frozen per frame.
- Visuals: `SelectedPedestrianVisual` (entity id, recipe, model, palette, colours, position,
  `pose`, transform, bounds), `SelectedVehicleVisual` (model, scene, transform, `wheels`,
  colours), `SelectedDynamicObjectVisual` (asset, render model, part transform).
- A transform-only path exists: `living_world_submission`
  (`selected_session_presentation_submit.rs:115-173`) records `EntityReplay(id, transform)`,
  `VehicleReplay(id, transform, spin, turn, lights)` and `ObjectReplay(id, transform)`;
  `LivingWorldRestore::restore` (`rendering/presentation_restore_living_world.rs:~290-320`)
  blends each entity between two frames, creates new ones, destroys missing ones. A streamed
  vehicle or prop could be drawn through it without a census entity; a pedestrian needs its
  pose, which that path does not carry.
