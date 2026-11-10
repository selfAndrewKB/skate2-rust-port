# RE: the objective system with one group per player (read 2026-11-10, M2)

What the port's objective system (`crates/skate2-core/src/challenge/objectives.rs`,
`objective_natives.rs`, `natives.rs`) does when a mission's objectives exist once per player,
as the `Everyone` rule needs, and what had to change. Found on the bench with
`m_1a_01_goto_slappys`, the opening go-to tutorial (eight objectives: push exit, minimap exit,
duck exit, a push/turn step, stairs exit, an ollie landing, a prop volume, a back entrance).

## How the system is built

- A group is per `(activity, player)` (`add_objective_challenge`, :525); `Objectives.
  MakeObjectives(playerID)` makes the activity's group for the id the script passes, which is
  `Online.GetLocalPlayerID()`: `world.local_player`, a different player on each console.
- `objectives::update` (:1371) evaluates every group whose player has a frame in
  `world.skaters` each tick (`continuous_standard`), so per-player groups evaluate as long as
  every player's facts are in the frames.
- The natives read the objective's player's `PlayerState` (`objective_natives.rs:26-30`), from
  `world.player_states` when the round keeps every player's, else the local `world.player`.
- A trigger-volume objective (`ObjectiveTrigger.EnteredVolume(id, name)`) is true when the
  volume is in the objective's `trigger_volumes`. Those are filled two ways: by the
  `Event::TriggerCollision { player }` handler (`receive`, :1450 → `trigger_enter`, :1784),
  which pushes an entered volume into every *active* objective of that player's groups; and by
  `take_colliding_triggers` (:817) at an objective's activation (modes 3 and 4), from the
  player state's `colliding_triggers`. Activation mode 1 and 2 objectives first need their
  `ScoringBoundary` entered; the tutorial's are mode 0.
- `reset_objective_data` (:733) clears `trigger_volumes`; `Objectives.ResetObjective(group,
  index, resetCumulative, resetStatus)` reaches it from scripts. The tutorial's first objective
  is reset by its script every tick while the player has not pushed and turned.

## What broke with two groups, and the fixes

1. **The per-player state was a tick stale.** `career_step` built `player_states` from the
   collisions kept from the tick before (`previous`) and refreshed only the local
   `world.player` with the tick's own collisions inside the trigger loop. The evaluation an
   enter event triggers in the same tick therefore never saw the volume in a per-player state.
   Fix: the trigger loop refreshes the matching per-player state too
   (`selected_session_career.rs`, the trigger loop).
2. **Script group operations reached one copy.** `MakeObjectives` returns one handle (the first
   player's group, the same on every console, so the script run stays identical) and the
   script's `ResetObjective`, `ResetObjectives`, `ForceObjectiveComplete`, `FailAllObjectiveStatus`,
   `SetEvaluationStatus`, `ChangeObjectivePointsRequirement`, `SetFractionNumerator`,
   `AddPointsObjective` and `DeleteObjectiveGroup` acted on that one group. The first player's
   objective was reset every tick (its entered volumes dropped) while the second player's
   never was: the asymmetry showed as "entered volume" traces for player 1 only. Fix: under an
   `Everyone` rule those natives apply to every group of the activity
   (`natives.rs`, `objective_groups`).
3. **Completion is decided once.** `on_objective_completed` (:1215) sends
   `cMsgAllObjectiveComplete` for the group that completed; with one group per player the
   first completion would advance the script. Fix: the rule at that point: `Everyone` sends it
   when every player's group is complete, naming the first player's group; `Anyone` sends it
   at the first and retires the others (`evaluate = false`).

## Measured

On the bench (2026-11-10, two seats, the round started by the server at tick 0 on both): the
mission executed at round tick 900 on both; each player's first three objectives completed as
that player's skater entered the push exit, the minimap exit and the duck exit, the traces
identical on both consoles tick for tick (`coop: trigger <id> entered by player P: groups [G]`
then `objective <key> of player P: Active -> Complete`).

## Still one player's

The HUD's objective display and the trigger-tester volume ids traced as `0x...03e38706` are
the challenge group's local ids; `world.player` keeps feeding the objective natives of any
player without a per-player state. The tutorials' `eChallengeTypes` family number is still
unknown, so the built-in rule table names them by prefix (`m_1a_`).
