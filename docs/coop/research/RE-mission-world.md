# RE: the mission-required world in a campaign: story characters, props, guards, the services (read 2026-11-10, M5)

What the port does with the living-world pieces a mission needs, what two consoles running the
same scripts did with them, and the rule M5 gives each. Code under `crates/skate2-core/src/`:
`challenge/characters.rs`, `challenge/dmo.rs`, `living_world/dmos/`, `living_world/security/`,
`living_world/pedestrians/pedestrian_graph_frame_*.rs`, `challenge/services.rs`,
`challenge/contacts.rs`; the co-op side in `coop/ownership.rs`, `coop/world_stream.rs`,
`session/online/coop_world.rs`.

## Story characters [CONFIRMED by reading; the bench shows the spawns]

- A challenge's characters (Slappy outside his shop, Reda at the flip-trick locator) are
  `cChallengeCharacterManager`'s: an entry per location or waypoint character once the
  activity's prerequisites pass, a box per entry, and every update the tester's box tested
  against them (`spot_test_characters_against_local_player` → `test_bbox_against_tree`): an
  entry entered gets a ground line test, its model and a forced census spawn
  (`LivingWorldOp::Spawn`, answered by the session with `challenge_owner` set on the
  pedestrian); an entry left is culled. The entity is an ordinary census pedestrian.
- Every later reference is the console's own: the character token, the cull keep lists
  (`CensusManagedCullAll`), the state flags and waypoint ops, and the pedestrian's marquee AI,
  which greets, faces and photographs the local skater (`pedestrian_graph_marquee.rs`). Speech
  (`PlayNpcSpeech`) takes a voice index, never the entity; scenes load their own characters.
  No script reads the character's existence into the facts.
- Two consoles: each spawned its own copy, and M1's stream sent it to the other as a puppet
  too, so a seat near Slappy saw two; the puppet, unknown to the other console's keep list, was
  culled and respawned by every challenge cull. M3's test per player's tester was worse still:
  `test_bbox_against_tree` overwrote `previous` per base, so an entry in one tester's box and
  not the other's was released and re-queued every tick (its line batch's answer thrown away)
  and never spawned while the skaters stood apart.
- The rule (`Rule::EachConsole`): every console spawns and simulates its own copy where its own
  skater is (the box test is the local tester's again); nothing travels (`coop_world.rs` skips a
  pedestrian with `challenge_owner`). Each player's Slappy greets that player; a console whose
  skater is far sees none, which nobody can see.

## Props: the dynamic objects [CONFIRMED by reading]

- A `DynamicObject` is a physics body in the island (`living_world_dmo_objects.rs`: `transform`,
  `target_transform`, `velocity`, `locator_id`, flags static / in use / condition cull). Each
  tick the body's part transform becomes the target (`try_update_target_transform`; a hand prop
  keeps the target its pedestrian set) and the target the transform (`try_update_transform`),
  which the locator follows. A skater moves one by contact (the skater–DMO job; the scoring
  pass sets `skater_collision_flags` for the player, `living_world_scoring.rs`) or by pushing
  and grabbing (`physical_player_grab_world.rs`, force to the body). A hand prop is driven to
  its target by `try_drive_living_world_collision_body` (velocity and spin toward it, a frozen
  body woken).
- The port's online free skate synchronizes no DMO: it places the hull's objects as static
  bodies nothing moves (`online_dmos.rs`), and the protocol has no DMO message. A campaign keeps
  the career's living world, so each console simulated its props alone: a prop one skater moved
  stayed where it was on the other console.
- The career's host answers the objectives' DMO queries with nothing (`CareerHost`:
  `dmo_locator_from_lua_id` none, `is_dmo_in_volume` false), the port's own gap: the opening
  tutorial's "moved object" objective completes only by diagnostic, offline too. When upstream
  answers it, the answer must be a fact of the owner's, not each console's body, or the lockstep
  splits ([open items](../plan/open-items.md)).
- The rule (`Rule::LastToucher`): the locator id (the asset's guid, the same everywhere) names a
  prop across consoles; the console whose skater's body touched it last owns it, each takeover
  claiming one higher than the claim it knew, the highest claim winning everywhere, a tie the
  lower slot (`ownership::claim_wins`). The owner streams it in the run's prop list (format 2:
  locator, claim, position, a unit quaternion, velocity, resting) while it moves, for 120 ticks
  after a touch, and every 20th run at rest; the receiver's object becomes a puppet
  (`coop_puppet`): its target is the owner's sample, never the body's, and its body is driven
  there as a hand prop's is. A challenge's DMO reset runs in lockstep on every console and moves
  the owner's copy too, so the stream agrees with it.

## Guards and chases [CONFIRMED by reading]

- A guard is a census pedestrian with `security_guard` (keys `security_nsz01/02`; Big Black is
  `big_black_regular01`), spawned only inside a world-painter security area (layer 6), never
  culled while chasing. Its graph (`living_world_guard_graph.rs`) takes a chasee from its wants
  (`NewChasee`), the chase manager (`ChaseMan`, one per console) ranks chasers, predicts
  intercepts and hands out formation slots; a takedown wipes the skater out and "jacks" it
  (`living_world_chase_frame.rs`: the jacked timer, then a teleport to the breadcrumb or the
  challenge start).
- "The player" is `TargetableId { kind: 0, id: player_index }`, resolved through the actor
  roster, which holds local actors only: a remote skater is a replay, not an actor, so a guard
  can only ever chase the skater of the console that simulates it. Every registration with
  `ChaseMan` is gated on `is_local_player`. `GetIfPlayerBeingChased` and the security service's
  availability read this console's `ChaseMan`; `cMsgLocalChaseStart/End` come from the guard's
  graph and are dropped in an online round.
- Two consoles: a puppet guard on B ran its whole graph (threats, wants, chase, takedown) and
  could chase and bust B's skater while its body followed A's stream; A's guards never saw B's
  skater; the scripts' "being chased" differed per console.
- The rule (`Rule::Chasee`): a guard belongs to the console of the player it chases, which is
  its spawner until M6's migration; a puppet perceives no threats, wants nothing, chases and
  guards nobody (`update_pedestrian_preframe_states` skips those steps for a puppet; its graph
  still animates it). The chase is a fact: `MissionFrame::being_chased` and `jacked`, filled on
  each console from its `ChaseMan` for its own skater (`career_mission_frames`), so every
  console's scripts read any player's chase at the same tick (`any_player_chased`), and the
  local chase messages are the facts' edges (`CampaignWorld::chase_on`). Protocol 120.

## The services [CONFIRMED by reading]

- The phone (`contacts.rs`) buys a service: security hires Big Black for 500 (a protector who
  walks with the local skater; guards back off while `ChaseeHasProtector`), uncapping 100,
  draining 200; `services::activate` sends `cMsgServiceRequested` and takes the cash, and the
  scripts play the rest (`StartSecurityTimerHUDAndSpawnBigBlack`, 300 s). `ServiceManager` is
  runtime only; the save holds the cash and the "available" settings.
- The rule: the purchase and the cancel are notes (`PromptAnswer::Service`, `ServiceCancel`):
  the phone pushes the answer to `World::coop_prompts`, the session queues it as a note, and
  every console applies it at the note's tick with the same availability and cash check (the
  shared cash pays once). Big Black is each console's own, assigned to its local actor, so
  everyone is protected; `big_black_active` is still each console's readback of its own spawn
  [ASSUMPTION: the spawns agree; a console whose spawn fails would branch alone].

## The bench's switches (M5)

`SKATE2_DIAGNOSTIC_COOP_NUDGE=<tick>` gives the nearest prop within 30 m a velocity and counts
it as touched; `SKATE2_DIAGNOSTIC_COOP_ZONES=<round tick>` scans the security areas within 600 m
and traces the nearest point of each; `COOP_PROMPT=<tick>:service:<n>` and `service_cancel`.
