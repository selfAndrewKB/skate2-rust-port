# The versions

**Scope:** the ledger of every tagged state of the `coop` branch: its tag, commit, what it
proved and how far (the bench, or play). Front page: [README](../README.md). The current
version is the README's Build line; each new one is prepended here.

| Tag | Commit | Date | What | Proof |
| --- | --- | --- | --- | --- |
| `coop-v0.3.0-shared-progression` | see `git log` | 2026-11-10 | M2: the campaign round (server-started facts lockstep over the whole career), rules as data, one objective group per player under `Everyone`, the script's group operations fanned out, the progression save on the server with hash agreement | [CONFIRMED on the bench] the opening tutorial completed with each player entering its go-to volumes and the activity completing only when both had; both consoles autosaved the same 130378 B (hash `4fd8652c017853b8`); the server logged the agreement. Logs: `../bench/evidence/2026-11-10-m2-*.log`. Not yet played by the player. |
| `coop-v0.2.0-world-stream` | `0918b45` | 2026-11-10 | M1: ownership rule (spawner), world stream format 1, puppet pedestrians, contacts to the owner, offline flags kept in a campaign, coop teleport and shove diagnostics | [CONFIRMED on the bench] two seats on one Mac at the waterfront: 11 pedestrians streamed at ~500 B/s each, 10 puppets on the other seat, a shove's knockdown confirmed by the owner 5 ticks later, the puppet up again with the owner's. Logs: `../bench/evidence/2026-11-10-m1-*.log`. Not yet played by the player. |
| `coop-v0.1.0-campaign-lobby` | `6945b84` | 2026-11-10 | M0: the bench script, the campaign lobby (challenge type 9, protocol 115), Continue Career with the online game attached, three bench fixes | [CONFIRMED on the bench] two seats in one lobby, each in its own career with progression on, both skaters visible. Not yet played by the player. |

Earlier commits on `coop` (docs, AGENTS rules, setup notes) carry no tag.
