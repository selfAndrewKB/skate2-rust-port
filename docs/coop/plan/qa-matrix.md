# QA matrix: seats × machines × situations

**Scope:** the combinations the co-op campaign must hold, and which have been exercised, so a
claim is never wider than its run. Front page: [README](../README.md).

| Seats | Machines | Situation | Exercised |
| --- | --- | --- | --- |
| 2 | one Mac, two instances | lobby, Continue Career, both skaters visible | M0, 2026-11-10 [CONFIRMED on the bench] |
| 2 | one Mac | the waterfront street: pedestrians streamed both ways, a shove's round trip | M1, 2026-11-10 [CONFIRMED on the bench] |
| 2 | one Mac | New Career from the lobby (the intro movies, Create-a-Skater) | not run [UNKNOWN] |
| 2 | one Mac | the first mission completed in the lobby (progression), identical saves, the server's agreement | M2, 2026-11-10 [CONFIRMED on the bench]; the steps a teleport cannot prove completed at the same tick by diagnostic |
| 2 | one Mac | a sign-up by one seat, the poster accepted by the other, the next tutorial started on both | M3, 2026-11-10 [CONFIRMED on the bench] |
| 2 | one Mac | the opening tutorial's three outro scenes and seven speech lines started and ended at the same round ticks on both seats, each objective's presentation once, the end phase waiting for its outro, then M3's sign-up | M4, 2026-11-10 [CONFIRMED on the bench]; own skater in a scene, a movie, a skip [ASSUMPTION] |
| 2 | Mac + Windows over the internet | any | not run; needs the same commit and `release-secret.txt` on both |
| 3-4 | one machine | lobby | not run (the lobby takes 4) |
| 2 | one Mac | a seat leaves and rejoins; the server restarted | not run |
| 2 | any | played by a person with a pad | never yet |

Transitions to play in each row: a hull reload (the 160-tick stall at Continue Career), a map
travel, a scene, the pause menu, a save.
