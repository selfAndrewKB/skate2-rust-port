# Saves, identity and resume (M2, M8)

**Scope:** the server's campaign blob, each console's own `SKATER.P` beside it, a stable
per-player id, resume at the same progression, a member joining a running campaign, reconnect.
Front page: [README](../../README.md). Design: [design](../design.md) §3.7, §3.8.

## State (2026-11-10, M8 in progress) [ASSUMPTION: built and tested in units; the bench runs are next]

- **The blob.** Every autosave sends the progression leaves (SignPost's data and messages, the
  unlockables, the dynamic hulls; never the skater's own orientation) as one byte string with
  its FNV-1a hash, the round's session and tick (`CampaignSave`); the server keeps the newest
  by (session, tick), says whether every member's hash agrees, writes it to
  `campaign-<key>.save` beside itself, and reads that file back when the campaign lobby is
  made (as session 0, so any member's live save supersedes it): a campaign survives the
  server's restart. The blob decodes back into leaves (`progression_leaves`, tested).
- **Resume: the server's save is the truth.** A joining member receives the newest blob before
  its career loads; `start_career` in a campaign takes its leaves in place of the local file's
  (the orientation stays the file's) and traces whether the local progression agreed or
  differed. Either player can still continue alone from their own `SKATER.P`, which the next
  autosave brings up to the campaign's progression.
- **Identity.** Each console's `online-config.json` holds a `player_id` (16 hex digits, made
  once on first use); it rides the hello (protocol 121). A sign-in with a player id the server
  already has drops that old connection at once, so a returning player's slot frees without
  the transport's 10 s or the campaign's 15 s.
- **A member joining a running campaign.** The server, on a `ChallengeLoaded` after the round
  started, sends every member the newest save and `CampaignReload`, clears the round's start,
  and starts it again once every member has loaded: each console takes its career down behind
  the fade and starts it again from the server's save, exactly as a game joined from play is
  entered; the round restarts at tick 0 with everyone. Mission progress since the last
  autosave is lost for all, as a reload would lose it.
- **Reconnect.** A console whose connection drops in a campaign keeps its career up but held
  (no scripts tick without the round), signs in again at once and rejoins the campaign lobby;
  the join brings the server's save and, if the others' round runs, a reload for everyone.

**Not done:** a save request before a reload (the newest autosave is used as held); a notice
for the player while the rejoin runs; a campaign per lobby on one server (one file per
server, by the lobby's one key).

## History

- 2026-11-10 M8: the study found the server's file never read back, keyed by a constant, with
  `tick >= held` rejecting a restarted round's saves; the client dropping the blob; a late
  member's career frozen for good behind `campaign_hold`; a lost connection sending the
  player to the main menu. All replaced as above
  ([research](../../research/RE-resume-join-reconnect.md)).
