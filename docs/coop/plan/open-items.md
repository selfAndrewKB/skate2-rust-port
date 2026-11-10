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

## Prompts ([topic](topics/prompts.md))

15. A note is applied one to two seconds after the press on the bench (the round's processed
    tick runs that far behind the facts produced); measure in play, and tune the ready lead if
    it shows.
16. The results screen's options, the Hall of Meat results, the blog, the sponsor and covers
    screens still apply their continue locally.
17. The sign-up HUD's own D-pad hold and delay are kept as the original's: a press is a note, a
    release is a note; holding is not modelled across the link.

## Scenes ([topic](topics/scenes.md))

9. `career.nis.set_online_game` stays false in a campaign: scenes pause the simulation as
   offline. M4 decides the online rule.
