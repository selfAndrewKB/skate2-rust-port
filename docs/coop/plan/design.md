# Co-op campaign: analysis and design record

Written 2026-11-10 from a read of the port's code and docs (three read-only surveys: netcode,
career, living world) and the methodology in the `split-screen-coop` skill. Nothing here has run
yet. Facts carry file paths; judgements are marked as such. Policy choices the player has to
make are collected in §7.

## 0. Requirements (the player's, 2026-11-10)

These are fixed. The design below is shaped to them; anything that contradicts them is wrong.

1. **The open world is synchronized.** Pedestrians, security, cars, props: both players see
   the same city doing the same things. Not cosmetic, not "mission-required only". Stated
   precisely (2026-11-10): identical wherever either player can see, as GTA Online and Red Dead
   do it; far apart, each has their own crowd, which nobody can observe.
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

Requirement 1 removes the cheap answer (an empty or cosmetic city). Four routes give a shared
living world.

| Route | How the world stays shared | What it costs | Latency where it matters |
| --- | --- | --- | --- |
| **A. Deterministic shared world** (the original's own design, minus the skater) | Both consoles simulate the living world from the session seed and the same inputs (tick-exact facts, contacts), with a residency barrier and a per-tick checksum. | Cross-architecture determinism (unmeasured); both players' areas streamed on both machines; a jitter lead between your skater and the world; the world freezes on a lag spike. | Your board: none. A ped's reaction: tick-exact on both, but the ped was already about 100 ms stale to your skater. |
| **B. Server-simulated world** | The dedicated server runs the living world and the scripts headless; clients receive snapshots. | The server becomes a full headless game; `apply_career_outputs` splits in two; fights the port's relay design. | A ped's reaction: one round trip. |
| **C. Host-simulated world** | As B, but one player's console is the world. | Least infrastructure; asymmetric. | The host at once, the joiner after a round trip. |
| **D. Distributed ownership** (what GTA Online, Red Dead Online and Saints Row do) | Every pedestrian, car, prop and guard has one owner at a time, the nearest player by default; the owner simulates it and streams its state; the others see an interpolated puppet; ownership migrates by rule. The port's remote-skater puppet model, extended to the world. | A per-entity ownership table and migration rule; a streamed world-entity format (the replay recorder's living-world record is the starting point); local prediction of the obvious outcomes (a knockdown starts on your screen before the owner confirms); bandwidth that scales with how many entities are near both players. | Your board: none. A ped you own: at once. A ped your partner owns: after a round trip, hidden by prediction. |

**Verdict (2026-11-10): D.** First recorded as A on 2026-11-10 and switched the same day
after the player asked why the games they trust never show A's costs. The deciding row is the
last: under A your skater runs a jitter lead ahead of the shared world anyway, so A's exact
agreement between the two machines buys nothing a player can see, at the price of determinism
and dual streaming. D gives the behaviour those games have, with their one known cost, and it
extends the port's netcode at its centre instead of beside it: a pedestrian becomes what a
remote skater already is.

**Route D's downsides, stated plainly:**

- **A partner's entity reacts to you after a round trip.** Your skater collides with the
  puppet where it was about 100 ms ago; the owner decides the outcome; you see it confirmed a
  round trip later. Local prediction starts the obvious result at once (the ped begins to fall,
  the cone begins to tip) and the owner's state replaces it. Where the two disagree, the puppet
  pops. On a LAN this is invisible; on a bad link it is GTA's rubber-banding.
- **Ownership handoffs can glitch.** The moment an entity changes owner, its new owner's
  simulation picks up from streamed state: a pedestrian mid-animation, a car mid-lane change.
  The migration rule must hand over with the full state, and the handoff is logged.
- **Bandwidth scales with the crowd you share.** Standing together in a busy street means
  streaming tens of entities each way. Pose for a pedestrian is streamed as animation-graph
  state, not bones, or it costs a skater's worth each; cars are root plus wheels. Budget and
  measure at M1.
- **Mission entities need a named owner, not "nearest".** A guard chasing you must be owned by
  you, or every turn you make arrives late; a story character is owned by the player whose
  tester box spawned it; a challenge prop by whoever touched it last (the port docs' own idea).
  The ownership rule is a table with these kinds, not a distance test alone.
- **Far apart, your crowds differ.** Unobservable, and stated in requirement 1.

What is not a downside: determinism (not needed), streaming (each machine streams around
itself, as today), lag spikes (one player's entities degrade; nothing waits), fairness
(symmetric by rule), fighting the netcode (none).

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
6. **The shared living world (route D).** Three parts, each a feature in four parts (§4):
   - *Ownership:* a pure rule from the entity's kind and every player's position to its owner
     (ambient ped, car, prop: the nearest player; a guard: the player it chases; a story
     character: the player whose tester box spawned it; a challenge prop: whoever touched it
     last) with hysteresis so ownership does not flap, and a migration that hands over the
     entity's full state. One table, one module, logged at its funnel: entity, old owner, new
     owner, reason.
   - *Streaming:* the owner's console sends each owned entity's state at the replay-frame
     cadence (root, velocity, animation-graph state, flags; wheels for cars; the DMO transform
     for props) in a world run beside the skater's run, coded against acknowledged baselines as
     the skater's frames are; the receiver drives a puppet through the same playout delay. The
     living-world entity on the receiving side is a real entity in the census with its physics
     body kinematic, so the local skater, AI and scoring see it as they see any pedestrian.
   - *Interaction:* a local skater's contact with a puppet is reported as a contact fact and
     applied by the owner; the obvious result is predicted locally and replaced by the owner's
     state. Mission scripts read world state (`GetIfPlayerBeingChased`, `IsDMOInVolume`) through
     facts the owner reports, so every console's script run sees the same answer at the same
     tick, as it does for skater facts today.
   - *Population:* each console's census spawns only entities it would own (around itself); a
     region both players share is populated by whichever console is nearer, and the other's
     census treats streamed entities as present so it does not double-spawn.
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
  Designed for N seats, tested with two: the ownership rule and the "nearest" helper take a
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
| M1 | Ownership spike | One pedestrian owned by the other machine, streamed as a puppet, knocked down by your skater, the knockdown predicted locally and confirmed by the owner; bandwidth per entity measured; the ownership and streaming formats fixed |
| M2 | Shared progression | Start State Machine and the tutorials complete with every player proving each objective (requirement 3); identical autosaves on both; the server holds the blob; ledger v1 |
| M3 | Prompts and sign-up | Sign-up, poster, dialogs, Skate 101, unlock overlay owned by the activator and mirrored; the training area plays through |
| M4 | Scenes in step | NIS, FMV and speech under the online rule, same tick on both, each console showing its own skater |
| M5 | Shared world, mission-required | Story characters, challenge props, guards and chases owned by rule and streamed; `Security` and `Service` kinds play |
| M6 | Shared world, the city | Ambient pedestrians, cars and AI skaters on; ownership by nearest with migration; the shared region populated once; contacts as facts; budget held in a busy street |
| M7 | Kinds | Per-kind completion rules (§7) across every mission family; AI photographer and cameraman kept for photo and film, both players skating |
| M8 | Whole campaign | Mission crawl green from the tutorial to the ender on two consoles; resume from the server's blob; reconnect; Training Facility travel |

## 6. Risks, honestly

- **Pacing hangs.** The facts lockstep hangs when two consoles disagree by one tick on a change
  of pacing (`docs/online/facts.md`). A session-long round multiplies the exposure; the `Stalled`
  vote-out is S.K.A.T.E.-only today and must become general.
- **Clock-local events.** Anything on a console's own clock (loads, streaming, scene ends) must
  reach the scripts only through the leader's notes. The career has more such events than a
  challenge round.
- **The living world is the requirement and the risk.** Route D's unknowns are bandwidth in a shared crowd and handoff glitches, both measured at M1 and M6; the ownership table is where a wrong kind shows up as a late guard or a double-spawned ped, so it is logged at its funnel from the first spike.
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

**The scripts' index space (2026-11-11):** a campaign's scripts see the career's own players,
as the single-player game numbers them: themselves as 0 (`Online.GetLocalPlayerID`), their AI
skaters 1 on; one funnel (`challenge::players::script_slot`) turns a script's player into the
simulation's slot, 0 to this console's skater, an AI's ordinal `k` to slot `N - 1 + k` after the
N humans, the same slot on every console. The other humans have no script index: what a script
learns of them comes through the rules and the facts. Decided on the bench: the shipped scripts
count their AI three ways (a literal, the local player plus one, every index but their own), and
only this view serves all three ([open items](open-items.md) 56). The challenge AI skaters were
each console's own at first (the original's challenge AI is offline-only), their frames joining
the scripts' per-player frames on that console alone; since M10 they are the first player's and
their frames travel with its facts (§7 below). The ambient skaters stay each console's own (53)
and, since M10.3, out of the scripts' frames altogether: on every console the scripts read the
humans and the one AI run, and nothing that differs between consoles (69).

**The competing kinds: outcomes as notes (2026-11-11, M9).** Each console plays a S.K.A.T.E.,
jam or contest against its own AI, and a script that reads its AI's run decides for its own
player alone. Such a decision (an objective forced complete, the objectives all failed, the
mission won or lost on the script's own terms) is not applied where it is made: it travels as a
note on that player's facts (`FactsNotes.outcomes`, protocol 122) and every console, the
decider's included, applies it to that player's copy at the note's tick
(`coop::outcomes`, `World::apply_coop_outcome`). The rules then settle the mission from every
player's copy as they already do for the objective kinds: `Anyone` and `Compete { false }` at
the first win, `Everyone` and `Compete { true }` once every human has won; a loss ends the
mission when the rule's failure count is met. A `Challenge.CompleteChallenge` that follows the
shared objective completion still applies at once, the same tick everywhere. What is not
shared is the AI's run itself: two consoles' copies of one pro skate their own paths; a
streamed, owned AI skater remains the alternative if play finds the copies' difference
matters.

**The AI skaters: one owner (2026-11-11, M10).** A campaign round's challenge AI skaters are
one console's, the first player's (`CampaignWorld::ai_owner`): it simulates them as the career
does; its facts carry their completed outputs at their slots (`Facts.owned`, protocol 123),
delivered with its own frame at the tick on every console, so every script reads one AI run;
their creation and destruction are the frames' (an AI slot's first and last frame, the same
tick everywhere); and their bodies reach the other consoles as the remote players' do (the look
once, replay frames in runs, a `RemoteSkater` puppet), under marked names apart from the
players' (`protocol::owned`). The other consoles take each spawn's row and slot and build no
actor. Chosen over shared AI simulation (the determinism §2 set aside) and over keeping each
console's copies (M9's residue: two pros skating, and the competing kinds settled only by
notes). A handoff when the owner leaves is owed ([ai-skaters](topics/ai-skaters.md)).

**Route (2026-11-10):** D, distributed ownership. A was the first choice the same day and was
set aside for the reasons in §2; its checksum remains available as a test instrument.
