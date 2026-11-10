# Saves, identity and resume (M2, M8)

**Scope:** the server's campaign blob, each console's own `SKATER.P` beside it, a stable
per-player id, resume at the same progression, a member joining a running campaign, reconnect.
Front page: [README](../../README.md). Design: [design](../design.md) §3.7, §3.8.

## State (2026-11-10, M8) [CONFIRMED on the bench unless marked]

**What M8 proved, resume** (`../bench/evidence/2026-11-10-m8a-*.log`): both seats played the
opening tutorial and saved at round tick 6072 with one hash (`e6a2db1d166eb003`; the server:
"Seat 2 agrees"); the server wrote `campaign-636f6f702d63616d.save` (130246 B); everything
stopped; the server alone came back and read the file ("campaign save read from ... a member's
newer one supersedes it"); each seat, started from its primed profile, received the save at
the join ("4 leaves, held for the career's load") and adopted it at load ("the server's save
adopted (session 0 tick 0); the local file's progression agreed"); the round started with both.

**What M8 proved, a late join** (`../bench/evidence/2026-11-10-m8b-*.log`): seats 1 and 2 in
round 2816581293; seat 3 joined and loaded its career; the server: "Seat 3 loaded its career
after campaign round 2816581293 started: every member reloads from the newest save and the
round restarts"; all three received the save and "the server asks for a reload", took their
careers down and up again adopting it, and round 2816581294 started with 3 players.

**What M8 proved, the server restarted under a round** (`../bench/evidence/2026-11-10-m8c-*.log`):
the server died at round 3325878957; both seats traced "the connection was lost; the career
waits, signing in again to rejoin", tried every three seconds while it was down ("Connection
refused"), and when it came back signed in with their player ids; the server read the save
file, made the campaign lobby, seated both, and round 1064924775 started with 2 players.

**What M8 proved, the crawl's first slice** (`../bench/evidence/2026-11-10-crawl-m1a.md`): the
eight opening missions, one bench start each with the saves carried forward (`BENCH_KEEP`),
each executed by name at round tick 900 and won by diagnostic at 1800: `m_1a_01` to `m_1a_04`,
`tutorial_scoring` and `tutorial_offboard` reached `challenge_cleanup` on both seats at the
same tick with the saves agreeing (the tutorials autosave nothing: "no save"); the two photo
missions stopped at the port's AI-skater blocker ([kinds](kinds.md)). Sixty seconds a mission
on the bench.

- **The blob.** Every autosave sends the progression leaves (SignPost's data and messages, the
  unlockables, the dynamic hulls; never the skater's own orientation) as one byte string with
  its FNV-1a hash, the round's session and tick (`CampaignSave`); the server keeps the newest
  by (session, tick), says whether every member's hash agrees, writes it to
  `campaign-<key>.save` beside itself, and reads that file back when the campaign lobby is
  made (as session 0, so any member's live save supersedes it): a campaign survives the
  server's restart. The blob decodes back into leaves (`progression_leaves`, tested).
- **Resume: the server's save is the truth.** A joining member receives the newest blob before
  its career loads; `start_career` in a campaign, on Continue Career, takes its leaves in place
  of the local file's (the orientation stays the file's) and traces whether the local
  progression agreed or differed; New Career starts from nothing, as it does alone. Either player can still continue alone from their own `SKATER.P`, which the next
  autosave brings up to the campaign's progression.
- **Identity.** Each console's `online-config.json` holds a `player_id` (16 hex digits, made
  once on first use); it rides the hello (protocol 121). A sign-in with a player id the server
  already has drops that old connection at once, so a returning player's slot frees without
  the transport's 10 s or the campaign's 15 s.
- **A member joining a running campaign.** The server, on a `ChallengeLoaded` after the round
  started, sends every other member the newest save and `CampaignReload` (the joiner's load,
  from that save, stands), clears the round's start, and starts it again once they have loaded:
  each of those consoles takes its career down behind the fade and starts it again from the
  server's save, exactly as a game joined from play is entered; the round restarts at tick 0
  with everyone (the net test `campaign` covers the reload's recipients and the restart with
  three; the `m8b` run predates the review and reloaded the joiner too). Mission progress since the last
  autosave is lost for all, as a reload would lose it.
- **Reconnect.** A console whose connection drops in a campaign, its career loading or up,
  keeps the career (held: no scripts tick without the round), signs in again after a second and rejoins the campaign
  lobby, trying again every three seconds while the server is not back; the join brings the
  server's save and, if the others' round runs, a reload for everyone.

**Not done:** a save request before a reload (the newest autosave is used as held); a notice
for the player while the rejoin runs; a campaign per lobby on one server (one file per
server, by the lobby's one key).

## History

- 2026-11-10 M8: the first rejoin run (`2026-11-10-m8c-*.log`) signed in once, at the moment
  the server was still down ("Connection refused"), and never again; the sign-in now retries
  every three seconds.
- 2026-11-10 M8: the study found the server's file never read back, keyed by a constant, with
  `tick >= held` rejecting a restarted round's saves; the client dropping the blob; a late
  member's career frozen for good behind `campaign_hold`; a lost connection sending the
  player to the main menu. All replaced as above
  ([research](../../research/RE-resume-join-reconnect.md)).
