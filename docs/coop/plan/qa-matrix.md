# QA matrix: seats × machines × situations

**Scope:** the combinations the co-op campaign must hold, and which have been exercised, so a
claim is never wider than its run. Front page: [README](../README.md).

| Seats | Machines | Situation | Exercised |
| --- | --- | --- | --- |
| 2 | one Mac, two instances | lobby, Continue Career, both skaters visible | M0, 2026-11-10 [CONFIRMED on the bench] |
| 2 | one Mac | the waterfront street: pedestrians streamed both ways, a shove's round trip | M1, 2026-11-10 [CONFIRMED on the bench] |
| 2 | one Mac | New Career from the lobby (the intro movies, Create-a-Skater) | not run [UNKNOWN] |
| 2 | one Mac | the first mission completed in the lobby (progression) | M2 |
| 2 | Mac + Windows over the internet | any | not run; needs the same commit and `release-secret.txt` on both |
| 3-4 | one machine | lobby | not run (the lobby takes 4) |
| 2 | one Mac | a seat leaves and rejoins; the server restarted | not run |
| 2 | any | played by a person with a pad | never yet |

Transitions to play in each row: a hull reload (the 160-tick stall at Continue Career), a map
travel, a scene, the pause menu, a save.
