# Co-op campaign: analysis and design record

Written 2026-11-10 from a read of the port's code and docs (three read-only surveys: netcode,
career, living world) and the methodology in the `split-screen-coop` skill. Nothing here has run
yet. Facts carry file paths; judgements are marked as such. Policy choices the player has to
make are collected in §7.

## 0. Requirements (the player's, 2026-11-10)

These are fixed. The design below is shaped to them; anything that contradicts them is wrong.

1. **The open world is synchronized.** Pedestrians, security, cars, props: both players see
   the same city doing the same things. Not cosmetic, not "mission-required only".
2. **Progression is shared**, and may be hosted on the dedicated server rather than on either
   player's machine.
3. **Tutorials are proven by everyone.** Each player completes the tutorial requirements
   before all players move on. The game asked one player to prove they can do it; co-op asks
   every player.
4. **No role co-op.** Nobody is reduced to holding the camera while the other skates. Both
   players skate every mission.
5. **Per-kind completion rules are fine**, but the first-cut table in the earlier draft is not
   the one; see §7 for the open choices.
6. **Cutscenes show your own skater.** Each player's console shows that player's character in
   scenes, as Saints Row does. Never both skaters in one scene.

## 1. What the port already is

**Online today** (`docs/online/implementation.md`, `crates/skate2-net`, `crates/skate2-core/src/online`):

- A dedicated server that simulates nothing: it relays recipes, replay frames and challenge
  facts, runs the lobby phase machine, barriers and timeouts, and hands out seeds
  (`crates/skate2-net/src/server.rs`). Max 8 players. No persistence, no reconnect, random user
  ids per sign-in.
- Each console simulates only its own skater. Remote skaters are physics-less puppets replayed
  from compressed replay-recorder frames at 20-30 runs/s with adaptive playout delay
  (`session/online/remote_skater.rs`, `skate2-net/src/frames.rs`). Zero input lag on your own
  board; no skater-to-skater contact.
- **The facts lockstep**: every console runs the whole challenge script (Lua, on the port's own
  VM) for every player, fed by each player's per-tick `MissionFrame` facts, seeded by the
  server's `GameReset`. Turns, scores, letters, timers and results are consequences of that
  shared run, not packets (`online/facts.rs`, `session/online/online_challenge.rs`,
  `docs/online/facts.md`). Scene and speech-line ends are delivered on the tick the pacing
  leader announces them (`HeldPresentation`, `FactsNotes::presented`).
- **The world is absent online.** Pedestrians, traffic and ambient AI are banned in modes 2, 4
  and 5 (`session/living_world/selected_living_world_dmo.rs:~785`, `ai/skater_manager.rs`
  `in_an_online_game`); props are the challenge hull's frozen statics (`session/online/online_dmos.rs`).
  Time of day does not exist; weather is a per-machine PC addition.
- Entering online drops the career (`start_online_game` → `drop_career()`,
  `session/selected_session_online.rs:~1110`). Online challenges build a fresh `CareerSession`
  with only unlockables loaded and `game_mode_flags = 8`; the autosave fires only for flag 4
  (career). **No online path writes career progression.**

**The career** (`docs/career/`, `crates/skate2-core/src/challenge`):

- Entirely data-driven: 233 progression state machines and 804 challenges as Lua 5.1 handlers
  in the Vault databases, run by the ported VM. Two script layers: progression machines
  (`challenge/signpost.rs`) and per-challenge state graphs (`challenge/runtime.rs`,
  `activity.rs`), linked by trigger records and `Progression.SendMessage`.
- Every script effect leaves through one enum, `Output` (`challenge/system.rs:~199`), applied in
  one function, `apply_career_outputs` (`session/selected_session_career.rs:~3017`). All saved
  career state is `World.signpost` + `World.unlockables` + `World.dynamic_hulls` + statistics,
  written as one `SKATER.P` together with the profile and Create-a-Skater record.
- Missions assume one local player: `world.player`, `local_player`, `Online.GetLocalPlayerID`,
  the sign-up box tested against the local tester only, teleports/pins/session marker for the
  local skater, dialogs and posters waiting on one pad.
- Cutscenes pause the whole simulation offline (`PAUSE_STATE_NIS`); online they keep simulating
  and cannot be skipped.
- Status: the campaign is "mostly there". An audit pushed wins through 255 challenges without a
  script error but was not a legal-order playthrough; the Training Facility world load was the
  stated blocker (`docs/evidence/career-progression-audit-2026-10-13.md`). Re-run
  `python tools/career/unbound_natives.py` for the current native count.

**The living world** (`crates/skate2-core/src/living_world`, `ai`):

- One manager RNG seeded from the wall clock (`math/native_rw_math_random.rs` `global_next`,
  `living_world/interaction/living_world_grind_services.rs:~268`). AI skaters use the same
  global generator plus a process-global C-runtime `rand` (`ai/crt_random.rs`). So placement
  already differs between any two processes.
- Population is centred on the one local skater (`local_skater_census_centre`) or the camera
  (AI); anything outside the cull radius is destroyed. There is no per-cell simulation.
- One simulation stream focus; two players far apart load different collision, navigation,
  AI-path and prop-locator cells.
- Missions do depend on it: chase state (`LivingWorld.GetIfPlayerBeingChased`), props in volumes
  (`DMOs.IsDMOInVolume`), force-spawned story characters (`challenge/characters.rs`), cull and
  avoid operations, AI competitors, spectator crowds.
- Numerics are in good shape for determinism: no fast-math, no arch-specific code, explicit
  `mul_add`, polynomial sin/cos. Risks: a handful of libm calls, NaN bit patterns, HashMap
  iteration order (needs an audit), asynchronous streaming timing, puddle physics.

## 2. The two axes

### Axis 1: how the machines stay in step

Requirement 1 removes the cheap answer (an empty or cosmetic city). Three routes remain that
give one shared living world; all three need the city streamed around *both* players (today
there is one stream focus), so that cost is common.

| Route | How the world stays shared | What it costs beyond the common part | Latency where it matters |
| --- | --- | --- | --- |
| **A. Deterministic shared world** (the original's own design, minus the skater) | Both consoles simulate the living world from the session seed and the same inputs: each player's tick-exact facts (positions, velocities, contacts) and a residency barrier so cells are resident on both before a tick needs them. A ported per-tick checksum detects divergence; the nearest player's console repairs a diverged entity from its own state. | Reseed the three generators (`rw_math::global_next`, `crt_random`, the manager RNG) from the session; audit HashMap order and the few libm calls; contacts as facts (the original's command types 13/14); the world tick runs a jitter lead (about 6 ticks) behind your skater, so your skater meets a ped where it will be, not where it was. | Your board: none. A ped's reaction to you: tick-exact on both, built from your contact facts. |
| **B. Server-simulated world** | The dedicated server runs the living world (and the scripts) headless with the game data; clients receive world snapshots (the replay recorder's `RecordedWorld` is the format) and send their skater's facts. | The server becomes a full headless game: streaming, physics for peds, cars and props, navigation, scripts. `apply_career_outputs` splits into a server half and a presentation half. Bandwidth: roughly 2 KB a world frame compressed, tens of KB/s a client. | Your board: none. A ped's reaction to you: one round trip (100-200 ms). |
| **C. Host-simulated world** | As B, but one player's console is the world. | Least new infrastructure; the host plays the solo game and streams it. | Asymmetric: the host's peds react at once, the joiner's after a round trip. |

Verdict (judgement): **A, with B's server as the store for progression.** Reasons: it is what
the game was designed to do and the port's numerics are nearly ready for it (no fast-math, no
arch-specific code, explicit `mul_add`, polynomial sin/cos); it is symmetric, which C is not; it
keeps the server the thin relay it is, which B does not; bandwidth is tiny. It is also the one
route whose central risk can be *measured before committing*: two headless instances fed the
same inputs, compared by checksum. That spike is milestone M1 below, and if it fails the plan
falls back to B without wasted work, because the stream-focus, contact-facts and ledger work
serve both.

**Route A's downsides, stated plainly** (the player asked to be told; this list is the
promise):

- **The one unknown is cross-machine determinism.** Mac arm64 and Windows x64 have never been
  compared. The port's numerics are promising (no fast-math, no arch-specific code, explicit
  `mul_add`, polynomial sin/cos, no FMA contraction by default), but a few libm calls, NaN bit
  patterns and HashMap iteration order are unaudited. If M1 finds divergence that cannot be
  removed, route A is dead and the plan goes to B. M1 is first so that this is learned before
  anything is built on it.
- **The city runs a few ticks behind your board.** Your skater simulates now; the shared world
  tick waits for the other player's facts, so it runs a jitter lead behind (about six ticks,
  100 ms, the same lead online challenge ticks use today). You meet a pedestrian where it will
  be, built from your contact facts, so knockdowns look right; a car at speed can be a metre off
  from where you felt it. A long link hitch freezes the city for a moment while your skater
  keeps moving. Skitching a car is the known wart: the car you hold is a few ticks stale on your
  own console; the fix is the docs' own idea, ownership of a movable to whoever touched it last.
- **Both players' areas load on both machines.** Memory and streaming roughly double, and a
  partner teleporting across the map puts a short load wait on you (the online sync point's
  loading screen, generalised). This cost is common to every shared-world route.
- **A lockstep waits on the slowest.** With four players the world waits on the slowest link.
  Fine for friends, wrong for strangers.
- **Repair is a safety net with sharp edges.** Adopting a diverged entity's state
  mid-simulation can pop; if the probe fires often, A has degraded into a worse B.

What is *not* a downside: bandwidth (tiny), fairness (symmetric), fighting the port's netcode
(A extends the facts lockstep; B would replace the relay with a headless game).

### Axis 2: what "co-op campaign" means to play

Decided by the requirements: **one shared career**, progression identical on both consoles and
held by the server; **no role co-op**; **each console shows its own skater in scenes**. What is
still open is the completion rule per mission kind (§7).

## 3. The architecture, in the port's own terms

Constraint: build on the port's netcode, never beside it. The relay server, the replay-frame
puppets, the facts lockstep and its notes, the barriers and the seeded `GameReset` are the
vocabulary; new needs become new facts groups, notes and messages added the documented way
(`docs/online/implementation.md`), not a second transport.

1. **A campaign session type.** A new `PlayNowMode`/game attribute for a private two-player
   campaign lobby (`skate2-net/src/protocol.rs`; bump `PROTOCOL_VERSION` and take a fork-own
   number so an upstream client is refused, not misparsed). The client enters a new game mode
   that *keeps* the career: progression on (`game_mode_flags` 4 semantics) **and** networked.
   `start_online_challenge_load` (`selected_session_career.rs:~1835`) is the template for an
   online-bound `CareerSession`; `start_career` (`~1890`) is what it must load instead.
2. **Session-long facts lockstep.** Today a `Round` lives for one challenge. The campaign needs
   the round to be the whole session: progression machines, sign-up, global activities and
   missions all step on the shared tick from both players' facts, scope "everything" (the
   type-42 path in `facts::group`). The server's `GameReset` seed becomes the career's
   `online_random`. Every console runs every script; nobody arbitrates.
3. **A "the player" ledger.** Every site that means one player must be classified once and
   routed through one helper per kind (the skill's rule, `networked-coop.md`). The sites are
   enumerable: `world.player`/`local_player`/`Online.GetLocalPlayerID` reads in
   `challenge/`, and each `Output` variant in `apply_career_outputs`. Kinds:

   | Kind | Means | Examples |
   | --- | --- | --- |
   | everyone | shared state and shared information | progression state, unlocks, cash, message settings, central messages, map icons, minimap |
   | the one who did it | the activator | sign-up (who entered the box), poster accept/decline, dialog Yes/No, Skate 101 continue |
   | the nearest player who is up | AI and scripts aimed at "the player" | `AI.RepositionAI` targets, story characters' spawn trigger, chase target |
   | each player, for themself | rules on one body | teleports and pins (the owner applies them, as online does), session marker, breadcrumb, HUD binding |
   | the server | one authority | the campaign's save blob, resume, who is in the campaign; no player is "the host" in any rule |

   Prompts answered by one pad become notes on the pacing player's facts (`FactsNotes` already
   carries votes and presented events); the other console mirrors the screen read-only.
4. **Completion rules as data, swappable without a rebuild.** The rules are felt out in play,
   so they are never hard-coded where a script or a sub-controller decides. One pure module
   (`coop/rules.rs`): a `Rule` per mission family (`Everyone`, `Anyone`, `Combined`,
   `BestOf`, `Compete { every_human_must_win }`), a table from `ChallengeType` (with per-mission
   overrides by challenge key) to `Rule`, and one function that turns per-player objective
   states into the mission's state. The table is read from `coop-rules.json` beside
   `config.json`, with a built-in default (tutorials `Everyone`, everything else `Anyone`), and
   can be changed from the campaign lobby's settings page and by `SKATE2_COOP_RULES=<file>`.
   The table is part of the session seed's companions: the server sends it in the campaign's
   `GameReset`, so every console runs the same rule. The objective system already evaluates
   per-player groups (`online_round_per_player_objectives`); the adapter at its one evaluation
   point asks the rule. A script that hard-codes one player is corrected in
   `challenge/script_fixes.rs`, which exists for online, never in the Vault data.
5. **Cutscenes, FMVs and speech.** The online rule: no pause, no skip, same ticks, ends
   announced by the pacing leader. Each console's scene borrows *its own* local skater as
   `current_player`, which is what the NIS does today, so requirement 6 costs nothing.
6. **The shared living world (route A).** Three parts, each a feature in four parts (§4):
   - *Stream foci:* `StreamFocusState` becomes a set; cells needed by either player are resident
     on both; a tick that needs a cell not yet resident everywhere waits (the sync point's
     "stayed loaded 16 updates" rule, generalised), announced in the facts notes.
   - *Inputs:* the census centre is a pure function of every player's tick-exact facts (root,
     velocity), not of `local_skater_census_centre`; skater contacts with peds, cars and props
     travel as facts (the original's `SkateboardContactsData`/`SkeletonContactsData`); the three
     generators are seeded from the session; remote skaters get a kinematic body at their facts
     position so peds, guards and AI see them.
   - *Agreement:* a per-tick world checksum modelled on `Simulation::GetSyncLog` (tick, RNG
     state, every living-world entity, movable props), compared through the facts; on a
     mismatch the nearest player's console sends the entity's state and the other adopts it,
     logged with the entity, the tick and both values. The probe runs always; the repair is the
     safety net, not the design.
7. **Progression on the server, mirrored on both.** Both consoles hold identical progression
   because the run is deterministic. The server keeps the campaign's save blob (`SignPost`,
   `Progress`, unlockables, hull states, statistics) in a new small store, keyed by campaign id,
   written when the scripts autosave and checked against both consoles' hashes. Each console
   still writes its own `SKATER.P` with that progression beside its own profile and
   Create-a-Skater record, so either player can continue alone. Resume: join the campaign on the
   server; the server's blob is the truth; a console whose local copy differs adopts it.
8. **Identity and resume.** The server has no persistent identity. A campaign needs a stable
   per-player id (derive from the profile's peer id, `skate_profile.rs`), a resume path that
   loads two saves at the same progression, and the reconnect fix the skill documents (drop a
   slot at once when the same address reconnects).

## 4. Method

From `split-screen-coop/references/project-structure.md`, `agent-coordination.md`,
`validation.md`, applied to this repo:

- **Four parts per feature.** Rules as pure, host-tested code that touches no engine (a
  completion rule, the ledger's kind table, the save-merge rule); one thin adapter at the
  engine lever (`apply_career_outputs`, `facts::group`, the census centre); presentation that
  only draws what it is told; one coordinator per feature. Randomised tests over invariants the
  rules must keep.
- **Seats and columns, not "player 2".** State indexed by seat; no `is_host` branches in rules.
  Designed for N seats, tested with two: the census union and the "nearest" helper take a
  list, never a pair.
- **Rules are data.** A completion rule, a kind in the ledger, a pacing set: each lives in one
  table that can be swapped at runtime, so playtesting changes a file, not code in six places.
- **The ledger is the backlog.** Classify sites before changing them; log the *decision* at the
  one funnel (old → new owner, reason), not the outcome.
- **Proof layers stay distinct.** Static reading, pure tests, headless career tests with the
  game data (`challenge::tests`), two-instance bench, field play. Promote a claim only as far as
  its layer reaches.
- **Two instances on one machine is the first test.** Two `SKATE2_SAVE_ROOT`s, one
  `skate2-server`, one game root. Decide this before writing the client path.
- **A mission crawl is the regression.** The audit that pushed wins through 255 challenges is
  the seed of it: run it with two consoles, assert same state at same tick
  (`SKATE2_TRACE_ONLINE_INPUTS`, `tools/online_diverge.py`), pass a mission only when both logs
  agree and both processes survive.
- **Log both ends of every bridge.** A facts note sent and not applied must say which.
- **Protocol hygiene.** Own protocol number; bump on a *meaning* change, not only a layout
  change; `static_assert`-style tests on the hand-coded message tags (`skate2-net/src/tests.rs`).
- **Upstream hygiene.** Everything above lives on `coop`. A fix that is upstream-worthy (a
  script fix, a determinism fix, the reconnect fix) branches from `main`.

## 5. Milestones (observable, in dependency order)

| # | Milestone | Done when |
| --- | --- | --- |
| M0 | Bench | Two instances + server on one machine, two scratch saves; a campaign lobby loads a career with progression on; both skaters visible; nothing synced yet |
| M1 | Determinism spike (decides route A or B) | Three generators seeded from the session; the world checksum ported; two headless instances fed identical recorded inputs stay in agreement for ten minutes of city, or the first divergence is named and priced |
| M2 | Shared progression | Start State Machine and the tutorials complete with every player proving each objective (requirement 3); identical autosaves on both; the server holds the blob; ledger v1 |
| M3 | Prompts and sign-up | Sign-up, poster, dialogs, Skate 101, unlock overlay owned by the activator and mirrored; the training area plays through |
| M4 | Scenes in step | NIS, FMV and speech under the online rule, same tick on both, each console showing its own skater |
| M5 | Shared world, mission-required | Story characters, challenge props, guards and chases from the shared tick; `Security` and `Service` kinds play |
| M6 | Shared world, the city | Dual stream foci and the residency barrier; ambient pedestrians, cars and AI skaters on and in agreement; contacts as facts; remote skaters as kinematic bodies |
| M7 | Kinds | Per-kind completion rules (§7) across every mission family; AI photographer and cameraman kept for photo and film, both players skating |
| M8 | Whole campaign | Mission crawl green from the tutorial to the ender on two consoles; resume from the server's blob; reconnect; Training Facility travel |

## 6. Risks, honestly

- **Pacing hangs.** The facts lockstep hangs when two consoles disagree by one tick on a change
  of pacing (`docs/online/facts.md`). A session-long round multiplies the exposure; the `Stalled`
  vote-out is S.K.A.T.E.-only today and must become general.
- **Clock-local events.** Anything on a console's own clock (loads, streaming, scene ends) must
  reach the scripts only through the leader's notes. The career has more such events than a
  challenge round.
- **The living world is the requirement and the risk.** Route A's determinism is measured at M1, not assumed; route B is the priced fallback and shares M0, the stream-foci work, contact facts and the ledger with A.
- **Career completeness upstream.** Co-op cannot be more finished than the solo campaign;
  upstream's own gaps (Training Facility, unbound natives) gate M7.
- **Build secret.** Both machines must build the same fork commit with the same
  `release-secret.txt`, or they never meet.

## 7. Decisions

**Decided (2026-11-10):** one shared career, held by the server and mirrored on both; the whole
open world synchronized; no role co-op; tutorials completed by every player; each console shows
its own skater in scenes.

**Completion rules (2026-11-10): start basic, tune in play.** The default table is tutorials
`Everyone`, everything else `Anyone`; the family options below are what the rules file can be
set to, and real sessions decide. Nothing is fixed here on purpose.

| Family (`eChallengeTypes`) | Rules available | Default |
| --- | --- | --- |
| Tutorials, Skate 101 | `Everyone` | `Everyone` (decided) |
| Go-to, follow, errands, services | `Anyone`, `Everyone` | `Anyone` |
| Photo and film (AI photographer and cameraman kept) | `Everyone`, `Anyone`, `Combined` | `Anyone` |
| Own the Spot, Hall of Meat | `Everyone`, `Combined`, `BestOf` | `Anyone` (best single run) |
| Races, Death Race, contests, throwdowns | `Compete { every_human_must_win: false }`, `{ true }` | best human result counts |
| Security chases and escapes | `Everyone`, `Anyone` | `Everyone` |

**Player count (2026-11-10):** designed for N seats, played and tested with two.

**Route (2026-11-10):** A, on the condition stated in §2: M1 measures it first, and B is the
fallback if determinism cannot be had.
