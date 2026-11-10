# Co-op campaign for Skate 2: the plan

**Goal:** the whole Skate 2 campaign played together over the port's online multiplayer, from
the tutorial to the ender, by two or more players each on their own machine: one shared career
held by the dedicated server, the open world synchronized wherever either player can see, no
role co-op, tutorials proven by everyone, each player's own skater in the scenes. Built on the
port's netcode, never beside it ([rules](plan/rules.md)).

**Build:** `coop` branch of `selfAndrewKB/skate2-rust-port`, **`coop-v0.8.0-kinds`**
(2026-11-10): M7, kinds. Every completion rule decides at the objective system's one
evaluation point, with one copy of the objectives per player under every rule; the built-in
table names the families; and synchronization is measured, not assumed: an inventory of every
pedestrian field generated from the code, a census dump, an offline audit, and a live audit in
every build ([kinds](plan/topics/kinds.md), [sync-inventory](plan/sync-inventory.md)). The
first milestone reviewed before its tag. Over `coop-v0.7.0-city` (M6, ownership by the nearer
player, puppets as the owner's copy, [world](plan/topics/world.md)),
`coop-v0.6.0-mission-world` (M5, story characters, props, guards and services,
[world](plan/topics/world.md), [prompts](plan/topics/prompts.md)), `coop-v0.5.0-scenes-in-step`
(M4, scenes under the online rule, [scenes](plan/topics/scenes.md)),
`coop-v0.4.0-prompts-as-notes` (M3, a pad's answer as a note, [prompts](plan/topics/prompts.md)),
`coop-v0.3.0-shared-progression` (M2, the campaign round and the shared save,
[progression](plan/topics/progression.md)), `coop-v0.2.0-world-stream` (M1, puppet pedestrians,
[world](plan/topics/world.md)) and `coop-v0.1.0-campaign-lobby` (M0, the bench and the lobby,
[session](plan/topics/session.md)). None has been played by a person yet; every claim is the
bench's ([versions](plan/versions.md)).

## Current focus (2026-11-10)

M8, the whole campaign: the mission crawl from the tutorial to the ender on two consoles,
resume from the server's blob, a member joining a running campaign, reconnect, Training
Facility travel ([saves](plan/topics/saves.md), [session](plan/topics/session.md)); and what
the audits say in play (the puppet's speed, open item 39).

## What's next

M8 the whole campaign, then play. Milestones and their done-when: [design](plan/design.md) §5.

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
