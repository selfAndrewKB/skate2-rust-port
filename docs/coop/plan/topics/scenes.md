# Scenes: NIS, FMV and speech (M4)

**Scope:** cutscenes, movies and speech lines under the online rule (no pause, no skip, the same
ticks on every console, their ends delivered from the pacing leader), each console showing its
own skater; a mission's presentation played once per activity. Front page:
[README](../../README.md). Design: [design](../design.md) §3.5. Research:
[scenes and the presentation director](../../research/RE-scenes-and-presentation.md).

## State (2026-11-10, M4) [CONFIRMED on the bench unless marked]

- **The online rule is the port's.** A campaign round sets `career.nis.set_online_game(true)`:
  a scene's load waits neither for the simulation's pause nor for the stream focus, and the
  scene manager ticks on round ticks (`update_online_round_nis`), so a scene the scripts play
  starts at the same round tick on every console and runs the same number of ticks.
- **Ends are notes on the facts.** A scene's end, a speech line's end and (M4) a movie's end do
  not reach the scripts where they happen: `CareerSession::present` holds them, the round logs
  them numbered (`Presented::Nis`, `Presented::Speech`, `Presented::Fmv`; protocol 119), the
  pacing leader (the lowest-numbered contributing player) puts the undelivered ones on its next
  facts, and every console, the leader's included, delivers them to the scripts at the tick those
  facts are processed. On the bench that is about 120 ticks after the end. A console that is not
  the leader keeps what it saw until a tick delivers it; an end only it saw is never delivered
  ([open items](../open-items.md) 20).
- **No skip in a campaign.** `skip_nis(false)` (the A press of the NIS state) is refused while
  the round runs (`set_skip_blocked`); a forced skip (the results screen's leave) still ends the
  scene. [ASSUMPTION: converted, no press reached it on the bench.]
- **A movie's end** travels as `Presented::Fmv` and becomes `Event::FmvFinishedPlaying` at the
  note's tick. [ASSUMPTION: no movie plays in the bench's stretch; the same path as a scene's
  end.]
- **Each console shows its own skater** by the scene's character source (`local_player`, the
  local recipe). [ASSUMPTION: the go-to outros the bench reaches instance no player character
  (`instanced 0 characters`; the welcome-back scene instances 4 characters, none a player);
  to be seen in a story scene.]
- **One presentation per activity.** Every objective owns activation, completion, failure and
  hint presentation groups; with one objective group per player (M2) each existed and played
  twice, and the doubled starts filled the director's 8-deep queue so the script's own outro
  group was dropped and the opening tutorial never reached `challenge_cleanup`. Now an
  objective's activation presentation plays from the first copy to activate and its completion
  presentation from the last copy under `Everyone` (the first under any other rule), the same
  decision as the message the script hears (`presentation_plays`, `script_hears`); failure and
  hint feedback plays from the local player's copy only (`feedback_plays`). The trace names
  every decision (`the Complete presentation of objective 7 from player 1: 2 of 2 copies: plays`)
  and every start the director cannot queue.
- **What M4 proved** (`../bench/evidence/2026-11-10-m4d-*.log`, the opening tutorial by M3's
  recipe with the sign-up teleports after the mission's end phase): the three outro scenes
  started at the same round ticks on both seats (`Goto_m_1a_01_goto_slappys_outro_2` at 1713,
  `_outro_3` at 2625, `_outro_4` at 3257); their ends were seen by the leader at 2133, 2954 and
  3901 and delivered to the scripts on both seats at 2254, 3075 and 4022; seven speech ends were
  delivered on both at 1097, 1713, 2625, 3283, 4423, 4555 and 6979; every objective's
  presentation played once (`1 of 2 copies: plays` for activations, `2 of 2 copies: plays` for
  completions); the end phase waited for its outro and both seats entered `challenge_cleanup` at
  4233 and autosaved at 4234; then M3's sign-up and accept started the flip-tricks tutorial on
  both at 6652. The starts the director dropped in that run are the bench's doing
  ([open items](../open-items.md) 21).

**Not done:** a story scene with the players' skaters in it; a movie; a skip press; whether the
NIS state's "skip" prompt still shows while refused [UNKNOWN]; the results screen's replay
(`REPLAY` events play locally, their ends not notes) [UNKNOWN].

## History

- 2026-11-10 M4 (`2026-11-10-m4-*.log`): scenes already in step (the outro at 1604 on both, its
  end delivered on both at 2144), but every presentation played twice and the mission never
  reached `challenge_cleanup`; traced first to the run continuing from the previous run's
  autosave (the bench now restores `SKATER.P.primed` every start), then (`m4b`) to the
  per-copy presentation groups overflowing the director's queue, so the script's outro group
  start was lost. With one presentation per activity (`m4c`) the mission completed at 4238,
  but the recipe's sign-up teleport fell inside the mission (its end phase now waits for the
  outro, about 700 ticks longer than M3's run, whose outro was unpresented). A run with the
  mission switches on one seat only ran the mission there alone (not kept). `m4d` is the proof.
  The two seats' campaign saves disagreed by the skater's own orientation leaf alone
  ([progression](progression.md) History).
