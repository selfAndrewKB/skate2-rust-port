# Open items

**Scope:** what is seen and not yet fixed or explained, by topic, so nothing waits in a log.
Front page: [README](../README.md).

## World ([topic](topics/world.md))

1. A puppet runs its own wander AI while its target transform is its owner's: it walks in
   place or slides. The owner's motion state travels (`EntityState::motion`) but is not applied
   yet. Fix in M6: a puppet's AI step skipped, its locomotion from the stream
   ([research](../research/RE-living-world-pedestrians.md)).
2. A streamed pedestrian costs about 500 B/s at 20 runs a second (25 B a sample): twelve of them
   are 6 KB/s a direction. Runs are self-contained; delta coding against acknowledged runs is
   M6's.
3. Puppets count against the census's own allowance (`max_pedestrians` 12), so a seat with ten
   puppets spawns only two of its own. Good for double spawns, to be decided on purpose in M6.
4. Only pedestrians stream. Cars, props and guards: M5, M6.
5. The remote skater flickers ("no longer shows" / "shows") after a tick backlog; harmless on the
   bench, matters for interpolation.
6. A `controlled host backlog` of about 160 ticks at Continue Career (a hull reload) stalls
   both seats; the campaign's inactivity window (15 s) absorbs it.

## City ([topic](topics/world.md), M6)

32. A promoted puppet keeps the navigation cell it was spawned in; a re-seat at the
    promotion (`set_cell` at its position, as the spawn does) is owed if it shows as a
    pedestrian walking through walls after a handoff.
33. The world stream is still self-contained (no acknowledgement): a descriptor every 20
    runs, a despawn 3 times, every sample absolute. The in-run ack design is in the research
    note; measured at the waterfront with twelve owned pedestrians, 6.6 KB/s a seat (M6), so
    it is not built until a busier street or a slower link asks for it.
34. A puppet's startle, stumble and mood gestures do not travel: its intents are its owner's
    velocity through the speed table, so a startled run shows as a run and nothing else. The
    owner's graph behaviour (a small index) is the next thing to stream if play wants it.

## Kinds ([topic](topics/kinds.md), M7)

35. `BestOf` decides completion like `Anyone`; the best single run's score on the results
    screen and the leaderboard is still each console's own display, not the best copy's.
36. A copy's failure is remembered until its objective activates again (`failed_sent`); a
    script that fails and re-activates an objective every tick (the go-to missions) hears the
    failure every tick under every kind, as before.
37. Each console reads its own rules file; two different files split the round silently
    beyond the hash each traces at the round start. The design has the creator's table ride
    the campaign's `GameReset`; not built.

## Audit ([inventory](sync-inventory.md), [world](topics/world.md))

39. The first audit: a puppet's speed differs from its owner's by more than 0.5 m/s in 28 %
    of pairs and its yaw by more than 5° in 5 %, while its position holds within 0.5 m in
    99.95 %. The puppet's velocity is its drive's own (the coast, then the hold); carrying the
    owner's velocity through to the puppet's `core.velocity` and yaw target is the next
    improvement the audit will measure.
40. Retired: the live audit (the receiving end measures every sample against the puppet it
    drives, traces a puppet 20 samples beyond 0.5 m once, and sums the agreement every ten
    seconds in the `live audit` line) runs in play on any two machines; no wire change was
    needed, the samples already carry the shared fields. The offline audit stays the long-form
    report over the census dumps.

## Play (2026-11-10, the player, M5 build)

31. Pedestrians' positions and the dodge when run into are synced; their models differ per
    screen (the puppet spawn takes the owner's entity key and the census picks the recipe and
    colours at random, though the descriptor carries them), and their startled runs and falls
    are not mirrored (the owner's motion byte travels and the receiver never applies it; the
    puppet's own graph has no moods since M5). All three are M6's first targets: the puppet
    spawned with the owner's recipe, its animation from the owner's motion state and flags.

## Session ([topic](topics/session.md))

7. Two primed saves continue at the same spot with the same look; the bench teleports the seats
   apart. A person's saves will differ.
8. The career's continue point has no pedestrians in the port (offline too); the waterfront
   spawn has them. Not a co-op matter, but every M1-M6 bench run teleports there.

## Progression ([topic](topics/progression.md))

10. A member who loads its career after the campaign round started runs its career alone; the
    server names it in its log. Joining a running campaign is M8's (the server's save adopted,
    the round's tick caught up).
11. Retired by M7: the families are `binding_table.rs`'s `eChallengeTypes` enumeration
    (tutorial 0, security 17, the races and contests, best trick 8, Own the Spot 9, Hall of
    Meat 31 and 33); the built-in table names them ([kinds](topics/kinds.md)).
12. With one objective group per player the HUD may list both copies' objectives; not looked
    at on the bench (no HUD capture of it) [UNKNOWN].
13. The Start State Machine's "New to skate?" dialog does not come back on Continue with a
    mission done, so the bench never answers it; the prompt's answer travelling as a note is
    M3's, and a fresh-profile bench prime is owed.
14. Statistics (Hall of Meat, achievements, service stats) stay each console's own backend;
    the shared save carries progression, unlockables and hulls only (the design's "mirrored on
    both, the server holding the blob" holds for those).
24. The shared blob carried the skater's own orientation leaf until M4, so the seats' hashes
    disagreed from M3 on by its low bits; left out now ([progression](topics/progression.md)
    History). The session marker and the camera are not in the blob either; whether any other
    per-console leaf hides in `cSignPost::Data` shows at the next disagreement.

## Mission world ([topic](topics/world.md), M5)

25. The career's host answers the objectives' DMO queries with nothing (`CareerHost`:
    `dmo_locator_from_lua_id`, `is_dmo_in_volume`), the port's own gap: a "moved object"
    objective completes only by diagnostic, offline too. When it is answered, the answer must
    be the owner's fact (a note), not each console's body, or the lockstep splits.
26. No challenge character spawned in the bench's stretch, so "each console's own Slappy" is
    by reading; a mission that places one (Slappy's shop) is the first to look at.
27. `big_black_active` is each console's readback of its own spawn; a console whose spawn
    failed would branch alone. A deterministic flag set by the lockstep is the fix if it shows.
28. A prop at rest is repeated every 20th run (about 38 B every second per owned prop); a
    busy hull with many touched props adds up. Delta coding and an acknowledged baseline are
    M6's.
29. A guard chases only the skater of the console that simulates it; a remote skater has no
    actor. A guard between two players chases one; the other player sees it as a puppet. To be
    judged in play; migration by the chased player's console is M6's.
30. The bench never saw a guard: five minutes at the edge of `NoSkateZone_Area` (the only
    security area within 1500 m of the waterfront; `2026-11-10-m5c-*.log`) spawned none, so the
    chase facts, the puppet guard and the chase messages are by reading. A teleported, still
    skater may not count as skating in the zone, and the census's security spawn switch may be
    off; a run with the skater moving, or a person, decides.

## Saves ([topic](topics/saves.md), M8)

41. New Career inside a campaign lobby is unexercised: the career starts from nothing on that
    console while the others hold the server's progression, and its first autosave would then
    be the newest. By design a campaign is joined on Continue Career; a guard (refuse New
    Career once the server holds a save, or adopt after Create-a-Skater) is to decide in play.
42. A reload for a late joiner uses the newest autosave the server holds; mission progress since
    it is lost for every member. A save request before the reload (the leader autosaving on
    `ChallengeLoaded`) would narrow the loss; not done.
43. One campaign per server: the save file is keyed by the lobby's constant key, so two campaign
    lobbies on one server would share a file. Fine for one server per group of friends; a key
    per lobby (the host's player id) if a public server is ever wanted.
45. Resolved in the main (2026-11-11, the AI day): the AI skaters never stopped a mission for
    want of a port; four co-op faults did (55 to 58: the manager thought the campaign a network
    game, the scripts' index space, the AI's messages dropped, the hull's locators registered
    after the crawl's spawn). With them fixed the photo, follow, demo and S.K.A.T.E. missions
    probed run the same states at the same ticks on both seats (`2026-11-11-crawl-ai2`: six
    missions, every state sequence identical). Each console runs its own AI skaters (53): a
    follow's pro skates its path on each console alone. Left to play: how a follow heat ends on
    the bench (the pro's path runs past the deadline), a demo lost by a bench that does no tricks
    (its end-options screen), S.K.A.T.E. rounds whose AI leaders fail in turn; and whether two
    consoles' own copies of one pro diverge enough to matter.
46. Resolved (2026-11-11): a teleport to a location no registered arena holds now falls back to
    the world's own table (`BAM_Sim.loc`, which holds 52 of the missions' `*_challengelocator01`);
    none of the crawl's "unknown location" stops was among those: every one was a mission hull's
    own locator, registered a few frames after the crawl's execute (58). The go-tos' arrival
    triggers in play are still unproven.
47. Fixed (2026-11-11): `m_1a_07_follow_slappy` spawns its pro at `m_1a_07_follow_slappy_startlocator01`
    once the crawl preloads the mission's assets before the execute (58), and runs its heat in
    step on both seats (`2026-11-11-crawl-ai2`).
44. No notice is shown to the player while the rejoin runs (the career sits held behind the
    fade); a HUD line is to add.

## Prompts ([topic](topics/prompts.md))

15. A note is applied one to two seconds after the press on the bench (the round's processed
    tick runs that far behind the facts produced); measure in play, and tune the ready lead if
    it shows.
16. The results screen's options, the Hall of Meat results, the blog, the sponsor and covers
    screens still apply their continue locally.
17. The sign-up HUD's own D-pad hold and delay are kept as the original's: a press is a note, a
    release is a note; holding is not modelled across the link.

## Scenes ([topic](topics/scenes.md))

9. Retired by M4: a campaign round sets `career.nis.set_online_game` and the scenes run under
   the online rule ([scenes](topics/scenes.md)).
18. "Each console shows its own skater" rests on reading (`SceneCharacterSource::local_player`):
    the go-to outros the bench reaches instance no characters. To be seen in a story scene
    [ASSUMPTION].
19. A movie's end as a note (`Presented::Fmv`) and the refused skip are converted but not
    reached on the bench [ASSUMPTION].
20. An end only a non-leader console saw (a speech line the leader's audio cut short) is never
    delivered to its scripts: it stays in the round's log. The port's rule, kept; watch for a
    script waiting on it in play.
21. The presentation director's queue is 8 deep natively and a ninth start is dropped. The
    bench's `COOP_COMPLETE` diagnostic completes objectives while an outro scene holds the
    director, and the dropped start was twice the script's own outro group, which the script
    waits on forever (M4's second run, M7's second proof run under `Anyone`); a campaign's
    queue holds 32 now (`presentation::start`). A callout start retried every tick by the
    go-to script still traces the same line each tick while a scene holds the director.
22. The go-to script resets its objectives every tick, so `cMsgObjectiveActivate` and
    `cMsgObjectiveFailed` reach it every tick; heard once per tick now, not traced (it would be
    a line a tick).
23. The mission's end phase waits for its outro scene now (about 700 ticks longer than M3's
    run, whose outro was unpresented), so a bench recipe's sign-up teleports must follow it
    ([tooling](tooling.md), "A typical M4 run").

## Missions ([ledger](missions.md), the crawl of 2026-11-10)

48. Fixed (2026-11-11): the scripts' AI player is an ordinal in the career's own index space,
    which a campaign's scripts now see on every console (56); the AI skates at a slot after the
    humans, the same slot on both seats.
49. `m_6p_01_demo_first_job` ran the same path on both seats 58 ticks apart on 2026-11-10
    (`setup_round` waited for a local teleport). On 2026-11-11 (`2026-11-11-crawl-ai2`, the
    assets preloaded) both seats ran its 14 states at the same ticks, to `end_options` at 11026;
    one run, so the split may return with a slower teleport: the teleport's end as a fact both
    seats wait for stays the fix if it does. The start spots apart (50) still owed.
50. Fixed (2026-11-11, [world](topics/world.md), "A puppet stands up"): a puppet never stood up
    after a knockdown, its own graph not being ticked past the collision's end, and the owner's
    "up" now ends a knockdown this copy still plays. The demo mission's census `down` agreement
    went from 33 % to 99.2 %; a diagnostic shove's puppet stood up with its owner, and so did a
    second knockdown the owner itself caused. Left in the demo: flags agree in 68 % of pairs
    while each seat's skater stands inside the other's spectators and hits them every three
    ticks (the demo's placement, 49); the trace shows hits during a collision rejected
    (`NotAdded`), which is the queue's rule for a repeated major intent.
51. Resolved (2026-11-11): `m_2a_01` and `m_4g_01_spotfilm_johnson` were not waiting after their
    win; the crawl's win at tick 1800 fell inside their intro scene (the pro's to tick 2453, the
    Shut-Down-of-SanVan scene to 4033), before `challenge_running`, so the scripts never saw a
    completion and the 170 s deadline cut them. Rerun with the win after the scene
    (`CRAWL_WIN_AT=3600`, `4500`) both pass through `challenge_cleanup`
    (`../bench/evidence/2026-11-11-crawl-long-intro.md`). A crawl's win must follow the mission's
    own `challenge_running`; the driver could wait for that state instead of a fixed tick.
52. The seven film-team missions (`m_6p_03_demo_pool_team_03`, `m_6p_05_film_team_05`,
    `m_6p_09_photo_sm_04`, `m_6p_10_film_team_10`, `m_6p_10_photo_sm_5`, `m_8_01_04`, `m_8_01_05`)
    complete themselves 182 ticks after the execute (`initial→challenge_running→complete`) and
    autosave with one hash. The crawl counts that a pass; what the mission asks of a player in
    play (a team film the diagnostic cannot make) is unproven.
53. The ambient AI skaters are each console's own: at the same two spots seat 1 spawned
    `ai_skater_06` and `sammy`, seat 2 `seb` and `ai_skater_06` (the probe's `ai:` lines). Not
    streamed, not in the inventory's counts; a player sees different skaters than the other.
54. Fixed (2026-11-11): four location missions (`m_08_01_teleport_trainingfacility_in`/`_out`,
    `m_2a_01_goto`, `m_3a_01_goto`) carry an empty `MapStartLocation`; the execute no longer asks
    for a teleport to the empty name (the native lookup fails on it and moves nobody). The crawl
    completes them by name.
55. Fixed (2026-11-11, the AI day): in a campaign round the AI skater manager believed it was in
    a network game (`LoadGame(true)` on the online path), and `IsActivityAnAiChallenge` is false
    online, so no challenge ever selected its characters at its asset load: every "recipe None"
    of the crawl (45) was this, not a missing port. A campaign's manager now loads as the career's;
    the photo mission `m_5a_01` selects Chris Haslam at its load on both seats, its asset streams
    and the spawn finds the recipe (`../bench/evidence/probe/2026-11-11-crawl-ai-fix2.md`, the
    `ai:` trace). Each console runs its own AI skaters, as every AI is each console's (53).
56. Fixed (2026-11-11, the AI day): the scripts count their AI players three ways: the follow
    script asks for a literal 1 on both seats, the photo script for `GetLocalPlayerID() + 1`
    (seat 2 asked for "player 2"), the demo script for every index but its own (seat 2 asked
    for 0, 2, 3, 4). A local-relative mapping served one convention and broke the others (seat 2's
    follow pro spawned at the other human's slot, `2026-11-11-crawl-ai`). The campaign's scripts
    now see the career's own index space, as the single-player game gives it: `Online.
    GetLocalPlayerID()` answers 0 on every console, `GetNumHumanSkaters()` 1, and every player a
    script hands a native goes through one funnel (`challenge::players::script_slot`): 0 is this
    console's skater, `k` from 1 on the AI's ordinal at slot `N - 1 + k`, the same on every
    console; a `cMsgSkaterCreated` of an AI carries its ordinal. The other humans have no script
    index; what the scripts learn of them comes through the rules and the facts, never by
    number (the design's "the player" ledger, §3). Not routed: the results screen's rows
    (display), `Objectives.MakeObjectivesForPlayer` (the rules' own fan-out), `SetPlayer` (Party
    Play's). Proven by the next crawl's AI families.
57. Fixed (2026-11-11): a campaign dropped every message of the challenge queue (`cMsgSkaterCreated`,
    `cMsgSkaterDestroyed`, the radius pair) since the local skater's are each console's own; the
    follow family's `wait_for_assetload` waits for its AI's `cMsgSkaterCreated` and never heard it
    (`m_4b_01`, the pro spawned at slot 2 and the script sat for good). The messages of this
    console's own AI skaters now reach the scripts, their `PlayerID` the AI's ordinal. Their tick is
    each console's own (the scene load's), a divergence the next crawl measured: of 32 AI
    missions rerun, one (`m_7a_02_jam_monster_public_park`) left `wait_for_assetload` one round
    tick apart (906 and 907) and ran the rest alike; the other 31 ran every state at the same
    tick. A barrier (the load's completion as a fact both seats wait for) would close that tick
    if play ever shows it.
58. Fixed (2026-11-11): a mission hull's own locators (`m_1a_07_follow_slappy_startlocator01` and
    six more `*_startlocat*`, in the hull's `cSim_Global` arena) register with the location manager
    when the hull's simulation collection streams in, a few frames after the crawl's execute; a
    follow script spawns its pro as its first state enters, before that, and failed on "unknown
    location" (47). In play the loading screen loads the challenge's assets before its scripts run
    (`StateLoading::StartChallengeState`); the crawl now does the same: `SKATE2_DIAGNOSTIC_COOP_PRELOAD`
    at round tick 600, the execute at 900. The world's own locators (`BAM_Sim.loc`, 52 of the
    missions' `*_challengelocator01` among them) are now a teleport's fallback when no registered
    arena holds the name (46); none of the crawl's unknown locators was among them, every one was a
    hull's.
59. Five team and pro-film missions and one follow have no competitor on the crawl's save
    (`2026-11-11-crawl-ai3`): `m_08_01_photo_team_shot_1` takes its team from the demo sponsor's
    (`SelectDemoCharacters`, the team of the pro beaten in a throwdown, `pro_beaten_key`), and
    `m_6a_01_filmfriend_70s`, `m_6c_01_profilm_soho`, `m_6d_01_profilm_classic` and
    `m_6a_01_follow_spillway` select from a dynamic competitor list that is empty
    (`SelectCharacters ... 0 competitors`; the asset load rightly counted them no AI challenge).
    Both lists are progression choices made in play (a pro beaten, a friend or team chosen); the
    diagnostic win completes objectives and chooses no one. The bench's limit, not co-op's: a
    save with those choices made, or a diagnostic that sets them, would run them; every one of
    the six stops at the same state at the same tick on both seats.
60. The three `m_1x_01_*_ccenter_dlc` missions are the downloadable content's: their hull loads
    from `missions.big` but their locator (`m_1x_01_ftl_ccenter_DLC_challengelocator_01`) is in no
    archive of this game folder, which has no DLC installed. They stop at the spawn's "unknown
    location" until the content is there.
61. One AI character lookup fails in the port, not in co-op: `m_5b_03_photo_sbm_johnson` names
    its competitor `johnson` and `GetCharacterID` finds no `ai_characters` entry with that
    layout name (`characters [0]`, no recipe, the spawn fails on both seats alike at the same
    tick, `2026-11-11-crawl-ai3`). The native's name match (word 8 of the `ai_characters` layout
    against the profile name) is the place to read.
62. The competing families' scripts follow their AI's outcome, which is each console's own:
    `m_4a_01_skate_carroll_koston` ran 53 states alike on both seats once (`2026-11-11-crawl-ai2`)
    and parted at its 29th state in one of three reruns (`leader_failure` at 5791 on seat 1 and
    later on seat 2; the other two alike), each console's leader failing its trick at its own
    tick. A follow or photo
    mission's flow does not read the AI's run; S.K.A.T.E., jams and contests do. The competing
    kinds need one AI outcome for both consoles (an owner's AI streamed, or its round results as
    facts) before they can be in step; until then they are each console's own game
    ([kinds](topics/kinds.md)).
