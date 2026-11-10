# Rules and decisions

**Scope:** the player's requirements, the working rules, the decisions made and open. Front page:
[README](../README.md). The analysis the decisions rest on: [design record](design.md).

## State (2026-11-10)

**The requirements (the player's, 2026-11-10; fixed, [design](design.md) §0):** the open world is
synchronized wherever either player can see (pedestrians, security, cars, props); progression is
shared and may be held by the dedicated server; tutorials are proven by everyone; no role co-op;
per-kind completion rules allowed but started basic and tuned in play; cutscenes show each player's
own skater, never both.

**In force:**

- **Build on the port's netcode, never beside it** (`AGENTS.md`, "Co-op fork rules"). New needs
  become new facts groups, notes, messages and barriers added the documented way; never a second
  transport; the server simulates nothing.
- **Route D, distributed ownership** (2026-11-10, after route A was set aside the same day;
  [design](design.md) §2). One owner per living-world entity, the owner streams it, the others
  drive puppets, contacts go to the owner, the obvious outcome is predicted locally.
- **Clean code, logging alongside** (the player, 2026-11-10): every mechanism is written with its
  trace after careful static analysis, compiled into every build (`SKATE2_TRACE_COOP`,
  `SKATE2_TRACE_ONLINE`, `SKATE2_TRACE_CAREER`), never blind. Every decision passes one funnel
  that names it (ownership: entity, old owner, new owner, reason).
- **Rules are data, swappable without a rebuild** (the player, 2026-11-10): completion rules, the
  ledger's kinds, the ownership table each live in one place.
- **Seats, not "player 2"**: state indexed by seat, designed for N, played with two.
- **Claims with their proof layer**: static reading, pure tests, headless career tests, the
  two-instance bench, field play. A claim is promoted only as far as its layer reaches. Markers in
  these docs: [CONFIRMED] seen on the bench or in play; [ASSUMPTION] inferred; [UNKNOWN] not
  looked at.
- **Commits as the work goes** (the player, 2026-11-10: "commit as you go along"), each finished
  step its own commit on `coop`, pushed to the fork; no attribution lines of any kind (the player,
  2026-11-09). A milestone is tagged `coop-vX.Y.Z-name` ([versions](versions.md)).
- **No images in the repository** (the player, 2026-11-10). Captures and logs that prove a
  milestone live in `../bench/evidence/` beside the repository, dated, and are named by path.
- **Upstream hygiene**: `main` tracks upstream; co-op work lives on `coop`; a fix worth sending
  upstream branches from `main` ([README](../README.md), Branches).
- **The method** of the `split-screen-coop` skill ([design](design.md) §4): four parts per
  feature (pure rule, one adapter at the engine lever, presentation that only draws, one
  coordinator), the ledger as the backlog, a mission crawl as the regression.

**Decided:**

- One shared career, held by the server and mirrored on both consoles (2026-11-10).
- Completion rules start at tutorials `Everyone`, everything else `Anyone`, tuned in play
  ([design](design.md) §7).
- A campaign keeps the simulation's offline flags (slow motion, board pickup, offline culling and
  collision, the living world's census); only the netcode is online (2026-11-10, M1; the
  online-game flag had switched the census off: [world](topics/world.md) History).
- A campaign's scenes load as the career's until M4 puts them under the online rule
  ([scenes](topics/scenes.md)).
- M1's ownership rule is "the census that spawned it owns it"; nearest-player ownership with
  hysteresis and migration is M6's ([world](topics/world.md)).

**Open:** the completion rule per mission family beyond the defaults; whether cars and props
stream as pedestrians do or through the replay recorder's living-world record
([research](../research/RE-living-world-pedestrians.md)); the server's save blob format (M2).

## History

- 2026-11-10 (M0, M1): the file begun, from `design.md` §0, §4, §7 and the player's instructions
  of 2026-11-09 and 2026-11-10 (no attribution, commit as you go, no images, logging alongside).
