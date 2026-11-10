# Co-op campaign for Skate 2: the plan

**Goal:** the whole Skate 2 campaign played together over the port's online multiplayer, from
the tutorial to the ender, by two or more players each on their own machine: one shared career
held by the dedicated server, the open world synchronized wherever either player can see, no
role co-op, tutorials proven by everyone, each player's own skater in the scenes. Built on the
port's netcode, never beside it ([rules](plan/rules.md)).

**Build:** `coop` branch of `selfAndrewKB/skate2-rust-port`, **`coop-v0.4.0-prompts-as-notes`**
(2026-11-10): M3, prompts and sign-up. A pad's answer on one console (a dialog, a sign-up press,
a poster's accept) rides that player's facts as a note and every console applies it at the same
tick; one seat's sign-up opened the poster on both and the other's accept started the next
tutorial on both ([prompts](plan/topics/prompts.md)). Over `coop-v0.3.0-shared-progression` (M2,
the campaign round and identical saves, [progression](plan/topics/progression.md)),
`coop-v0.2.0-world-stream` (M1, puppet pedestrians, [world](plan/topics/world.md)) and
`coop-v0.1.0-campaign-lobby` (M0, the bench and the lobby, [session](plan/topics/session.md)).
None has been played by a person yet; every claim is the bench's ([versions](plan/versions.md)).

## Current focus (2026-11-10)

M4, scenes in step: cutscenes, FMVs and speech under the online rule (no pause, no skip, the same
ticks on every console, their ends delivered from the pacing leader), each console showing its
own skater ([scenes](plan/topics/scenes.md)). A first sighting: the "welcome back" cutscene after
the opening tutorial already ran under that rule on both seats in M3's bench.

## What's next

M4 scenes in step → M5 the mission-required world
(story characters, props, guards) → M6 the city (ownership by nearest, migration, delta coding,
population) → M7 kinds → M8 the whole campaign, resume and reconnect. Milestones and their
done-when: [design](plan/design.md) §5.

## Documents

- [plan/rules.md](plan/rules.md): the requirements, the working rules, the decisions.
- [plan/design.md](plan/design.md): the analysis and decision record (the port today, the
  routes, the architecture in the port's terms, the method, milestones, risks).
- [plan/versions.md](plan/versions.md): every tagged state and its proof.
- [plan/tooling.md](plan/tooling.md): the bench, the switches, building; per-machine notes in
  [setup-macos.md](plan/setup-macos.md) and [setup-windows.md](plan/setup-windows.md).
- [plan/qa-matrix.md](plan/qa-matrix.md): seats × machines × situations, exercised or not.
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
