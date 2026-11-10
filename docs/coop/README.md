# Co-op campaign (fork notes)

This fork (`selfAndrewKB/skate2-rust-port`) adds co-op campaign play on top of the port's
online multiplayer. Work happens on a Mac (Apple silicon) and a Windows PC; this folder is the
shared project knowledge for both. Upstream's rules in `AGENTS.md` still apply to all code.

## Documents

- `design.md`: the analysis of the port's netcode, career and living world, the paradigms
  considered, the recommended architecture, method, milestones and the open decisions.
- `bench.md`: the two-instance bench (`tools/coop/bench.sh`), the campaign lobby and what each
  milestone proved on it, starting with M0.
- `setup-macos.md`, `setup-windows.md`: per-machine notes.

## Branches

- `main` tracks `upstream/main` (skate6743). Keep it clean; never commit co-op work to it.
- `coop` is the long-lived project branch. These docs and the root `CLAUDE.md` exist only here.
- Fixes worth sending upstream branch from `main`, not from `coop`, so the PR carries nothing
  co-op-specific.

Remotes: `origin` = the fork, `upstream` = skate6743/skate2-rust-port.

## Game data

Each machine keeps its own extracted retail Xbox 360 copy (`default.xex` + `data/`) outside
this repo and points the game at it with `SKATE2_GAME_ROOT` or `config.json`'s `game_root`.
Per-machine notes: `setup-macos.md`, `setup-windows.md`.

## Running together

- Server: `./skate2-server [ip:port]`, or `./skate2-pc --server [ip:port]` from the game.
- Clients only meet a server built with the same build secret. Both PCs must build from the
  same commit of this fork with the same `release-secret.txt`. Without that file each build makes its own random secret,
  so two separate builds cannot meet. See `../release/auto-update.md`.
- Controllers only; keyboard skating is not implemented.

## What does not travel between machines

Claude Code auto-memory is keyed by the project's absolute path and stays local. Anything
durable goes in this folder.
