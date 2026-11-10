# World: the shared living world (route D)

**Scope:** who owns each pedestrian, story character, prop, guard and car, how the owner
streams it, how the others drive puppets, how a contact reaches the owner, and what it costs.
Front page: [README](../../README.md). Design: [design](../design.md) §3.6. Research:
[pedestrians](../../research/RE-living-world-pedestrians.md),
[transport](../../research/RE-frame-transport.md),
[the mission-required world](../../research/RE-mission-world.md),
[the city: population, puppets, handoff, coding](../../research/RE-city-ownership-and-puppets.md).

## State (2026-11-10, M6 in progress) [CONFIRMED on the bench unless marked]

**M6 so far [ASSUMPTION: built and tested in units, not yet run on the bench]:**

- **A puppet is its owner's copy.** Spawned with the owner's model key and recipe id (the
  descriptor's, which the census's own draw ignored before: the player saw different models
  per screen); its walk, run and stand come from the owner's velocity through the animation
  data's own speed table (`update_puppet_intent`), with no navigation and no turn stage; its
  own graph and navigation run only while it collides, so a knockdown still plays. A startled
  run shows as a run; the stumble and the mood's gestures do not travel yet.
- **One world id for life, a table of owners.** A world id is the first spawner's slot and
  local id and never changes; every console keeps `Tracked` by world id (the local entity,
  the owner, the epoch, the kind), and samples, despawns and contacts route through it.
- **The handoff.** An owned pedestrian a peer is nearer to by 15 m for 60 ticks, within the
  peer's stream radius, is handed over: this console's copy becomes a puppet, the record
  names the new owner with the epoch raised (three runs, with the descriptor), the new owner
  promotes its puppet (its graph and navigation take over where it stood); the highest epoch
  wins everywhere. World stream format 3.
- **The population rule.** The peers' positions reach the census each tick: a spawn point
  nearer a peer is that peer's census's to fill, a puppet in a peer's half does not count
  against this console's allowance, and a pedestrian within a peer's cull radius is not
  culled here, only one beyond every player's.
- **Not yet:** delta coding against acknowledged runs (the design is in the research note;
  the budget is measured first); cars and AI skaters; the puppet's stumble and mood gestures.

## State (2026-11-10, M5)

**Ownership** (`crates/skate2-core/src/coop/ownership.rs`): one rule per kind, one funnel that
traces every decision (entity, old owner, new owner, reason). The table:

| Kind | Rule | Streamed |
| --- | --- | --- |
| ambient pedestrian | `Spawner`: the census that spawned it, for life (M1) | yes |
| story character (a pedestrian a challenge placed) | `EachConsole`: every console spawns and simulates its own copy where its own skater is (M5) | no |
| security guard | `Chasee`: the console of the player it chases, which is its spawner until M6's migration (a guard chases only the skater of the console that simulates it) (M5) | yes, as a pedestrian |
| prop (a dynamic object with a locator) | `LastToucher`: the console whose skater touched it last, by claim number, the highest claim winning everywhere and a tie the lower slot (M5) | yes, in the run's prop list |

Nearest-player ownership with hysteresis and migration: M6.

**Props (M5).** A prop is named across consoles by its locator id (the asset's guid). The owner
samples it in the run's prop list (format 2: locator, claim, position, a unit quaternion,
velocity, resting; 38 B) while it moves, for 120 ticks after a touch and every 20th run at
rest; the receiver's object becomes a puppet: its target transform is the owner's sample and its
body is driven there as a hand prop's is, never read back. A touch (the scoring pass's skater
contact, the bench's nudge) claims it. Proof (`../bench/evidence/2026-11-10-m5a-*.log`): the
nudge on seat 1 at its frame 2700 claimed the tutorial's prop (locator `ccf33c75656ac459`, claim
1); seat 2 took it as a puppet at its frame 2231; 44 samples in the next 600 frames while it
moved, 10 per 600 at rest. The puppet's convergence (its distance behind the sample) is traced
every 60th sample [ASSUMPTION: not yet read in a run].

**Story characters (M5):** each console's own, never streamed; the box test is the local
tester's again (M3's test per player released and re-queued an entry every tick while the
skaters stood apart). [ASSUMPTION: the bench's stretch placed no challenge character; the
pedestrians around the flip-trick box are the census's.]

**Puppets' AI (M5):** a puppet perceives no threats, wants nothing, chases and guards nobody;
its graph still animates it (the locomotion from the owner's motion state stays M6's).

**Guards (M5):** the chase of each console's own skater rides its facts (`being_chased`,
`jacked`), every console's scripts read any player's, the local chase messages are the facts'
edges. [ASSUMPTION: the bench reached no chase. `2026-11-10-m5c-*.log`: both seats stood five
minutes at the edge of `NoSkateZone_Area` (x 591..663, z 640..748, the only security area
within 1500 m of the waterfront) and the census spawned no guard; whether the area wants a
skater skating in it, or the security spawn switch, is for play ([open items](../open-items.md)
30).]

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

**Not done:** the puppet's locomotion from the owner's motion state (it walks its own way; [open
items](../open-items.md) 1); delta coding; cars; migration; the population rule for a shared
region (M6); a guard's chase of a remote skater (a remote skater has no actor to chase).

## History

- 2026-11-10 M5: the table grew by story characters, guards and props; the stream by the
  props (format 2); a puppet's AI cut to its animation
  ([research](../../research/RE-mission-world.md)).

- 2026-11-10 M1 (commit `0918b45`): the module, the format, the puppets and the contacts; the
  net message (`a7859ad`, protocol 116). The bench first showed zero pedestrians on both seats:
  the joined online game's flag had turned the census off ([session](session.md)); then the
  career's continue point proved empty of pedestrians in the port itself, so the seats are
  teleported to the waterfront spawn, where the plain free skate fills 12 pedestrians in 20 s.
