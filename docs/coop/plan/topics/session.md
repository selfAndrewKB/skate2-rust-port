# Session: the campaign lobby

**Scope:** how two consoles meet in one campaign, enter the career together and stay in the
game: matchmaking, the lobby, the simulation's flags, inactivity, leaving. Front page:
[README](../../README.md).

## State (2026-11-10, M1) [CONFIRMED on the bench unless marked]

- **The campaign lobby** is a network challenge type of its own, `challenge::CAMPAIGN` (9), at
  one key (`protocol::campaign::KEY`), asked for as `PlayNowMode::Campaign`: the server makes or
  finds exactly one campaign lobby per key, takes up to four players, and never starts a
  challenge in it ("a campaign takes no challenge start"). Protocol 116 (the fork's own number;
  an upstream client is refused, not misparsed).
- **Entering:** `start_online_game` routes a campaign into `start_online_campaign`: Continue
  Career when the profile has one, else the New Career intro, with the online game attached;
  `start_career` then marks the game entered (`online_campaign_entered`), which is what makes
  the port send and show replay-frame puppets, and builds the characters already held from the
  lobby. New Career from the lobby: [UNKNOWN], not run.
- **The simulation keeps its offline flags** in a campaign (slow motion, board pickup, offline
  culling and collision, the living world's census): the joined online game had set the
  game-info online flag, which `IsLWSpawnEnabled` and 28 other sites read. Traced once per game
  ("the simulation keeps its offline flags"). The network game's living-world reset (no census
  spawns, the challenge hull's static objects) is skipped too.
- **Inactivity:** a campaign member may be silent 15 s (`CAMPAIGN_INACTIVITY`) before the kick,
  against 2 s elsewhere: Continue Career reloads dynamic hulls on the game thread, a stall of
  about 160 ticks, and the client's periodic `Busy` cannot go out during a synchronous stall.
- **Diagnostics:** `SKATE2_DIAGNOSTIC_ONLINE=create:9|join:9` from the main menu;
  `SKATE2_TRACE_ONLINE` names the campaign lobby's steps and why the diagnostic waits.

**What M0 proved** (`../bench/evidence` of 2026-11-10; the seats' logs):

```
game 1 created: type 9 (campaign) challenge 636f6f702d63616d
online: campaign lobby: the career starts as Continue Career
career: SignPost loaded (1 completed, 44 available)
online: campaign lobby: entered with the career
online: <other seat> shows
```

**Not done:** a seat leaving and rejoining; the server restarted under a campaign; identity
across sessions (M8, [saves](saves.md)); a person playing it.

## History

- 2026-11-10 M0: the lobby type, the client routing, three bench fixes (commit `a6ebbff`).
- 2026-11-10 M1: the offline flags kept (the census was off: zero pedestrians on both seats until
  the game-info flag was gated), the network-game reset skipped (commits `2b80ee7`, `0918b45`).
