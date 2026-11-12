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
   M6's. **M17 (2026-11-12):** a sample goes only when the receiver's extrapolation of the
   last would be off (5 cm, 2°) or the flags, motion or graph state changed, else every
   twentieth run: the world stream at 1.5 and 1.6 kB/s a seat in the M16 probe's soak, from
   6.0 and 6.2 in the field test (`../bench/evidence/probe/2026-11-12-m16-puppets.md`).
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
    **Fixed (2026-11-12, M16):** every sample carries the owner's AI graph state
    (`EntityState::graph`, stream format 5) and the puppet's graph is held at it every tick
    (`PedestrianGraph::update_delegate_held`): the behaviours' `Begin` and `End` run on the
    puppet as on the owner, so a startle, a stumble, a knockdown, a photographer's pose or a
    sit are the owner's and not a reading of the puppet's own perception; a state the puppet
    cannot enter is traced once and that puppet keeps the M6 gate. With it a puppet predicts
    no hit of its own any more: a contact goes to the owner and the owner's flags and state
    play it back one round trip later (the field test had 63 corrections of predicted
    knockdowns against 38 confirmations on one seat). Proof:
    `../bench/evidence/probe/2026-11-12-m16-puppets.md`.

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
    99.95 %. M16 turns a puppet to each sample's facing over the stream's cadence (three
    ticks, `YAW_SLEW_TICKS`) instead of snapping and holding it; the M16 probe's same-tick
    census audit reads yaw within 5° in 93.9 % of pairs (the field test's 91 %), speed within
    0.5 m/s in 74.8 % (71 %), position within 0.5 m in 100 %; the live audit's own figures
    are no longer comparable since M17 (it sees only the samples the send-on-error rule sent). The puppet's velocity is its drive's own (the coast, then the hold); carrying the
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

## Play (2026-11-12, the player, M12 build 4be2feea)

The player's field test on the lobby (muted, the waterfront, then `m_2d_01_photo_tm_burnette`,
"A Little Help Here", from its sign-up by the host): the seat logs are kept as
`../bench/evidence/probe/2026-11-12-field-test-seat-{1,2}.log` (the round `1639400822`, the
mission's graph from seat 1's line 43582 and seat 2's 46693).

70. **The photographer stands in two places** (the player, in play). A story character is
    every console's own (`ownership::Kind::StoryCharacter` is `Rule::EachConsole`, M5). The
    host's photographer is the intro scene's: the scene instanced him for player 0 (seat 1's
    line 43605, `spawned as pedestrian 33557996`, at the scene's tick 936383). The joiner's
    scene instanced no characters (`instanced 0 characters for this console's local player
    index Some(1)`), so its photographer is the challenge's own spawn at the photographer
    locator, 1,400 ticks (23 s) later (seat 2's line 47095, `33558129`). From then on each copy
    walks its own waypoint and runs its own AI graph. **Fixed (2026-11-12, M14):** a story
    character is its spawner's (the console whose tester box reached the entry first) and
    streams as a pedestrian, wherever the players are, its descriptor naming the challenge's
    entry (stream format 4); the other consoles hold the puppet in that entry and spawn no copy
    of their own; two testers at once keep the lower slot's copy ([world](topics/world.md)).
    Proof: `../bench/evidence/probe/2026-11-12-m14-story.md` (the photo mission: the joiner's
    tester first, the host's puppet of it at the same spot and facing to the centimetre, no
    second copy, the mission completing on both); the tie fired in the regression's flip-tricks
    tutorial (`2026-11-12-crawl-m14`: both testers in one tick, the host kept its copy, the
    joiner culled its own and took the puppet); the owner's despawn freeing the entry is
    unexercised. The intro scene's own characters stay each console's scene's.
71. **The joiner's objective copy dies at the host's first bail** (the logs; the player saw
    the joiner unable to complete). Every console's scripts hear every human's skater messages
    from the lockstep frames with the message's `PlayerID` the campaign slot, and the shipped
    handlers compare it with `vars.LocalPlayerID`, which `Online.GetLocalPlayerID` answers as
    0 on every console (item 48): so the host's events are "the player's" to the joiner's
    scripts and the joiner's own are nobody's. On the host's bail seat 2's script failed its
    group too (`HoM start player=Some(0)` then `ObjectivesFailed ... decided here` on seat 2),
    the note carried only the group's ordinal, and the receiver resolved it against the
    sending console's player: the joiner's copy. Seven times the joiner's `ObjectivesFailed`
    landed exactly 502 ticks (the lockstep lead) after the host's (ticks 937566/938068,
    938650/939152, ...), and the restart flow re-armed the host's copy alone (the script holds
    the first player's group), so from tick 938068 to the mission's end at 956707 the joiner
    had no live objective, and the restart at 958635 re-made the host's copies alone. **Fixed
    (2026-11-12, M13):** a human's skater message reaches the graphs as player 0's, and while
    a graph handles it the group natives act on that human's copy (`World::event_player`,
    `objectives::subject_copy`), the state changes and timers it starts carry that human, and
    a forced outcome in reaction to it is applied where it is made, at the event's tick, on
    every console (a note of it landed a lockstep lead after the restart timer); a decision
    outside any human's event travels as a note naming its player (protocol 125). Proof:
    `../bench/evidence/probe/2026-11-12-m13-bail.md`.
72. Whether the joiner's copy of a photo objective arms when the joiner skates into the zone
    is unproven: in the field test seat 2's skater made one trigger event in the whole mission
    against the host's 250 (it stood in its start volume), and the crawl completes copies by
    the diagnostic, never by a skater in a volume. The two players' trigger events name
    different volume ids for the mission (`4ea7103e38706` and `4ea6c03e38706`); the objective
    arms on the attribute's scoring boundary id. **Answered (2026-11-12, M13 and M15):** the
    M13 probe's seat 2, teleported into the Burnette photo zone, armed its copy on both seats
    (the same volume id as the host's that time); M15's crawl gave the diagnostic win to the
    joiner's seat alone (`CRAWL_WIN_SEAT=2`) in the six `Anyone` missions of the regression
    set, and every one completed on both seats from the joiner's notes
    (`../bench/evidence/probe/2026-11-12-m15-joiner-wins.md`). What a joiner's own trick in a
    photo zone does on the host remains play's to judge (the bench does no tricks).
73. A story character's pedestrian id is drawn from the same counter as the census's, and the
    world stream names an owned pedestrian by its owner's slot and local id: seat 2's
    photographer `02000e71` was later also the host's census pedestrian `02000e71`
    (`Pedestrian 02000e71 None -> Remote(0)` on seat 2 at line 47892, a puppet of another
    entity), so the ownership table's entry for the story character read as the host's. Not
    seen to break anything in this log. **Fixed (2026-11-12, M14):** an owned story character
    is named on the wire by its owner's slot as every streamed pedestrian is (`world_id`), and
    the ownership table holds that name.
74. The Hall of Meat manager tracked `PlayerID` 0's bails as the local skater's on every
    console (`hom::receive`, the single-player 0), so the joiner's own bails (slot 1) were never
    tracked there and the host's were. **Fixed (2026-11-12, M13):** the manager's player is the
    campaign's own slot.
75. The field test's bandwidth, from the stream traces of 4.9 hours: the world stream 6.0 and
    6.2 kB/s a seat on average, 7.2 and 9.9 kB/s at the peak; 27 B a pedestrian sample at 20 Hz
    (about 570 B/s a streamed pedestrian); a skater's look 31 kB each, 318 sent (9.9 MB), one
    at every ambient respawn; the skater frames and the facts uncounted. **M17 (2026-11-12),
    measured on every channel** (`net bytes` trace, `skate2_net::client::ChannelBytes`) over
    the M16 probe's five-and-a-half-minute soak: the world stream 1.5 and 1.6 kB/s a seat (the
    send-on-error rule, item 2); the owned skaters' frames the largest channel, 7.8 and 4.1
    kB/s (from 12.5 and 6.0 before the ambient skaters' poses went to half the cadence); the
    player's own frames 2.7 kB/s; the facts 1.7 kB/s; the looks 31 kB each at every ambient
    respawn. A look (32,566 bytes, `SKATE2_DIAGNOSTIC_DUMP_LOOK`) compresses to 27 % under
    deflate and 22 % under lzma, 81 % under a zero-run RLE: a compressor or a dedup by content
    hash (the ambient outfits repeat) is the next step, not built; neither is delta coding
    against acknowledged runs (item 33), which the stream no longer needs for its budget.
76. **A script's state and vars are one per activity, not per player** (M13's residue): the
    graph's state (`challenge_running`, a restart state) and its `vars` table (bail counts,
    timers) are shared by both humans' events, so two humans bailing within one restart
    interleave in the one state machine. A skater message names the subject, and the state
    changes and timers a handler of it starts carry it (the bail's restart runs from a timer:
    without that, the M13 probe's run 4 saw the joiner's bail reset the host's copy and leave
    the joiner's failed); a handler reached by none of those (a scene's end, a HUD press) acts
    on the held group, the first player's copy; a human with no copy of the activity (one
    made before they joined) drives the first player's copy. Also the central message of a bail
    (`ID_CHALLENGE_CENTRAL_MESSAGE_OBJECTIVE_BAILED`) shows on every console for either
    human's bail, as the design's "everyone" kind has it; play may want it the bailer's alone.

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
53. **Fixed (2026-11-12, M12).** The ambient AI skaters were each console's own: at the same
    two spots seat 1 spawned `ai_skater_06` and `sammy`, seat 2 `seb` and `ai_skater_06`, and
    a player saw different skaters than the other. Every console now streams its own ambient
    skaters as it streams the challenge AI it owns, under its own slots, and shows the others'
    as puppets with their sound; an owned skater's name carries its owner on the wire
    (protocol 124), and each console spawns its share of the allowance (the original's three
    split by player index: two and one for two players). They stay no tester (68) and no frame of the scripts' (69). Proof
    (`../bench/evidence/probe/2026-11-12-m12-soak-seat-{1,2}.log`, build 930d1ba8): four
    minutes at the waterfront on the first build (each seat two skaters, before the split),
    each seat sending each one's look as it spawned (ten and twelve looks) and building the
    other seat's as puppets (twelve and ten characters) whose frames carried their sound, then
    the tagged build 4be2feea with the split (two skaters on seat 1, one on seat 2, three looks
    each way; `probe/2026-11-12-m12-ambient-skaters.md`); `-m12-handoff-seat-{1,2}.log`: the
    S.K.A.T.E. with the owner leaving, the pros' puppets named by player 0 taken over by seat 2
    at tick 2821 under its own name; the nine-mission regression `2026-11-12-crawl-m12` [see
    the run note in missions]. What a player will see: a respawn at a slot re-dresses the
    puppet in place (a new look, a jump to the new spot) within a second; whether that reads
    well is play's to say. Not yet played by the player.
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
    ([kinds](topics/kinds.md)). M9 (2026-11-11) takes the second way: a script's decision about
    its own player (an objective forced complete, the objectives failed, the mission won or lost
    on its own terms) rides that player's facts as an outcome note and every console applies it
    to that player's copy at the note's tick; the rules settle the mission. Proven with the
    bench's win on seat 1 alone: both seats applied the two notes at tick 1884 and entered
    `challenge_cleanup` together (`../bench/evidence/probe/2026-11-11-m9-one-seat-win-*.log`).
    The AI's run itself stays each console's own; a S.K.A.T.E. still plays its rounds twice,
    once per console, and the first human to win (or every human, by the rule) ends it for all.
    M10 (2026-11-11) takes the first way as well: the first player's console owns the challenge
    AI skaters, its facts carry their outputs and the others show puppets of its frames
    ([ai-skaters](topics/ai-skaters.md)); the notes stay, for the decisions a script makes
    about its own player. Resolved on the bench: three reruns of the S.K.A.T.E. on the M10
    build ran 57 states alike on both seats through `leader_failure`
    (`../bench/evidence/probe/2026-11-11-crawl-m10-skate-{1,2,3}.md`).
63. A player who leaves a round and rejoins (a new stint, its note numbers starting again at 1)
    has its first prompts and outcomes dropped on the consoles that keep the old high-water
    marks (`prompts_applied`, `outcomes_applied` in `Round`, never reset for a slot). Seen in the
    M9 review, not on the bench. The fix is to reset a slot's marks when that player's stint
    changes.
64. Fixed (2026-11-11, M11): `CampaignWorld.players` never shrank when a player left, so a
    rule's count of copies (`Everyone`'s completion, `Anyone`'s failure, the won and lost
    notes') was one more than the players left and could never be met. The leave handling
    (`online_player_disconnected`) now removes the leaver from `players`; the rules count the
    objective copies of the players still in the round (`present_siblings`), and the won notes
    count the players left. The slot space does not shrink with them: `CampaignWorld.seats`, the
    round's seat count, is the humans' and the AI skaters' boundary everywhere `players.len()`
    had been. Proof: the hand-off probe (65), seat 2 alone after seat 1 left, "1 of 2 seats
    left"; the nine-mission regression `2026-11-11-crawl-m11` [see the run note in missions].
65. **Fixed (2026-11-11, M11): the AI skaters' hand-off.** The owner was the first player and
    never changed: when it left, its AI skaters' facts and frames stopped, the other consoles'
    scripts saw the slots destroyed and the mission had no AI. Now the owner is the lowest
    player still in the round (`ai_owner` over the shrinking `players`, 64), decided at the
    tick every console delivers the leave. Until the next owner's facts carry the AI, the
    round keeps the leaver's last owned frames in the tick's frames (`Round.ai_handoff`, at
    most `AI_HANDOFF_GRACE` = 600 ticks, then the slots end), so no console's scripts see a
    destruction. The new owner spawns the skaters afresh from the spawn commands it had dropped
    as the old owner's (`SessionAi.streamed_spawns`, kept by slot while streamed), each at the
    last frame the old owner's facts carried (its path and behaviour start over), drops their
    puppets, and from its next facts carries their frames and streams their bodies; every other
    console keeps its puppets for the new stream (the same slot ids, their receivers already
    dropped at the leave). `SKATE2_DIAGNOSTIC_COOP_LEAVE=<round tick>` makes a seat quit.
    Proof (`../bench/evidence/probe/2026-11-11-m11-handoff.md`, build 44c1c62a): the S.K.A.T.E.
    with seat 1 quitting at round tick 2700; seat 2 delivered the leave at 2821 with two pros
    standing, owned them from 2822, spawned Koston and Carroll at their last frames, carried
    them in its facts the same tick (its lead gone with the other player), sent their looks,
    and played four more rounds with the camera switching to each leader; the scripts heard no
    creation or destruction across the hand-off. What a new owner starts over is the AI's
    routine (a leader mid-trick begins its run again); a script's camera switch to the AI in
    the ticks between the take-over and the actor's build would find no actor (the M10 first
    build's stop), not seen here. Not yet played by the player.
66. **Fixed in part (2026-11-12, M11.1).** An owned AI skater's puppet was silent: no audio
    record travelled with its frames (the remote players' do), so the pro's wheels and landings
    sounded only on the owner's console. The owner now puts the actor's audio physics record
    (`try_audio_phys_out`, relative to its root as the player's own travels) in every owned
    frame, and the puppet sounds through the audio's remote skaters as a remote player does;
    the one the console follows is the audio's local skater, so the microphone follows the
    leader a S.K.A.T.E. switches the camera to. Also fixed: the look went out twice per spawn
    (at the first frame and again when the slot's creation reached the owner through the
    round's frames); the sent record now clears on a slot's destruction alone, and a look goes
    once (31 KB). Proof: `../bench/evidence/probe/2026-11-12-crawl-m11-1-probe.md` (the
    S.K.A.T.E.: two looks for two pros where there had been four, seat 2 tracing each pro's
    frames carrying its sound; the Dyrdek demo: four looks for four). **Still open:** the AI's
    own audio words (`AIOut_Audio`: its call-outs and gesture cues, `controller_audio`) do not
    travel; the wire frame has no field for them, so a pro's call-out sounds on the owner alone.
67. **Props an AI skater moves** (the player, 2026-11-11, in play): a railing an AI skater
    knocks down on one instance stands on the other, and the AI there runs through it as if it
    were not there. The first reading (that an AI's hit claims nothing) was wrong: the scoring
    pass marks a prop touched for any skater body's contact, the AI's included, and the M10
    S.K.A.T.E. logs show the pro's hits claiming two props on seat 1 during the leader's run
    (claims at seat 2's ticks 2421 and 2502, the crawl's human idle, both puppets on seat 2
    within 0.04 m of the sample). What the M9 logs show instead is a contest: with each
    console's own AI copy, one rail (`ad4ab2b1`) changed owner twelve times in forty frames
    (claims 1 to 12 between seat 1's frames 2799 and 2835, every touch on the non-owner taking
    it back with the next claim number), the other rail's puppet up to 4.7 m behind its sample,
    each console's copy a puppet one tick and a body the next; that is a prop down on one
    screen and standing on the other. And the sighting's plain cause, found by the M10.1
    probe (run 4): since M5 the receiver passed a prop sample only when its claim beat the
    known one, which the owner's own continuing samples never do, so a puppet prop took the
    first sample of each claim and no other, the sample of the touch's own tick, before the
    prop had moved; the owner's copy fell and the puppet stood where it had been (every
    convergence trace in every kept log is "sample 1"). M10.1 takes the owner's continuing
    samples (run 5: samples 1, 61, 121 applied, the puppet 0.1 m behind); M10 ends the contest
    for the challenge AI (one owner: two claims, no flip in three S.K.A.T.E. runs; the
    non-owner's AI puppet has no body to hit anything with, by design); M10.1 closes it for
    the rest (two humans, or each console's own ambient skaters, 53): a touch does not take a
    prop whose owner's latest sample, within 30 ticks, still has it moving; the touch waits,
    the puppet stays driven, and the next touch after it rests claims it
    ([world](topics/world.md)). **And the rest (2026-11-12, M11.2):** a car's or a
    pedestrian's moving push is a touch too (the scoring pass's living-world contacts of a
    vehicle's or a pedestrian's body with a relative velocity; a prop resting against another
    claims nothing), so a prop one of those moves is claimed for the console that simulates the
    pusher and streamed, the claim's trace naming the push. Proof
    (`../bench/evidence/probe/2026-11-12-m11-2-soak-seat-{1,2}.log`, build d7185887): six
    minutes at the waterfront with 15 to 17 pedestrians, seven pushes claimed props (three on
    seat 1, four on seat 2), each a puppet on the other seat; one prop (`b33abfb8`) changed
    hands between the seats' pedestrians four times under the contest rule. The nine-mission
    regression `2026-11-12-crawl-m11-2` [see the run note in missions]. Not yet played by the
    player.
68. On the M10 regression run (`2026-11-11-crawl-m10`) seat 2 unloaded and reloaded the hub's
    hull `ur_hotel_pool_drain` (its graph restarting at `initial`) once more than seat 1 after
    the win of `m_1a_03_tutorial_fliptricks` (at round tick 2057) and `m_6p_03_demo_pool_team_03`
    (1891), each time right after an `UnloadDynamicHull`; the missions' own state sequences and
    the saves agree, and the M9 run of the same missions showed no extra reload. **Read and
    fixed (2026-11-11, M10.2):** `ur_hotel_pool_drain` is a location global, run while a
    tester's box is in its bounds and terminated when every tester's box has left
    (`perform_bbox_tests`); the testers were every state in `player_states`, built from every
    frame of the tick, and since the AI day those frames carry this console's own ambient
    skaters (each console's own, 53), so an ambient skater wandering into the hotel pool's
    bounds on one seat ran the global there and leaving it terminated and reloaded the hull
    there alone; the sewer film's hull `f_1a_13_ots_sh06_sewer` cycled the same way (three
    times on seat 2 in the pool demo, never on seat 1). The original's box tests are the local
    tester's, not any skater's. `player_states` now holds the campaign's players alone, so the
    sign-up boxes, the location globals, the discovery volumes, the dynamic hulls and the Hall
    of Meat check read the humans. Proof: `../bench/evidence/probe/2026-11-11-crawl-m10-2-probe.md`
    (the two missions again: the location global's run and suspension traced at the same points
    on both seats, two testers each time, no sewer hull on either seat, both missions passing),
    then the nine-mission regression `2026-11-11-crawl-m10-2` [see the run note in missions].
69. **Each console's ambient skaters reach its scripts as AI players** (2026-11-11, the item 68
    probe): the frames carry this console's own ambient skaters at the slots after the humans,
    and `online_round_ai_messages` posts `cMsgSkaterCreated`/`Destroyed` for every AI slot that
    appears or goes, so seat 1's scripts heard "AI player 1, 2, 3" created at round ticks 945,
    1005 and 1065 in `m_1a_03_tutorial_fliptricks` and seat 2's at 986, 1046 and 1106 (the
    spawner's 60-tick cadence, each seat's own clock), with the respawns at different slots on
    each seat (seat 1's slot 3 at 2058, seat 2's slot 4 at 2156); in the pool demo seat 1 had
    three present at the round's first tick and seat 2 gained them at 26, 86 and 146. Both
    missions pass, so these scripts do not read them; the follow family's wait for its pro's
    creation reads the ordinal, which an ambient skater can hold (the AI day's count, 56),
    and a script that counts AI skaters would count each console's own. The original posts the
    same messages for its ambient skaters, so the messages are faithful; what differs is the
    population. **Fixed (2026-11-11, M10.3):** in a campaign this console's ambient skaters are
    no frame of the scripts' at all: the round's frames are the humans' and the owned challenge
    AI's, so on every console the scripts read the same skaters, and an ambient skater's
    creation, position and tricks reach no script; the ambient count is traced per console
    instead. Proof: `../bench/evidence/probe/2026-11-11-crawl-m10-3-probe.md` (the flip-tricks
    tutorial with no AI creation on either seat where each had heard three, the pool demo, and
    the S.K.A.T.E. with the owned slots in the facts of tick 2133 on both seats and the pro's
    creation at 2134 on both), then the nine-mission regression `2026-11-11-crawl-m10-3` [see
    the run note in missions]. What a script could still see of an ambient skater is through
    the AI natives by slot (pins, speech, repositions) if a mission addressed a slot an ambient
    skater holds on one console; none of the crawled missions does.
