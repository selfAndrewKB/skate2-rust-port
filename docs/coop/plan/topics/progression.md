# Progression: one career on every console (M2)

**Scope:** the progression machines, challenges, unlocks, cash and statistics running
identically on every seat from shared facts, the autosave identical on both, the server holding
the blob. Front page: [README](../../README.md). Design: [design](../design.md) §3.2, §3.3, §3.7.
Research: [the objective system per player](../../research/RE-objectives-per-player.md).

## State (2026-11-10, M2) [CONFIRMED on the bench unless marked]

- **The campaign round.** The server starts a campaign's round once every member's career is
  loaded and there are two or more members (`GameReset(CAMPAIGN)`, kind 3, with the seed and the
  table by slot; protocol 117). Until then a loaded career holds: no script update, no event
  flushed (the load's outputs, its hulls, applied), so every console starts the scripts at the
  same tick 0 on the same facts. From the start the career is the port's facts round with scope
  "everything", every player pacing it, the players back end the table's, the career's random
  stream the seed's, scenes under the online rule. The career stays the career: single-player
  mode, progression on, autosaves. The server relays a campaign's facts to every member.
- **Rules as data** (`crates/skate2-core/src/coop/rules.rs`): `coop-rules.json` beside the saves
  (or `SKATE2_COOP_RULES=<file>`), else the built-in table: the opening tutorials (`m_1a_*`)
  everyone's, the rest anyone's; by key, name prefix, family, default; its hash traced at the
  round start on every console. The tutorials' family number is not yet known.
- **The ledger's "everyone" kind in the objective system.** Under an `Everyone` rule
  `Objectives.MakeObjectives` makes one group per player, each evaluated from that player's facts
  on every console; the script holds the first player's group (the same on every console) and its
  group operations (reset, force complete, fail, evaluation status, points, delete) reach every
  copy; `on_objective_completed` asks the rule at the one evaluation point: `Everyone` completes
  the activity when every player's copy is complete, `Anyone` at the first (the other copies
  retired). A per-player state takes the tick's own trigger collisions (it was a tick stale).
- **The save.** Each autosave sends the progression savables (SignPost's three leaves, the
  unlockables, the dynamic hulls) to the server with their hash and the round tick; the server
  keeps the newest beside itself (`campaign-<key>.save`), logs whether every member's hash
  agrees, and hands it to a member joining later (adopting it is M8's).

**What M2 proved** (`../bench/evidence/2026-11-10-m2-*.log`): the round started on both seats at
tick 0 with seed `d4407991`; `m_1a_01_goto_slappys` executed at round tick 900 on both; each
player's push exit, minimap exit and duck exit completed as that player's skater entered them
(each at the round tick of its own stop, the traces identical on both consoles tick for tick); the remaining steps completed at the same round ticks on both (the bench's same-tick
completion); "player 0 completed its objectives; 1 of 2 players complete under Everyone: waiting
for the others", then "2 of 2: the activity completes" at round tick 3000; `show_results` at 3100,
`complete` at 3515; both consoles autosaved at round tick 3516 the same 130378 bytes, hash
`4fd8652c017853b8`; the server: "Seat 2 agrees (tick 3516)".

**Bench stand-ins:** the steps a teleport cannot prove (buffer, stairs, ollie, a moved prop, the
session marker, meeting Slappy) were completed by `SKATE2_DIAGNOSTIC_COOP_COMPLETE` at the same
round tick on every console; the Start State Machine's "New to skate?" dialog does not come
back on Continue with a mission done, so the bench did not pass through it (its answer is M3's).

**Not done:** a member joining a running campaign (M8); the HUD showing one objective list
(both copies may show); statistics stay each player's own backend.

## History

- 2026-11-10 M2 (commits `1917acc` server, `da0e21d` client, then the fixes): the round would
  not tick until the server relayed facts outside a challenge phase; the hold leaked queued
  events through `flush` (one seat's global activities started early); the go-to objectives
  completed for one player only until the per-player state took the tick's collisions and the
  script's group operations fanned out ([research](../../research/RE-objectives-per-player.md)).
