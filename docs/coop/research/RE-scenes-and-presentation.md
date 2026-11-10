# RE: scenes, speech, movies and the presentation director in a campaign (read 2026-11-10, M4)

What the port does with a mission's cutscenes (NIS), speech lines and movies when the career runs
in an online round, and what a campaign with one objective group per player did to it. Found on
the bench with `m_1a_01_goto_slappys` (the opening go-to tutorial, whose go-to exits each play a
`Goto_..._outro_2` scene and whose end phase plays speech, a `Goto_..._outro_3` event and a
highlight replay). Code: `crates/skate2-core/src/nis/manager.rs`, `session/online/
online_challenge.rs`, `session/selected_session_career.rs`, `challenge/presentation.rs`,
`challenge/objectives.rs`.

## The port's online rule for scenes [CONFIRMED by reading, then on the bench]

- `NisSystem::set_online_game(true)` (`Sk8::Net::IsInOnlineGame`): a scene's load waits neither
  for the simulation's pause nor for the stream focus (`UpdateLoadState` states 1, 2 and 6). The
  campaign round sets it when it starts (`start_campaign_round`).
- In a round the scene manager ticks on round ticks (`update_online_round_nis`, called once per
  processed tick before the career step), so a scene the scripts play runs the same number of
  ticks on every console.
- **Ends are notes on the facts.** A scene's end (`cMsgNISChallengeNotify` 1, with
  `cMsgNISComplete`), the photo-view and killable notifications, and a speech line's end
  (`cMsgSpeechEnded`) do not reach the scripts where they happen: `CareerSession::present` holds
  them (`held_presentation`) while `hold_presentation` is on, the round logs them numbered
  (`Round::log_presentation`: `Presented::Nis { seq }`, `Presented::Speech { character, id, seq
  }`), the pacing leader (the lowest-numbered contributing player, `presentation_leader`) puts the
  undelivered ones on its next facts (`announce_presentation`, `FactsNotes::presented`, at most
  15 per tick), and every console, the leader's included, delivers them to the scripts at the
  tick those facts are processed (`deliver_presented`, dedup by sequence). A console that is not
  the leader keeps what it saw until a tick delivers it; an end only it saw (a line the leader's
  audio cut short) is never delivered and stays in its log.
- The campaign round uses that path unchanged. M4 added the movie's end (`cMsgFMVFinishedPlaying`,
  `Presented::Fmv { seq }`, wire byte 0xf0; protocol 119): the port posted it locally where the
  movie ended, which in a campaign would have run the scripts' `FMVFinishedPlaying` handlers at
  different ticks on each console. [ASSUMPTION: no movie plays in the bench's stretch; proven by
  the same code path as the scene's end.]
- A skip (`skip_nis(false)`, the A press of the front end's NIS state) is refused in a campaign
  (`set_skip_blocked(true)`): a skipped scene would end on one console at a tick the others never
  see. A forced skip (the results screen's leave) still ends the scene.
- The scene's characters come from `SceneCharacterSource::local_player` (the local recipe) and
  `local_index`; the go-to outros instance no characters at all (`instanced 0 characters`), so
  "each console shows its own skater" is by reading only [ASSUMPTION], to be seen in a story
  scene.

## The presentation director [CONFIRMED on the bench]

- `ChallengePresentation.AddPresentationGroup` makes a group per call; `StartPresentation` plays
  it at once if none is active, else queues it, at most 8 deep: a ninth start is dropped
  (`presentation::start`). Each event either continues (`played on`) or waits for its end
  (`SPEECH`, `NIS`, `WAIT`, `FADE`, `SET_DYNAMIC_HULL_STATE`); `cMsgPresentationEventGroupFinished`
  names the finished group, and the opening tutorial's `play_pres_event` state waits for its
  `OutroPresGroup`'s.
- **Every objective owns presentation groups** (`objectives::presentation_group`): an activation
  group (the go-to's `mini_teleport` icon, pin, the CameraMan line, the tutorial callout), a
  completion group (the outro scene, the end lines, a highlight replay), failure and hint groups,
  started from `set_status` and the failure and hint evaluation. With one objective group per
  player (M2) every such group existed twice and both copies played: the go-to's pin and speech
  twice, the outro scene twice (ticks 1604 and 2145 in `2026-11-10-m4-seat-*.log`), the end
  lines twice; the doubled starts filled the director's queue and the script's own outro group
  was dropped, so `m_1a_01_goto_slappys` never reached `challenge_cleanup`
  (`2026-11-10-m4b-seat-*.log`, groups 23, 79, 26, 82, 45, 52 played in turn and the script's
  group never finished).
- The fix (M4): an objective's activation presentation plays from the first copy to activate, its
  completion presentation from the last copy under `Everyone` (the first under any other rule),
  the same decision as the message the script hears (`presentation_plays`, mirroring
  `script_hears`); failure and hint presentations play from the local player's copy only
  (`feedback_plays`). `presentation::start` traces a start it cannot queue.

## What else the bench taught

- An objective's `cMsgObjectiveActivate` and `cMsgObjectiveFailed` come every tick from the go-to
  script, which resets its objectives each tick; the activation message was doubling too and is
  now heard once per tick (`script_hears`); the failure is heard from every copy (no state to
  count), naming the first player's group.
- A replayed run on a profile the previous run had saved (the sign-up accepted, the hull states
  the outros set) takes other paths in the end phase (`2026-11-10-m4-seat-*.log` loaded
  `PJ01_SchoolPool` at a different time than M3's): the bench now starts every run from the
  seat's `SKATER.P.primed` (`tools/coop/bench.sh`).
- With one presentation per activity the mission's end phase really waits for its outro
  (`m4c`: `challenge_cleanup` at 4238, M3's at 3512 with the outro unpresented), so a bench
  recipe's sign-up teleports must come after it; and `COOP_EXECUTE` is a seat's own switch (a
  mission a diagnostic starts is not a note), so a run with it on one seat ran the mission there
  alone while the other seat idled in the same round and still took the sign-up as a note.
- The two seats' campaign saves of every run since M3 hashed differently; `cmp` of the two
  `SKATER.P` and the dump tool's `Position` row showed one leaf, `cSignPost::Position`, the
  skater's own orientation, differing in its floats' low bits (each seat's own physics at the
  autosave). M2's agreement was the identical teleports'. The leaf is each console's, not the
  career's, and stays out of the blob and the hash now.
