# Co-op campaign for Skate 2: the plan

**Goal:** the whole Skate 2 campaign played together over the port's online multiplayer, from
the tutorial to the ender, by two or more players each on their own machine: one shared career
held by the dedicated server, the open world synchronized wherever either player can see, no
role co-op, tutorials proven by everyone, each player's own skater in the scenes. Built on the
port's netcode, never beside it ([rules](plan/rules.md)).

**Build:** `coop` branch of `selfAndrewKB/skate2-rust-port`, **`coop-v0.17.0-puppets-and-bytes`**
(2026-11-12): M12, the ambient skaters on both screens: every console streams its own as
puppets to the others, the owned skaters' names carrying their owner (protocol 124), each
console spawning its share of the allowance (open item 53); M11.2, a car's or a pedestrian's
push claims the prop it moves for its console (67); M11.1, the owned pro's puppet sounds (its frames carry the actor's audio
physics record) and its look goes once per spawn (66); M11 (2026-11-11), the AI skaters pass
to the next player when their owner leaves (standing at their last frames until the new owner's
facts carry them, open item 65), and the campaign's players shrink by a leaver so the rules
count those left (64). Before them, M10.3, each
console's own ambient skaters are no frame of the scripts' (their creation had reached the
scripts as AI players at each console's own ticks, open item 69); M10.2, the box tests read the campaign's players alone (an ambient skater had run and reloaded
a location global's hull on one seat, open item 68); and M10.1, a puppet prop
follows its owner's samples (it had taken the first of each claim and no other since M5: the
player's railing standing on one screen) while a touch does not take a prop its owner is still
moving ([world](plan/topics/world.md)). Before them,
`coop-v0.12.0-owned-ai` (2026-11-11): M10, the AI skaters are one console's: the first
player's facts carry its challenge AI skaters' outputs, every console's scripts read one AI
run, the other consoles show puppets of its replay frames
([ai-skaters](plan/topics/ai-skaters.md), [design](plan/design.md) §7), and
`coop-v0.11.0-outcome-notes` (2026-11-11): M9, the competing kinds: a script's
decision about its own player travels as a note on its facts and the rules settle the mission
(a win on one seat completes it on both under `Anyone`), and `coop-v0.10.0-ai-skaters`
(2026-11-11): the AI families run in a campaign. The AI skaters were never the port's blocker:
a campaign told the AI manager it was in a network game, the scripts count their AI three ways
and a two-seat round made one index mean two skaters, the AI's creation message was dropped, and
a hull's locators registered after the crawl's spawn ([open items](plan/open-items.md) 55 to 58).
Each console now runs its own AI skaters, the scripts see the career's own index space
([design](plan/design.md) §7), and the crawl stands at 94 of 115: the 21 that stop run in
step to a state only play ends (14), or lack a competitor on the crawl's save (5), the DLC (1),
a character name (1). Before it, M8 (`coop-v0.9.2`): the whole campaign, its ledger, and the
puppets standing up again. The progression lives on the server and
survives its restart; a console joining a running campaign brings every other member onto the
newest save and the round starts again with everyone; a console whose connection drops keeps
its career and rejoins by its player id, trying every three seconds. The mission crawl ran every
one of the campaign's 115 missions on the bench (2026-11-10: 83 pass, 32 stop); every
mission's standing, cause, synchronization numbers and census audit are
in [missions](plan/missions.md), regenerated from the kept logs. Reviewed (twelve findings, all applied: the complete-by-name
diagnostic now reports whether it completed anything, the ledger refuses to run without the
career database, and ten lesser ones). The milestone
pages: [saves](plan/topics/saves.md), [kinds](plan/topics/kinds.md), [tooling](plan/tooling.md),
the ledger in [versions](plan/versions.md).

## Current focus (2026-11-12)

Play: the field test of 2026-11-12 (the lobby on the M12 build, the Burnette photo "A Little
Help Here") found the photographer in two places and the joiner's objectives dead after the
host's first bail; the day's five milestones answer it ([versions](plan/versions.md), M13 to
M17): a human's bail fails and re-arms that human's copy alone on both consoles (M13), the
story characters are one console's and a puppet elsewhere (M14), the joiner's completion wins
the mission for both (M15), the puppets play their owner's animation state and take no hit
of their own (M16), and the stream sends only what the other console cannot extrapolate, every
channel metered (M17). What play should judge on the next build: a photo mission with the
joiner doing the trick; the photographer standing the same on both screens (and on the third,
if three); a pedestrian run into by either skater, which should fall on both screens a
round trip later and get up with its owner; a bail by each player; the `net bytes` lines in
the seat logs on a real link. Still each console's own: the script's state and vars (76), the
intro scenes' characters, the ambient skaters' looks at 31 kB a respawn (75), the demo's start
spots (50), the sponsor team's save (59), Training Facility travel; the open items a person's
play decides (New Career in a lobby, 41; a rejoin notice, 44).

## What's next

Play on the `coop-v0.17.0` lobby build (both machines on it: protocol 125, stream format 5).
The AI day's report says what changed and what it
measured, its evening section the owned AI:
[reports/2026-11-11-ai-skaters.md](plan/reports/2026-11-11-ai-skaters.md); the night before it:
[reports/2026-11-11-overnight.md](plan/reports/2026-11-11-overnight.md).
Play, and what it finds. Milestones and their done-when: [design](plan/design.md) §5; M0 to M8 are
tagged ([versions](plan/versions.md)).

## Documents

- [plan/rules.md](plan/rules.md): the requirements, the working rules, the decisions.
- [plan/design.md](plan/design.md): the analysis and decision record (the port today, the
  routes, the architecture in the port's terms, the method, milestones, risks).
- [plan/versions.md](plan/versions.md): every tagged state and its proof.
- [plan/tooling.md](plan/tooling.md): the bench, the switches, building; per-machine notes in
  [setup-macos.md](plan/setup-macos.md) and [setup-windows.md](plan/setup-windows.md).
- [plan/qa-matrix.md](plan/qa-matrix.md): seats × machines × situations, exercised or not.
- [plan/reports/2026-11-11-overnight.md](plan/reports/2026-11-11-overnight.md): the overnight
  report, 2026-11-10 to 11: the crawl recorded, the puppets' stand-up fixed, what is next.
- [plan/reports/2026-11-11-ai-skaters.md](plan/reports/2026-11-11-ai-skaters.md): the AI day:
  what the AI skaters turned out to be, the four fixes, the bench's measurements, what is open.
- [plan/missions.md](plan/missions.md): the mission ledger, every campaign mission's standing in
  co-op from the crawl's logs, regenerated after every crawl; the details per mission in
  [plan/missions-detail.md](plan/missions-detail.md).
- [plan/sync-inventory.md](plan/sync-inventory.md): every field of a pedestrian's state, streamed,
  derived or local, generated from the code; measured by `tools/coop/sync_audit.py`.
- [plan/open-items.md](plan/open-items.md): seen and not yet fixed or explained.
- Topics, each with Scope, State and History: [session](plan/topics/session.md),
  [world](plan/topics/world.md), [progression](plan/topics/progression.md),
  [prompts](plan/topics/prompts.md), [scenes](plan/topics/scenes.md), [kinds](plan/topics/kinds.md),
  [saves](plan/topics/saves.md).
- Research, one note per reading of the port: [research/](research/).
- Evidence (logs and captures that prove a milestone) lives beside the repository in
  `../bench/evidence/`, dated, never in it.

## Branches

- `main` tracks `upstream/main` (skate6743). Keep it clean; never commit co-op work to it.
- `coop` is the long-lived project branch. These docs and the root `CLAUDE.md` exist only here.
  Milestones are tagged `coop-vX.Y.Z-name`.
- Fixes worth sending upstream branch from `main`, not from `coop`, so the PR carries nothing
  co-op-specific.

Remotes: `origin` = the fork, `upstream` = skate6743/skate2-rust-port.

## Game data

Each machine keeps its own extracted retail Xbox 360 copy (`default.xex` + `data/`) outside
this repo and points the game at it with `SKATE2_GAME_ROOT` or `config.json`'s `game_root`.

## Running together

- Server: `./skate2-server [ip:port]`, or `./skate2-pc --server [ip:port]` from the game.
- Clients only meet a server built with the same build secret. Both PCs must build from the
  same commit of this fork with the same `release-secret.txt`; without that file each build
  makes its own random secret. See `../release/auto-update.md`.
- Controllers only; keyboard skating is not implemented.

## What does not travel between machines

Claude Code auto-memory is keyed by the project's absolute path and stays local. Anything
durable goes in this folder.
