# Kinds: completion rules as data (M7)

**Scope:** the rule per mission family (`Everyone`, `Anyone`, `Combined`, `BestOf`, `Compete`),
the table read from `coop-rules.json`, sent in the campaign's `GameReset`, changed from the
lobby and by `SKATE2_COOP_RULES`. Front page: [README](../../README.md). Design:
[design](../design.md) §3.4, §7.

## State (2026-11-10, M7) [ASSUMPTION: built and tested in units; the bench run is next]

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
- **The AI photographer and cameraman** are each console's, as every AI is; both players skate
  [ASSUMPTION: no photo or film mission reached on the bench].
- **Not done:** the lobby's settings page for the table (the file and the switch suffice on the
  bench; a page is play's to ask for).

**M5 gave the `Security` and `Service` kinds their world:** a chase is any player's (the
facts), a service bought by anyone is everyone's ([prompts](prompts.md), [world](world.md)).

## History

- 2026-11-10 M7: the kinds applied; the families named (open item 11 retired).
