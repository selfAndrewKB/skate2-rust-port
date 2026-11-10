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
    note; the measured budget in a busy street decides whether it is built.
34. A puppet's startle, stumble and mood gestures do not travel: its intents are its owner's
    velocity through the speed table, so a startled run shows as a run and nothing else. The
    owner's graph behaviour (a small index) is the next thing to stream if play wants it.

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
11. The tutorials' `eChallengeTypes` family number is unknown; the built-in rule table names
    the opening tutorials by prefix (`m_1a_`).
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
21. The presentation director's queue is 8 deep and a ninth start is dropped (the port's
    rule). The bench's `COOP_COMPLETE` diagnostic completes objectives while an outro scene
    holds the director, so some activation and completion presentations are lost there
    (`not queued ... lost` in the trace); a person cannot complete objectives during a scene,
    so this is the bench's, not play's. A callout start retried every tick by the go-to script
    traces the same line each tick while a scene holds the director.
22. The go-to script resets its objectives every tick, so `cMsgObjectiveActivate` and
    `cMsgObjectiveFailed` reach it every tick; heard once per tick now, not traced (it would be
    a line a tick).
23. The mission's end phase waits for its outro scene now (about 700 ticks longer than M3's
    run, whose outro was unpresented), so a bench recipe's sign-up teleports must follow it
    ([tooling](tooling.md), "A typical M4 run").
