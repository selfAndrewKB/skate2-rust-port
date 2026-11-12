# The AI day, 2026-11-11

**For:** the read after the overnight report. The morning's instruction was to take the open
list on, AI skaters first. This is what the AI skaters turned out to be, what was changed, what
the bench then measured, and what is still open. Front page: [README](../../README.md); the
ledger of states: [versions](../versions.md); every mission: [missions](../missions.md); the
items: [open items](../open-items.md) 45 to 49 and 55 to 58.

## The finding in one paragraph

The port had the AI skaters all along. The 19 "port: AI skater" stops and the 8 "seats diverged"
stops of the crawl were four co-op faults, found one after another with a decision trace
(`SKATE2_TRACE_AI`) on the bench: a campaign round told the AI skater manager it was in a
network game, where the original never fields challenge AI, so no mission ever picked its
characters (55); the scripts count their AI three different ways and a two-seat campaign made
the same index mean different skaters on the two consoles (56); the campaign dropped the
`cMsgSkaterCreated` the follow family waits for (57); and a mission hull's own locators register
a few frames after the crawl's execute, when the follow scripts have already asked for them
(58). Each fix is one mechanism in one place; the crawl's driver now preloads a mission's
assets and wins it after its own running state instead of at a fixed tick.

## What changed (commits since `coop-v0.9.2-puppet-standup`)

- **The AI skater manager runs as the career's in a campaign** (`load_game(false)` on both
  paths into a campaign). Characters are selected at the asset load, their recipes stream, the
  spawn finds them.
- **The scripts see the career's own index space** ([design](../design.md) §7): on every
  console `Online.GetLocalPlayerID()` is 0 and `GetNumHumanSkaters()` 1; every player a script
  hands a native goes through `challenge::players::script_slot` (0 → this console's slot, `k` →
  the AI's slot `N - 1 + k` after the N humans). The AI natives, pins, repositions, speech,
  stance, name, race and trick natives are routed; the results rows, the objectives' per-player
  fan-out and Party Play's `SetPlayer` are not (they are the rules' or display's).
- **The AI skaters exist for the scripts**: this console's AI skaters' frames join the facts
  frames; the online players back end answers for the AI rows; their teleport, pin and
  camera requests pass the online filter; their `cMsgSkaterCreated`/`Destroyed` reach the
  scripts with the AI's ordinal.
- **The bench**: `SKATE2_DIAGNOSTIC_COOP_PRELOAD` (the loading screen's asset load, 300 ticks
  before the execute), `SKATE2_DIAGNOSTIC_COOP_WIN=running+300` (the win after each family's
  running state: `challenge_running`, `run_heat`, `run_round`, `round_setup`), a teleport's
  fallback to the world's location table, and the traces: the manager's decisions and asset
  states, the location arenas and their locators.
- **The ledger**: a new cause, "in step: the mission runs on", for a mission both seats ran
  alike through the win and that only play can end.

## What the bench measured

`../bench/evidence/2026-11-11-crawl-ai2.md` (six missions, the fixed build): the photo
`m_5a_01_photo_thrasher` passes; the follows `m_4b_01_pro_navarrette_1` and
`m_1a_07_follow_slappy`, the S.K.A.T.E. `m_4a_01_skate_carroll_koston` and the demos
`m_7a_01_demo_dyrdek` and `m_6p_01_demo_first_job` run identical state sequences at identical
round ticks on both seats (8, 8, 53, 14, 14 states; no line differs) and stop where only play
ends them: the heat (the pro's path outlasts the deadline), the rounds (the AI leaders fail in
turn, the bench sets no trick), the end-options screen (a demo lost by a bench that does no
tricks).

`../bench/evidence/2026-11-11-crawl-ai3.md`: the 26 other missions the crawl had stopped at,
rerun on the fixed build: 11 pass, 15 stop, every stop at the same state at the same tick on
both seats (one, the jam, left its asset wait a tick apart and ran the rest alike). The ledger
stands at **94 of 115** (from 83): the 21 that stop are 14 in step at a state only play ends, 5
with no competitor on the crawl's save (the sponsor team and the dynamic lists are chosen in
play, 59), one downloadable-content mission (60), one character the data does not name (61).
`../bench/evidence/2026-11-11-crawl-ai4.md`: the six probe missions again on the build with the
review's fixes, the milestone's proof.

## Found, recorded, not fixed

- Two consoles' own copies of one pro: each console drives its AI on the same path from the
  same data; whether the copies drift enough to matter is play's to say (45, 53).
- S.K.A.T.E. and the contests in a campaign: each console's own AI leaders, each console's own
  rounds; the competing kinds need either shared AI or a shared outcome ([kinds](../topics/kinds.md)).
- The demo's start spots apart (50); the first-job demo ran in step this time, the 58-tick
  split of 2026-11-10 not seen (49), one run.
- The results screen's rows and the objectives' per-player fan-out keep the simulation's
  indexes (56, "not routed").

## The afternoon: the competing kinds (M9)

The competing kinds (S.K.A.T.E., jams, contests) read their AI's outcome, and that is each
console's own, so their scripts part where a leader fails its trick (open item 62). The answer
chosen is the ledger's "the one who did it" kind, as the prompts: a script's decision about its
own player (an objective forced complete, the objectives all failed, the mission won or lost on
the script's own terms) is not applied where it is made; it travels as an outcome note on that
player's facts (protocol 122) and every console applies it to that player's copy at the note's
tick; the rules settle the mission once per attempt. A completion that follows the shared
objective completion still applies at once, as before. Proven on the bench with the diagnostic
win on seat 1 alone: both seats applied the notes and entered the mission's cleanup at the same
tick. The review found the first discriminator wrong (a retired copy would have routed the
shared completion as a note and completed the mission twice on one console); it is the rule's
threshold now. Two pre-existing gaps are recorded (63, 64). What is not shared is the AI's run
itself: a S.K.A.T.E. still plays its rounds once per console; the first human to win ends it
for both.

## The evening: one owner for the AI skaters (M10)

The residue of the afternoon was two pros: each console drove its own copy, and a S.K.A.T.E.
read its own leader. M10 makes the challenge AI skaters one console's, the first player's
(`CampaignWorld::ai_owner`). The owner runs them as the career does. Its facts carry their
completed outputs at their slots (`Facts.owned`, protocol 123), delivered with its own frame at
the tick on every console, so every script reads one AI run. An AI slot's first frame in a
tick's frames is `cMsgSkaterCreated` to the scripts and its last `cMsgSkaterDestroyed`, the same
tick everywhere, the owner's included; the local challenge queue's skater messages are dropped
in a campaign. The AI's body reaches the other consoles as the remote players' do: its look once
per spawn (`OwnedRecipe`), its replay frames in runs (`OwnedFrames`, relayed under a marked slot,
acknowledged by slot), a `RemoteSkater` puppet under a marked user id (`protocol::owned`). The
non-owner takes each spawn's row and slot and builds no actor; a script's camera switch to the AI
follows the puppet there, with the owner's camera riding the AI's frames while the owner follows
it ([ai-skaters](../topics/ai-skaters.md)).

**The first probe** (`../bench/evidence/probe/2026-11-11-crawl-m10-probe.md`, the S.K.A.T.E.,
the follow `m_4b_01` and the photo `m_5a_01`): seat 2 built no actor ("is its owner's: no actor
here"), built each AI's character from seat 1's look in 12 to 15 ms and showed it; both seats
posted each AI's creation at the same round tick (2134, 2381, 1474); the puppet played seat 1's
pro at pace 1.000, four to five ticks behind, never held at its newest sample. The follow and
the photo ran in step as before. The S.K.A.T.E. stopped seat 2 at `round_setup`: the script's
`SetLocalIndex` to the leader's slot found no actor there ("Missing local party skater"), the
fix above.

**The review** found twelve things, nine applied before the proof build: the streamed flag
leaking past a kicked or quit campaign into the next offline career (every later mission would
have spawned no pro); slot 8 accepted by the owned messages and refused by the frame relay's
reader (one bound now, the manager's nine rows); the mission's end converting the pro to an
ambient skater on the owner alone, so the two consoles' scripts parted on the AI's destruction
(a campaign unspawns it everywhere); a teleporting tick sent without a pose sample; the look
keyed by the actor's address (now the recipe, and sent again at the slot's creation); the
client's receivers of a leaver's owned streams kept; a destroyed skater's unsent frames
prepended to the next spawn's run; the owner's standing ticks dropping the AI (the AI stands
with its owner now); a silent truncation past eight streams (traced, the coders unstepped).
Recorded, not fixed: the server keeps a round's owned looks for joiners (a stale one builds a
puppet that never shows), and the owner's scripts now read the AI's shared frame (position at
the tester, no gesture), as the other consoles do.

**The proof** (`../bench/evidence/probe/2026-11-11-crawl-m10-skate-{1,2,3}.md`, the review
build): the S.K.A.T.E. three times, 57 states each, every sequence identical on both seats
through `leader_failure` at the same tick, the state where the AI day's runs had parted once
in three (62); seat 2 followed each leader's puppet at the script's camera switch. The follow
and the demo (`-follow-demo.md`, four owned skaters, their creation at tick 1891 on both seats)
in step to the heat and the end-options screen as before. A look goes out twice per spawn (at
the first frame, and again when the slot's creation reaches the owner: 31 KB each), which the
next build can halve. The nine-mission regression on the tagged build:
`../bench/evidence/2026-11-11-crawl-m10.md` [see the run note in missions].

## The night: a contested prop (M10.1)

The player's note after M10: a railing an AI knocks down on one instance stands on the other,
and the AI there runs through it. The first reading, that an AI's hit claims nothing, was wrong
on the logs: the scoring pass marks a prop touched for any skater body's contact, and the M10
S.K.A.T.E. runs show the pro's hits claiming two rails on seat 1 during the leader's run (the
crawl's human idle), puppets on seat 2 within 0.04 m. The M9 S.K.A.T.E. logs, each console's
own pro, show the sighting's mechanism instead: one rail changed owner twelve times in forty
frames, every touch on the non-owner taking it back with the next claim number, the other
rail's puppet 4.7 m behind its sample; a prop down on one screen and standing on the other.
M10 ends that for the challenge AI (two claims, no flip in three runs). M10.1 ends it for the
rest, two humans or each console's own ambient skaters: a touch does not take a prop whose
owner's latest sample, within 30 ticks, still has it moving; the touch waits, the puppet stays
driven, the next touch after it rests claims it. The probe's fourth run then found the plain
cause of the sighting, older than any of this: since M5 the receiver passed a prop sample only
when its claim beat the known one, which the owner's own continuing samples never do, so a
puppet prop took the first sample of each claim and no other, the sample of the touch's own
tick, before the prop had moved; the owner's copy fell, the puppet stood. Every convergence
trace in every kept log reads "sample 1" (62 lines); the M5 proof's 44 samples were the
sender's count, demoted in the version ledger. The owner's continuing samples are the puppet's
now. The claim's trace names the touching players (`touched_by`, recorded at the contact, since
the scoring flags are cleared before the claim runs), the nudge takes a list of ticks and round
ticks, and every bench seat is muted (`SKATE2_DIAGNOSTIC_MUTE`, the mix and its clock running).
Proof (`../bench/evidence/probe/2026-11-11-m10-1-props.md`, six runs, two of them failing a
criterion and each fixing a rule, the sixth on the tagged build): run 5, seat 1's nudge at round tick 2400 claimed the
tutorial's prop, seat 2's at 2430 waited on a sample three ticks old, seat 2's after 3600 took
it with claim 2, and seat 1 as the puppet took samples 1, 61 and 121 at 0.14, 0.11 and 0.09 m
behind while the prop settled 1.4 m lower. The review (six findings) moved the "not resting"
of a touch's own tick to the sender, reset the wait counter when a prop changes hands, sorted
the nudge ticks and pinned the equal-claim tie in a test; the regression crawl of the
S.K.A.T.E. and the follow on the tagged build (`../bench/evidence/2026-11-11-crawl-m10-1.md`)
[see the run note in missions]. What stays: a car's or a pedestrian's push claims nothing
(67).

## Late: the hub's hull (M10.2)

Open item 68, seat 2 reloading the hub's hull once more than seat 1 after two wins, read on the
M10 run's kept logs: `ur_hotel_pool_drain` is a location global, run while a tester's box is in
its bounds and terminated when every tester's box has left, and the testers were every state
in `player_states`, built from every frame of the tick, which since the AI day carry this
console's own ambient skaters. An ambient skater wandering into the pool's bounds on one seat
ran the global there, and its leaving terminated it and reloaded the hull there alone; the
sewer film's hull cycled the same way, three times on seat 2 in the pool demo and never on
seat 1. The original's box tests are the local tester's. `player_states` now holds the
campaign's players alone, so every box test, the Hall of Meat check and the objectives' lookups
read the humans, and the location globals' run and termination are traced. Proof
(`../bench/evidence/probe/2026-11-11-crawl-m10-2-probe.md`): the two missions again, the
global's run and suspension at the same points on both seats with two testers each time, no
sewer hull on either seat, both missions passing. The same traces named the "AI skaters" whose
creation ticks had differed between the seats: each console's own ambient skaters, posted to
the scripts as AI players at that console's own ticks and slots (open item 69); the owned
challenge AI's creation stays the same tick everywhere (no owned slot was streamed in either
mission). The regression of the nine missions on this build: `2026-11-11-crawl-m10-2` [see the
run note in missions].

## Later still: the ambient skaters out of the scripts (M10.3)

Open item 69, found by the hull probe: each console's own ambient skaters were frames of the
scripts' in a campaign, so their creation and destruction reached the scripts as AI players at
each console's own ticks and slots (three per seat in the flip-tricks tutorial, 945/1005/1065
on one seat and 986/1046/1106 on the other, their respawns at different slots), and a script
that counted or addressed AI players by ordinal could have read an ambient skater on one
console and the pro on the other. The round's frames are now the humans' and the owned
challenge AI's alone; the ambient population is counted in a trace. Proof
(`../bench/evidence/probe/2026-11-11-crawl-m10-3-probe.md`): the flip-tricks tutorial with no
AI creation on either seat, the pool demo, and the S.K.A.T.E. with the owned slots read in the
facts of tick 2133 on both seats and the pro's creation posted at 2134 on both, the M10 claim
now read in the facts themselves; all three in step. The nine-mission regression on this build:
`2026-11-11-crawl-m10-3` [see the run note in missions].

## Last: the AI's hand-off (M11)

Open item 65: the AI skaters' owner was the first player and never changed, so its leaving
ended the AI for everyone. Now the owner is the lowest player still in the round, which shrinks
by a leaver at the tick every console delivers the leave (item 64 with it: the rules count the
players left, and a new `seats` keeps the slot boundary). Until the next owner's facts carry
the AI, the round keeps the leaver's last owned frames in the tick's frames, 600 ticks at most,
so no scripts see a destruction; the new owner replays the spawn commands it had dropped as the
old owner's, each at the last frame's position, drops its puppets and streams the skaters as
the old owner did. A bench switch makes a seat quit at a round tick. Proof
(`../bench/evidence/probe/2026-11-11-m11-handoff.md`): the S.K.A.T.E. with seat 1 quitting at
round tick 2700; seat 2 delivered the leave at 2821 with the two pros standing, owned them from
2822, spawned Koston and Carroll at their last frames, carried them in its own facts the same
tick, sent their looks, and played four more rounds with the camera switching to each leader;
the scripts heard no creation or destruction across the hand-off. The AI's routine starts over
on the new owner. The nine-mission regression on this build: `2026-11-11-crawl-m11` [see the
run note in missions].

## Next

Play on this build: a follow, a photo, a S.K.A.T.E. and a demo with two people, the one pro on
both screens (silent on the second, 66). The owner leaving (65). The demo's start spots. The
go-tos' arrival triggers in play (46).
