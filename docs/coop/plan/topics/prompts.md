# Prompts and sign-up (M3)

**Scope:** sign-up boxes, posters, dialogs, Skate 101, unlock overlays: owned by the player who
acts and mirrored on the others through notes on the facts. Front page:
[README](../../README.md). Design: [design](../design.md) §3.3.

## State (2026-11-10, M3) [CONFIRMED on the bench unless marked]

- **A prompt's answer is a note.** In a campaign round a pad's answer on one console (the
  progression dialog's Yes or No, a script popup's option, Skate 101's continue, the unlock
  overlay's continue, the sign-up D-pad press and release, the several-challenges choice, the
  poster's accept and decline: `coop/prompts.rs`) is not applied there; `CareerSession::prompt`
  queues it, it rides the player's next facts as a numbered note (`FactsNotes::prompts`,
  protocol 118), and every console, the presser's included, applies it at the tick the note
  arrives (`apply_prompt`: what the press did, the prompt closed where it is still up). A
  prompt travels until a tick carries it; each is applied once, in order. Outside a campaign a
  press applies at once as before.
- **Any player may answer any open prompt** (the ledger's "the one who did it" is whoever
  pressed first; a rule per surface is M7's if play wants one). The prompts show on every
  console because the scripts run on every console.
- **The box tests read every player.** Sign-up boxes, global activities, discovery volumes,
  dynamic hulls and story-character boxes test every player's tester from the facts, alike on
  every console, so a box any player stands in counts; a Hall of Meat bail of any player
  withholds the sign-up as the local one did.
- **What M3 proved** (`../bench/evidence/2026-11-10-m3-*.log`): after the opening tutorial
  completed on both seats (round tick 3512), both stood in the `m_1a_03_tutorial_fliptricks`
  box; seat 2's D-pad press was applied on both at tick 5701 and opened the poster on both
  (`OpenSignUpPoster`); seat 1's accept was applied on both at tick 6442 and the flip-tricks
  tutorial started on both at 6473 with everyone's objectives. The progression dialog's answer
  goes through the same funnel (the bench's `COOP_ANSWER`), [ASSUMPTION] for Skate 101 and the
  unlock overlay, converted but not reached on the bench.

**Not done:** the results screen's options (Leave, Retry, Adjust Settings), the Hall of Meat
results, the blog, the sponsor and covers screens still apply their continue locally (every
player presses their own, which the scripts tolerate while the facts pace them, but it is not
one answer); the training area beyond the flip-tricks sign-up needs tricks the bench cannot
make ([open items](../open-items.md)).

## History

- 2026-11-10 M3 (commits `af6b504`, `7ad7f51` and the M3 docs): the first press did nothing
  because the sign-up HUD is a hold with a delay (a press, a release, a press), then because the
  prompt is withheld while the mission's end phase (speech lines, a cutscene) is still the main
  challenge; two testers tested in turn loaded and unloaded a hull every tick until the test
  took one set.
