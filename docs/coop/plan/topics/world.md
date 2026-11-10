# World: the shared living world (route D)

**Scope:** who owns each pedestrian, car, prop and guard, how the owner streams it, how the
others drive puppets, how a contact reaches the owner, and what it costs. Front page:
[README](../../README.md). Design: [design](../design.md) §3.6. Research:
[pedestrians](../../research/RE-living-world-pedestrians.md),
[transport](../../research/RE-frame-transport.md).

## State (2026-11-10, M1) [CONFIRMED on the bench unless marked]

**Ownership** (`crates/skate2-core/src/coop/ownership.rs`): one rule per kind, one funnel that
traces every decision (entity, old owner, new owner, reason). M1 knows `Rule::Spawner`: the
census that spawned a pedestrian owns it for life. Nearest-player ownership with hysteresis and
migration: M6.

**The stream** (`coop/world_stream.rs`, format 1; `ClientMessage::World` relayed as it is to the
campaign's other members): every third tick a seat sends the pedestrians it owns within 80 m of
its skater, nearest first, until `MAX_WORLD` (1024 B): position (1/256 m), yaw, velocity
(1/64 m/s), flags (knocked down, colliding, hit direction, standing hit), the owner's motion
state, and a descriptor (entity key, model, recipe, colours) with the first run and every 20th
after; plus the entities it stopped owning (said in three runs) and the contacts its skater made
with others' puppets. Runs are self-contained; a lost one costs a moment. Measured: 54 B for a
described pedestrian, 25 B plain; live, about 500 B/s per streamed pedestrian at 20 runs a
second.

**Puppets** (`session/online/coop_world.rs`, `pedestrian_puppet.rs`): a streamed entity unknown
here spawns through the census's own spawn (its model loaded first, `force_spawn`, never
culled by distance) and carries a `PuppetDrive`; each tick before the collision bodies are
driven, the puppet's target transform is its owner's last sample carried on at its velocity
(30 ticks at most), so its bodies, its transform and its presence in the census follow as every
pedestrian's do. Its owner's knockdown plays here through `record_accepted_full_collision`
(the same funnel a local hit takes). A puppet is culled when its owner despawns it or has not
streamed it for 180 ticks.

**Contacts:** a full collision of the local skater with a puppet resolves here as for any
pedestrian (the prediction) and is recorded on the drive; the next run carries it (force,
direction, the skater's velocity and angular velocity); the owner applies it through the same
funnel; the owner's flags come back and confirm it ("the owner confirmed the knockdown"). The
bench's shove: predicted on seat 2 at its tick 4800, applied on seat 1 at its 5271, confirmed on
seat 2 at 4805 (five ticks); the puppet up again with the owner's 420 ticks later.

**Measured (2026-11-10, two seats on one Mac, the waterfront):** seat 1 owned 9-11 pedestrians
in range of 12 in its roster and sent 3.4-5.3 KB/s; seat 2 held 9-10 puppets, owned 2-3 of its
own and sent 0.7-1.6 KB/s. Logs: `../bench/evidence/2026-11-10-m1-seat-{1,2}.log`.

**Not done:** the puppet's locomotion from the owner's motion state (it runs its own AI; [open
items](../open-items.md) 1); delta coding; cars, props, guards; migration; the
population rule for a shared region (M6).

## History

- 2026-11-10 M1 (commit `0918b45`): the module, the format, the puppets and the contacts; the
  net message (`a7859ad`, protocol 116). The bench first showed zero pedestrians on both seats:
  the joined online game's flag had turned the census off ([session](session.md)); then the
  career's continue point proved empty of pedestrians in the port itself, so the seats are
  teleported to the waterfront spawn, where the plain free skate fills 12 pedestrians in 20 s.
