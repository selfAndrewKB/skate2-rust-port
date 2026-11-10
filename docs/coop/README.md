# Co-op campaign for Skate 2: the plan

**Goal:** the whole Skate 2 campaign played together over the port's online multiplayer, from
the tutorial to the ender, by two or more players each on their own machine: one shared career
held by the dedicated server, the open world synchronized wherever either player can see, no
role co-op, tutorials proven by everyone, each player's own skater in the scenes. Built on the
port's netcode, never beside it ([rules](plan/rules.md)).

**Build:** `coop` branch of `selfAndrewKB/skate2-rust-port`, **`coop-v0.9.0-whole-campaign`**
(2026-11-10): M8, the whole campaign. The progression lives on the server and survives its
restart; a console joining a running campaign brings every other member onto the newest save
and the round starts again with everyone; a console whose connection drops keeps its career
and rejoins by its player id, trying every three seconds; the mission crawl runs the campaign
mission by mission on the bench. The crawl's first slice passed six of the eight opening
missions; the two photo missions stop where the port itself stops (AI skaters not ported).
Reviewed (six findings, all applied). The milestone pages: [saves](plan/topics/saves.md),
[kinds](plan/topics/kinds.md), [tooling](plan/tooling.md), the ledger in
[versions](plan/versions.md).

## Current focus (2026-11-10)

Play: the M8 build in the player's hands (every milestone so far was proven on the bench,
M5's build alone was played); what the audits say in play (the puppet's speed, open item 39);
the crawl beyond the opening missions once the port's AI skaters land (open item 45); Training
Facility travel ([session](plan/topics/session.md)); the open items a person's play decides
(New Career in a lobby, 41; a rejoin notice, 44).

## What's next

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
