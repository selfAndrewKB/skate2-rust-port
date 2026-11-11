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

## Next

Play on this build: a follow mission, a photo mission and a demo with two people, watching the
pro and the AI's copies. The competing kinds' design (S.K.A.T.E., jams, contests). The demo's
start spots. The go-tos' arrival triggers in play (46).
