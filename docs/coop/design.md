# Co-op campaign: analysis and design record

Written 2026-11-10 from a read of the port's code and docs (three read-only surveys: netcode,
career, living world) and the methodology in the `split-screen-coop` skill. Nothing here has run
yet. Facts carry file paths; judgements are marked as such. Policy choices the player has to
make are collected in §7.

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

| Paradigm | Cost against the facts above | Verdict |
| --- | --- | --- |
| **Full lockstep** (the original's design, `docs/online/lockstep.md`) | Reseed three generators from the session; audit HashMaps and libm; keep both players' cells loaded on both machines (the single stream focus is the blocker); your own skater waits on the peer's input. The port abandoned this on purpose. | No for the skaters. The *checksum probe* (`Simulation::GetSyncLog` model) is worth porting as a test instrument. |
| **Puppets + facts lockstep** (today) | Exists and is proven. Scripts agree. The world is empty and single-player assumptions are everywhere in the career code. | The base. |
| **Host-authoritative world** (Jedi Knight route) | The joiner rebuilds pedestrians, traffic, props, guards from snapshots; the replay recorder already captures the living world per tick (`RecordedWorld`), so a stream exists, but the joiner's interactions with guards and props round-trip. | Fallback for the parts of the world that resist determinism. |
| **Hybrid** | Skaters owner-authoritative; progression and mission scripts in the facts lockstep for the whole session; the living world seeded from the session and driven by a *union* of both players' census centres, with the nearest player owning corrections; saves written identically on both. | **Recommended** (judgement). It extends mechanisms that exist and keeps the board's feel. |

### Axis 2: what "co-op campaign" means to play

| Model | Feel | Build cost |
| --- | --- | --- |
| One shared career | The real thing: one story, two skaters, both progress | Progression lockstep + a "the player" ledger + save mirroring |
| Host's career with a buddy | Player two is a guest | Cheapest; player two keeps nothing |
| Two careers, one city | Ride together, drift apart | Lowest sync, weakest story |
| Role co-op (photo/film) | Player two *is* the photographer or cameraman | New: a human in an AI role |
| Competitor swap | Human rival in races, contests, S.K.A.T.E., throwdowns | The online types already run these |

Recommendation (judgement): **one shared career**, with per-kind rules (§7) and role co-op as
the feature that makes it more than "Skate 2 with two people".

## 3. The architecture, in the port's own terms

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
   | the host | one authority | nothing by design if saves are mirrored; else save, resume, world travel |

   Prompts answered by one pad become notes on the pacing player's facts (`FactsNotes` already
   carries votes and presented events); the other console mirrors the screen read-only.
4. **Objectives per kind.** The objective system already evaluates per-player groups
   (`online_round_per_player_objectives`). A mission's completion rule (either / both /
   combined / compete) is a small pure rule keyed on `ChallengeType`, chosen in one table, not
   scattered through scripts. Where the original script hard-codes one player, the fix goes in
   `challenge/script_fixes.rs`, which already exists for online.
5. **Cutscenes, FMVs and speech.** Use the online rule: no pause, no skip, same ticks, ends
   announced by the pacing leader. Stretch: the second skater as a scene character (the NIS can
   borrow a skater by player id; 32-character limit).
6. **Living world, in two steps.**
   - *Step one, mission-required only:* story characters, challenge props, security guards and
     chases spawn from the shared tick and seed, owned by the nearest player, corrected from the
     owner's facts when they diverge. Ambient pedestrians and traffic stay banned online, as now.
     This is enough for every mission kind except "the crowd looks the same".
   - *Step two, the shared city:* reseed the three generators from the session, drive the census
     from the union of both centres, keep both players' cells resident, and port the original's
     per-tick checksum as a divergence probe. Go only as far as the probe says is cheap; where it
     is not, stream that class from its owner (the replay recorder's `RecordedWorld` is the
     format).
7. **Saves mirrored, not hosted.** Because the progression run is deterministic, both consoles
   hold identical `SignPost`/`Progress`/unlockables/hulls and each writes them into its own
   `SKATER.P` beside its own profile and Create-a-Skater record. Resuming a campaign means both
   present the same progression hash; a mismatch is shown and the newer save wins (policy, §7).
   This keeps "either of us can continue alone" true.
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
| M0 | Bench | Two instances + server on one machine; a campaign lobby loads a career with progression on; both skaters visible; no mission runs |
| M1 | Shared progression | Start State Machine and `m_1a_01` complete together; both consoles autosave identical progression; ledger v1 covers every site the first mission touches |
| M2 | Prompts and sign-up | Sign-up, poster, dialogs, Skate 101, unlock overlay owned by the activator and mirrored; the training area (`m_1a_01` → `m_1b_01`) plays through |
| M3 | Scenes in step | NIS, FMV and speech under the online rule, both consoles same tick; second skater in scene (stretch) |
| M4 | Mission-required world | Story characters, challenge props, guards and chases shared; `Security` and `Service` kinds play |
| M5 | Kinds | Per-kind completion rules; role co-op for photo and film; competitor swap for races, contests, throwdowns |
| M6 | Shared city | Seeded census, union centre, divergence probe; ambient pedestrians and traffic match within the probe's tolerance |
| M7 | Whole campaign | Mission crawl green from the tutorial to the ender; resume and reconnect; Training Facility travel |

## 6. Risks, honestly

- **Pacing hangs.** The facts lockstep hangs when two consoles disagree by one tick on a change
  of pacing (`docs/online/facts.md`). A session-long round multiplies the exposure; the `Stalled`
  vote-out is S.K.A.T.E.-only today and must become general.
- **Clock-local events.** Anything on a console's own clock (loads, streaming, scene ends) must
  reach the scripts only through the leader's notes. The career has more such events than a
  challenge round.
- **The living world is the open end.** Step one is bounded; step two is measured, not promised.
- **Career completeness upstream.** Co-op cannot be more finished than the solo campaign;
  upstream's own gaps (Training Facility, unbound natives) gate M7.
- **Build secret.** Both machines must build the same fork commit with the same
  `release-secret.txt`, or they never meet.

## 7. Decisions that are yours

1. Shared career (recommended) or host's career with a guest?
2. Per-kind completion rules. A first cut: tutorials, go-to and services **either**; photo and
   film **one performs, one shoots**; Own the Spot and Hall of Meat **combined**; races,
   contests and throwdowns **both compete**; security and escape **everyone**.
3. Must ambient pedestrians and traffic match between you (M6), or is "the same guards, props
   and story characters" enough?
4. Both skaters in cutscenes (stretch), or the activator only?
5. Save conflict on resume: newer wins, or host wins?
6. Player count: two by design, or leave room for four (the ledger's "nearest" kind is what
   changes).
