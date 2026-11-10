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
