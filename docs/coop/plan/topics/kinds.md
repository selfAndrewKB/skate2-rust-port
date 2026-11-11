# Kinds: completion rules as data (M7)

**Scope:** the rule per mission family (`Everyone`, `Anyone`, `Combined`, `BestOf`, `Compete`),
the table read from `coop-rules.json`, sent in the campaign's `GameReset`, changed from the
lobby and by `SKATE2_COOP_RULES`. Front page: [README](../../README.md). Design:
[design](../design.md) §3.4, §7.

## State (2026-11-10, M7) [CONFIRMED on the bench for `Anyone` and `Everyone`; the rest by unit test]

- **Every kind decides at the one evaluation point** (`coop/rules.rs`, `Rule::completes`,
  `Rule::fails`, `Rule::points_combined`; `challenge/objectives.rs`): `Everyone` and
  `Compete { every_human_must_win: true }` complete an activity when every player's copy of
  its objectives is complete, and fail it at the first copy's failure; `Anyone`, `BestOf`,
  `Combined` and `Compete { .. false }` complete it at the first copy's completion (the other
  copies retired) and fail it only when every copy has failed (each copy remembers its reported
  failure until its objective activates again); under `Combined` every copy's points objective
  reads the sum of the copies' points (`GetAccumulatedPoints`), so the players' points add up
  toward one requirement and the copies complete together. The script hears each message
  once, naming the first player's group, by the same decision.
- **The built-in table names the families** (`eChallengeTypes`, [design](../design.md) §7):
  tutorials (0) and security (17) `Everyone`; the contests and races (1, 3, 10-13, 15, 16,
  18-21, 26, 27) `Compete { .. false }`; best trick (8), Own the Spot (9) and Hall of Meat (31,
  33) `BestOf`; the rest `Anyone`; the opening missions `m_1a_*` `Everyone` by prefix. A
  `coop-rules.json` beside the saves or `SKATE2_COOP_RULES=<file>` overrides it by challenge
  key, name prefix or family; each console reads its own file and traces the table's hash at
  the round start, so a difference shows there (sending the creator's table in the
  campaign's `GameReset`, as the design intends, is owed: [open items](../open-items.md) 37).
- **`BestOf` beyond completion:** the best run's score on the results screen and the
  leaderboard stay each console's own display [open item].
- **The AI skaters are each console's own (2026-11-11, the AI day)** [CONFIRMED on the bench,
  six missions]: the original's challenge AI is offline-only, so a campaign's AI skater manager
  runs as the career's on every console; each console selects the mission's characters at its
  asset load, streams their recipes, and spawns and drives its own copies (the pro you follow,
  the photographer's subject, a demo's judges and competitors, S.K.A.T.E.'s leaders). The
  scripts see the career's own index space (themselves 0, their AI 1 on; [design](../design.md)
  §7), so a script's AI player lands on the same slot after the humans on every console, and its
  `cMsgSkaterCreated` arrives with the ordinal the script expects. Measured
  (`2026-11-11-crawl-ai2`): the photo `m_5a_01`, the follows `m_4b_01` and `m_1a_07`, the
  S.K.A.T.E. `m_4a_01`, the demos `m_7a_01` and `m_6p_01` ran identical state sequences at
  identical ticks on both seats (8 to 53 states each, no difference). What differs between the
  consoles is the AI's own skating: two copies of one pro, each following the path on its own
  console, not streamed (open items 45, 53); whether that shows in play is play's to say. The
  photo missions pass; the follows run their heat, the demos their rounds to the end-options
  screen, S.K.A.T.E. its rounds, all past the diagnostic win: the bench does no tricks, so those
  families' own ends (the pro's path, the demo's score, a human's set trick) are not reached on
  it [UNKNOWN in play].
- **The families in the data.** The campaign's `challenge_global_data` uses `ChallengeType`
  0 (tutorial, 9 missions), 2, 4, 6 (photo, 5), 15 (jam, 3), 23 (follow, 7), 30 (go-to, 13),
  38 (30 photo missions), 39 (11 film), 40 (20 spot film), 41 (4 pro film), 43 (7 film/demo
  team) and 45 (the two Training Facility teleports); 38 to 43 are not named in `rules::family`
  (the enumeration past 34 is unread) and fall to the table's default, `Anyone`, which is what
  the design gives photo and film anyway ([missions](../missions.md) names them from the
  missions, marked †). A rule for those families by number works today; naming them is owed.
- **The families in the crawl** (2026-11-10, [missions](../missions.md)): every tutorial (9),
  go-to (13), property teleport (2), film pool team (11) and the film-team kind (7, which
  completes on execute) passed; photo (team) 21 of 30, spot film 15 of 20, pro film 1 of 4,
  S.K.A.T.E. 1 of 2, jam 1 of 3 passed; every follow (7), photo (5) and demo (2) stopped. Every
  stop but the eight `check_ai` divergences and one out-of-step demo is the port's AI skater or
  a locator ([open items](../open-items.md) 45, 47 to 52).
- **Not done:** the lobby's settings page for the table (the file and the switch suffice on the
  bench; a page is play's to ask for).

**M5 gave the `Security` and `Service` kinds their world:** a chase is any player's (the
facts), a service bought by anyone is everyone's ([prompts](prompts.md), [world](world.md)).

## History

- 2026-11-10 M7: the kinds applied; the families named (open item 11 retired). The first
  proof run (`../bench/evidence/2026-11-10-m7a-*.log`, the opening tutorial under a rules file
  making it `Anyone`'s) split the round: under any rule but `Everyone` the objective system
  still made one copy per console, for its own player, so seat 1's player completed its
  objectives and ended the mission alone ("1 of 1 players complete under Anyone") while seat 2
  stayed in it. Every rule now makes one copy per player (`natives.rs`,
  `make_objectives_helper`); the rule decides only at the evaluation point.
