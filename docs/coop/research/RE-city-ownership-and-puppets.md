# RE: the city: population, puppets' animation, ownership handoff, the stream's coding (read 2026-11-10, M6)

What the port's census does around one skater, what a puppet's animation is made of, what a
pedestrian's state is, and what the port's own frames do for acknowledgement; and the rules M6
gave each. Code under `crates/skate2-core/src/`: `living_world/population/`,
`living_world/pedestrians/pedestrian_intent_queue.rs`, `pedestrian_animation.rs`,
`pedestrian_entity_intent.rs`, `session/living_world/selected_living_world_pedestrian_*.rs`,
`session/online/coop_world.rs`, `coop/world_stream.rs`; `skate2-net/src/frames.rs`.

## The census around one skater [CONFIRMED by reading]

- The centre is the local actor (`local_skater_census_centre`), shifted along its velocity;
  the ring (`no_spawn`, `spawn`, `cull`) blends two circles by speed. Each tick the cull pass
  culls what has `allow_culling` beyond the cull radius, then the spawn pass makes 2 attempts
  for 1 spawn on the ring (6000 for 600 at the initial spawn): a random point, the painted
  density gate (`spawn_minimum` against the roster's whole count), then `spawn`'s allowance
  (`max_pedestrians`, 12 on the bench) against the roster's whole count.
- Every roster member counts: puppets, story characters, guards; `force_spawn` only raises
  the one spawn's limit. A puppet (`allow_culling` false) is never census-culled; it goes by
  its owner's despawn or after 180 stale ticks. So ten puppets left the allowance only two
  own spawns (open item 3), and two skaters apart each filled their own ring, twice where
  the rings met.
- The rule (M6): the peers' positions (from the facts) reach the census each tick
  (`LivingWorldGrindServices::coop_peers`): a spawn point nearer a peer than this console is
  that peer's census's (`nearer_a_peer`, the random point and a prop's point alike); a puppet
  nearer a peer than this console does not count against this console's allowance
  (`Services::count`); an entity within a peer's cull radius is not culled here (the handoff
  takes it there), only one beyond every player's radius.

## A puppet's animation [CONFIRMED by reading]

- A pedestrian's motion graph reads intents by name only (`HasIntent`): the intent queue
  (`IntentQueue::update`) either plays a packet (a collision, a jump) or picks the base
  locomotion from the navigation's speed through the animation data's speed table
  (`select_speed_mapping`: a major and a minor name per speed band), and `set_packet` copies
  the active stage's names into the motion graph's input. `PedStateOut::state`, the byte M1
  streamed as "motion", is the speech and audio event state (72 default, 8 a light hit), not
  locomotion.
- A puppet ran its own navigation and graph, so its intents were its own wander's while its
  transform was its owner's: it walked in place or slid (open item 1); since M5 it had no
  threats or moods, so it never startled or fled when its owner's copy did (the player's
  first play, open item 31).
- The rule (M6): a puppet's intents come from its owner's velocity through the same speed
  table (`update_puppet_intent`): walk, run or stand as the owner's copy moves; no
  navigation, no turn stage (its yaw is the sample's); its own graph and navigation run only
  while it collides (`PuppetDrive::colliding`), so a knockdown's packet plays. A startled run
  shows as a run; a startle's stumble and the mood's gestures do not travel yet (the
  behaviour itself would have to).
- The rule since M20 [CONFIRMED on the bench]: the owner's active stage travels (a hash, its
  names on a change) and the puppet's base packet takes it (`IntentQueue::mirror_stage`); a
  trailing turn name (`override_turn`'s eight) goes in as its own stage, which the puppet's
  motion graph completes and `pop_completed_turn` pops. The speed table stood in for intents
  it could not see: a stop to talk read as a slow walk (`shuffle`) from the speed left over,
  and a walk near a band's edge flickered between bands (the walk in place and the pauses,
  `../bench/evidence/probe/2026-11-12-m20-walk.md`).
- The puppet's look: the census spawn drew a model and colours for the owner's entity key at
  random on the receiving side, though the descriptor carried them; the spawn now takes the
  owner's model key and recipe id (`PedestrianCensusFrame::look`,
  `spawn_census_pedestrian_as`).

## Ownership handoff [CONFIRMED by reading]

- A pedestrian's state beyond the stream: its AI state (wants, knowledge, moods, chaser,
  timers), its graph's current behaviour, its navigation (waypoint, rig cell), its collision
  info, its hand prop. A puppet already holds a real entity whose graph can run, so promotion
  is clearing `puppet` and `allow_culling` and letting it re-plan (its wants re-form on the
  next evaluation, a direction change at most); demotion the mirror.
- The world id named the current owner (its slot in the top bits), and every receiver route
  (samples, despawns, contacts, the announce and gone tables) keyed on that, so a migrated
  entity would have broken all of them.
- The rule (M6): a world id is fixed at first spawn (the spawner's slot and local id) and
  never changes; every console keeps one table by world id (`Tracked`: the local entity, the
  owner, the epoch, the kind). An owned pedestrian a peer is nearer to by 15 m for 60 ticks,
  within the peer's stream radius, is handed over: this console's copy becomes a puppet, the
  record names the new owner's slot with the epoch raised (repeated three runs, with the
  descriptor, so a console that never saw the entity spawns it), and the new owner promotes
  its puppet. The highest epoch wins everywhere; a record with a lower epoch is stale. The
  promoted entity keeps the navigation cell it was spawned in [open item: a re-seat at the
  promotion is owed].

## The stream's coding [CONFIRMED by reading]

- The port's replay frames are coded against baselines the receiver acknowledged
  (`FrameRing::ack`, `ClientMessage::FrameAcks`, the server re-coding each run per receiver
  against that receiver's holds, `AckState::baselines`); the facts coder codes against the
  previous tick over reliable, ordered delivery. The world stream has no acknowledgement:
  `ClientMessage::World` is relayed opaquely to every member, runs are self-contained, so a
  descriptor repeats every 20 runs and a despawn 3 times.
- The smallest addition, not yet built: acks inside the run (every console sends a run every
  3 ticks anyway): a header of (slot, latest tick, mask) per peer and a `base` distance, each
  entity coded as zigzag deltas against its sample in the newest run every peer holds, the
  descriptor only until acknowledged; a sender ring of ~64 runs, a receiver ring per owner;
  on loss the base stays and the deltas grow. Measured first ([world](../plan/topics/world.md)):
  whether the budget needs it.
