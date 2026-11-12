# World: the shared living world (route D)

**Scope:** who owns each pedestrian, story character, prop, guard and car, how the owner
streams it, how the others drive puppets, how a contact reaches the owner, and what it costs.
Front page: [README](../../README.md). Design: [design](../design.md) §3.6. Research:
[pedestrians](../../research/RE-living-world-pedestrians.md),
[transport](../../research/RE-frame-transport.md),
[the mission-required world](../../research/RE-mission-world.md),
[the city: population, puppets, handoff, coding](../../research/RE-city-ownership-and-puppets.md).

## State (2026-11-10, M6) [CONFIRMED on the bench unless marked]

**What M6 proved** (`../bench/evidence/2026-11-10-m6a-*.log`, both seats at the waterfront,
seat 2 skating 70 m east at round tick 2400 and back at 4200): six pedestrians seat 2's census
had spawned near seat 1 were handed over as seat 2 left ("that player is nearer by 15 m for 60
ticks (epoch 1)" on seat 2; "handed to this console: its player is nearer" and "promoted: entity
… runs its own graph" on seat 1 at its ticks 3055, 3484, 4279 and 4297); none came back when
seat 2 returned to 6 m (the margin holds); every puppet spawned with its owner's model key
(the trace names it). The shared street held 18 pedestrians between the two censuses where
M1's held 24 (12 own and 12 puppets per seat). The stream's cost: about 555 B/s per owned
pedestrian, 6.6 KB/s a seat with twelve owned, 1.9 KB/s with four.

**The first synchronization audit** (`tools/coop/sync_audit.py` over
`../bench/evidence/2026-11-10-m6b-*.log`: the M6 recipe, then two hours idle with the census
dump on; 6981 dumps, 135 053 entity pairs): no pair where both seats claimed the same role;
model identical in every pair; position within 0.5 m in 99.95 % (worst 2.58 m, at a handoff);
down within 99.8 %; flags 97.9 %; yaw within 5° in 95.3 % (worst 142°, a puppet mid-turn);
speed within 0.5 m/s in 71.7 % (the puppet's speed is its drive's, not the owner's: the
first thing the audit says to improve, [open items](../open-items.md) 39). The graph behaviour
differs in 99.5 % of pairs and the moods in 41 %, by design (not shared).

**M6's mechanisms:**

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
  at 6.6 KB/s a seat with twelve owned pedestrians the budget holds without it, so it waits
  for a reason: [open items](../open-items.md) 33); cars and AI skaters; the puppet's stumble
  and mood gestures.

## State (2026-11-10, M5)

**Ownership** (`crates/skate2-core/src/coop/ownership.rs`): one rule per kind, one funnel that
traces every decision (entity, old owner, new owner, reason). The table:

| Kind | Rule | Streamed |
| --- | --- | --- |
| ambient pedestrian | `Spawner`: the census that spawned it, for life (M1) | yes |
| story character (a pedestrian a challenge placed) | `Spawner`: the console whose tester box spawned it, for life, never handed over by distance; two testers reaching the entry at once keep the lower slot's copy (M14; `EachConsole` from M5 to M13) | yes, as a pedestrian, wherever the players are, its descriptor naming the challenge's entry |
| security guard | `Chasee`: the console of the player it chases, which is its spawner until M6's migration (a guard chases only the skater of the console that simulates it) (M5) | yes, as a pedestrian |
| ambient AI skater | `Spawner`: the console whose spawner spawned it, near its own skater; each console spawns its share of the original's three, split by player index (M12) | yes, as an owned skater's frames and look (protocol 124, [ai-skaters](ai-skaters.md)) |
| prop (a dynamic object with a locator) | `LastToucher`: the console whose skater, or whose car or pedestrian (M11.2), touched it last, by claim number, the highest claim winning everywhere and a tie the lower slot (M5); a touch waits while the owner's latest sample still moves it (M10.1) | yes, in the run's prop list |

Nearest-player ownership with hysteresis and migration: M6.

**Story characters (2026-11-12, M14).** A pedestrian a challenge placed (the photographer, a
sign-up's pro, a sponsor at a poster) was every console's own from M5: each console's tester
box spawned its copy and ran its AI, so the field test saw the photographer in two places
(open item 70: the host's from the intro scene's placement, the joiner's own 23 s later at the
locator). Now it is its spawner's and travels as a pedestrian does (`Kind::StoryCharacter`,
stream format 4): the descriptor carries the challenge entry it stands for (the activity's
key, the entry's character index), and the receiver's character manager holds the puppet in
that entry (`characters::remote_story_spawned`), so its own tester spawns no second copy and
the challenge's culls and state flags reach the puppet as they reach its own. When both testers
reach an entry in the same instant each spawns its own, each receives the other's descriptor,
and the tie rule keeps the lower slot's copy: the higher culls its own (or the spawn in flight,
`yielded`) and takes the puppet (`remote_story_offered`); the lower refuses the higher's until
its despawn (`CoopWorld.refused`). A story character streams whatever the distance (a
mission's character is wanted on every console) and is never handed over by distance; its
puppet going (its owner despawned or stopped streaming it) frees the entry
(`remote_story_gone`) for the local tester. The id clash of item 73 ends with it: an owned
story character is named by its owner's slot as every streamed pedestrian is. What stays each
console's own: the intro scenes' own characters (`nis::characters`, each console's scene).

**Props (M5).** A prop is named across consoles by its locator id (the asset's guid). The owner
samples it in the run's prop list (format 2: locator, claim, position, a unit quaternion,
velocity, resting; 38 B) while it moves, for 120 ticks after a touch and every 20th run at
rest; the receiver's object becomes a puppet: its target transform is the owner's sample and its
body is driven there as a hand prop's is, never read back. A touch (the scoring pass's skater
contact, the bench's nudge) claims it. The touch is any skater body's on this console: the
human's, and on the AI's owner the pro's (M10; the M10 S.K.A.T.E. logs show the pro's hits
claiming two rails on seat 1, puppets on seat 2), each recorded by player bit (`touched_by`,
in the claim's trace). A car's or a pedestrian's moving push is a touch too (M11.2: the
scoring pass's living-world contacts of a vehicle's or a pedestrian's body with a relative
velocity, flagged `WORLD_TOUCH` in the trace; a prop resting against another claims nothing),
so a prop one of those moves is claimed for the console that simulates the pusher and
streamed, and a puppet pedestrian's push on the other console contests it under the rule
below (open item 67). Proof (`../bench/evidence/2026-11-10-m5a-*.log`): the nudge on seat 1 at its frame
2700 claimed the tutorial's prop (locator `ccf33c75656ac459`, claim 1); seat 2 took it as a
puppet at its frame 2231; 44 samples in the next 600 frames while it moved, 10 per 600 at
rest. **Demoted (2026-11-11):** the 44 samples were the sender's count; the receiver passed a
sample only when its claim beat the known one (`claim_wins`, false for the owner's own equal
claim), so from M5 to M10 a puppet prop took the first sample of each claim and no other, the
sample of the touch's own tick, before the prop had moved: a prop knocked down on the owner's
screen stood on the other (the player's sighting, open item 67). Every convergence trace in
every kept log is "sample 1" (62 lines). Since M10.1 the owner's continuing samples are the
puppet's; read in `../bench/evidence/probe/2026-11-11-m10-1-props.md` run 5: samples 1, 61 and
121 applied 180 ticks apart, the puppet 0.14, 0.11 and 0.09 m behind, the prop settling 1.4 m
lower than it stood; and on the tagged build's S.K.A.T.E. crawl (`2026-11-11-crawl-m10-1`) the
rails the pro hit (claimed by seat 1 with `players 0b100`, the leader's bit) followed on seat 2
through samples 61, 121 and 181 at 0.11, 0.01 and 0.00 m behind.

**A contested prop (M10.1).** Two consoles' actors leaning on one prop claimed it back and
forth: each touch on the non-owner took it with the next claim number, the owner's next touch
took it back, and each copy was a puppet one tick and a body the next (the M9 S.K.A.T.E.,
each console's own pro: twelve claims of one rail in forty frames, a puppet 4.7 m behind; a
prop down on one screen and standing on the other, open item 67). Now a touch does not take a
prop whose owner's latest sample, within 30 ticks (`CONTEST_TICKS`; samples come every third
tick while a prop moves), still has it moving: the touch waits, traced once per contest, the
puppet stays driven, and the next touch after the prop rests claims it. A sample that raises
the claim counts as moving until the owner's next sample (the touch's own tick samples the
velocity before the physics step reads it back, so that sample says resting). Two touches in
the same tick on two consoles fall to the tie rule as before (the lower slot). Proof
(`../bench/evidence/probe/2026-11-11-m10-1-props.md`, six runs): seat 1's nudge at round
tick 2400 claimed the tutorial's prop; seat 2's nudge at 2430 waited ("while Remote(0) is
moving it", its sample 3 ticks old); seat 2's nudge after 3600, the prop at rest, took it with
claim 2 and seat 1 became the puppet.

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
seat 2 at 4805 (five ticks); the puppet up again with the owner's 420 ticks later ("up again" is
the owner's flag: the local copy itself never got up, see below).

**A puppet stands up (2026-11-11).** It did not before: once knocked down, by its owner's flags
or by this console's prediction, a puppet read `down` in every census until it was culled (M6's
logs, the crawl's demo mission). Two causes, read in the code and seen in a trace of the colliding
state (`SKATE2_TRACE_COOP_PUPPET=1`): a puppet's graph was ticked only while
`required_to_begin_colliding || colliding` held, so when the colliding behaviour completed and
cleared `colliding` the graph never ticked again and the exit that runs the behaviour's `End`
(which clears `flags[1] & 4`, releases the monitored intent packet, pops head tracking) never
happened; and the un-released packets clogged the intent queue, so later collisions' packets were
rejected (`NotAdded`) and their animations never played. Now the graph is ticked one tick past
the collision (`PuppetDrive::was_colliding`), so the exit runs; and the owner's word ends a
knockdown this copy still plays: when the owner's sample says up while the copy lies down, with
no knockdown contact awaiting confirmation (or one older than 60 ticks, a refusal), the collision
is closed as its own completion closes it and `End` follows on the next tick. A contact is awaited
only when it is a knockdown (a standing one has no flag to confirm it). Proof: the demo mission's
census `down` agreement 33 % → 99.3 % (`../bench/evidence/2026-11-11-crawl-demo-standup.md`);
a diagnostic shove at the waterfront, predicted at tick 2400, confirmed by the owner at 2405, the
owner up at 2825 and the puppet up at 2825, `down` agreeing in 2176 of 2178 pairs
(`../bench/evidence/probe/2026-11-11-shove-standup-*.log`). Still apart in the demo: flags agree in
59 % of pairs while each seat's skater stands inside the other's spectators and hits them every
three ticks (open item 50).

**Measured (2026-11-10, two seats on one Mac, the waterfront):** seat 1 owned 9-11 pedestrians
in range of 12 in its roster and sent 3.4-5.3 KB/s; seat 2 held 9-10 puppets, owned 2-3 of its
own and sent 0.7-1.6 KB/s. Logs: `../bench/evidence/2026-11-10-m1-seat-{1,2}.log`.

**Not done:** the puppet's locomotion from the owner's motion state (it walks its own way; [open
items](../open-items.md) 1); delta coding; cars; migration; the population rule for a shared
region (M6); a guard's chase of a remote skater (a remote skater has no actor to chase).

**Puppets play their owner's state (2026-11-12, M16).** Until M16 a puppet's animation was
its intents from the owner's velocity (walk, run, stand) and its own graph only while it
collided, and a hit by the local skater was predicted here (a knockdown played at once and was
stood up again if the owner's flags never confirmed it: 63 corrections against 38
confirmations on one seat of the field test). Now every sample carries the state the owner's
AI graph is in (`EntityState::graph`, stream format 5, two bytes), and the puppet's graph is
held at that state every tick (`PedestrianGraph::update_delegate_held`: the exited and entered
states' behaviours end and begin as on the owner, without a transition or its hooks), so what
the owner plays, the puppet plays: a startle, a stumble, a knockdown's ragdoll direction, a
photographer's pose. A state the puppet cannot enter here (a behaviour wanting what a puppet
has not) is traced once and that puppet keeps the M6 gate. A hit by the local skater is no
longer predicted: the contact goes to the owner, and the owner's flags (knocked down, or a
standing hit with its direction) and state play it back here one round trip later. The facing
turns to each sample's over the cadence (three ticks) instead of snapping and holding.

**The stream sends on error, and every channel is metered (2026-11-12, M17).** An owned
pedestrian's sample went out every run (every third tick) whatever it said: about 27 bytes a
sample, 570 B/s a pedestrian, 6 kB/s a seat in the field test (open item 75). Now the owner
sends a pedestrian's sample only when the receiver's extrapolation of the last one (its
position carried on at its velocity, `PuppetDrive::step`) would be off by more than 5 cm or
its facing by more than 2°, or its flags, motion or graph state changed, or a descriptor is
due, or 20 runs have passed (the keepalive, one second); the puppet coasts at its velocity
for 72 ticks before it stands, past the keepalive. A sample the run's byte budget cut goes
next run. The stream trace counts the samples withheld. Every channel's bytes are counted
where the messages are encoded (`skate2_net::client::ChannelBytes`: the player's frames, the
owned skaters' frames, the world runs, the facts, the looks, the rest) and traced every ten
seconds as `net bytes at <tick>`, so the ledger names what each costs. Not built here: delta
coding against acknowledged runs (open item 33) and the looks' compression (item 75), which
wait on the measure of this build.

## History

- 2026-11-10 M5: the table grew by story characters, guards and props; the stream by the
  props (format 2); a puppet's AI cut to its animation
  ([research](../../research/RE-mission-world.md)).

- 2026-11-10 M1 (commit `0918b45`): the module, the format, the puppets and the contacts; the
  net message (`a7859ad`, protocol 116). The bench first showed zero pedestrians on both seats:
  the joined online game's flag had turned the census off ([session](session.md)); then the
  career's continue point proved empty of pedestrians in the port itself, so the seats are
  teleported to the waterfront spawn, where the plain free skate fills 12 pedestrians in 20 s.
