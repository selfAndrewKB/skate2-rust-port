# RE: the campaign's save on the server, a member joining late, a lost connection (read 2026-11-10, M8)

What the port and the fork did with the campaign's save, a member whose career loads after the
round started, and a connection that drops, and the rules M8 gave each. Code:
`crates/skate2-net/src/server.rs` (`start_campaign`, `campaign_message`, `hello`, `leave_game`),
`client.rs`, `protocol.rs`; `crates/skate2-core/src/session/online/coop_campaign.rs`,
`online_challenge.rs` (`start_campaign_round`, `end_online_round`),
`session/selected_session_online.rs`, `selected_session_career.rs` (`save_career`,
`start_career`), `ui/online_config.rs`.

## The save [CONFIRMED by reading]

- `save_career` builds the progression leaves (`signpost_progress::save`: data, position,
  messages; `unlockables.save`; `dynamic_hulls.save`), sends them to the server as the blob
  (M2) and writes them all to `SKATER.P` on a worker thread. The load side (`start_career` at
  Continue Career) reads the same leaves from the file and feeds `unlockables.load`,
  `dynamic_hulls.load` and `signpost_progress::load`, each taking `Option<(version, tuples)>`:
  a decoded blob feeds them directly. Applying leaves to a loaded career in place is not safe
  (`signpost_progress::load` re-inits the signpost and re-activates challenges over running
  scripts; the unlockables only add; the hull loader only unpacks), so adoption happens at
  load, never in place.
- The server kept the newest blob by `tick >= held` (a restarted round's lower ticks would
  have been refused), wrote it to `campaign-<key>.save` and never read the file back: the save
  lived only while the lobby did. The client traced the blob it received and dropped it.
- M8: the save carries the round's session; the server keeps the newest by (session, tick),
  reads the file back when the campaign lobby is made (session 0), and the client decodes the
  blob (`progression_leaves`, the inverse of `progression_blob`, tested) into `campaign_adopt`,
  which `start_career` takes in place of the file's leaves in a campaign, keeping the file's
  orientation leaf and tracing whether the local progression agreed.

## A member joining a running campaign [CONFIRMED by reading]

- The server started the round once every member's career was loaded (two or more, or the
  host's start); `started` was never cleared, and a `ChallengeLoaded` after the start only
  logged "runs its career alone". The late console in fact froze: `online_campaign_entered`
  sets `campaign_hold`, cleared only by the round's start that never came. The facts lockstep
  cannot admit a joiner: the facts coder codes every frame against the previous tick back to
  tick 0, and the career's script state at tick N cannot be sent at all; the port's own
  challenges never admit a joiner either (a late member waits apart).
- M8: restart with everyone. A `ChallengeLoaded` after the start makes the server send every
  member the newest save and `CampaignReload`, clear the start and the loaded set, and start
  the round again once everyone has loaded. A console on `CampaignReload` sets the campaign
  start and covers the fade, which takes the path a game joined from play takes
  (`online_enter_from_play`: `unload_game` tears the career down, `start_online_game` →
  `start_online_campaign`, which now also drops the round, the shared world table, the hold
  and the entered flag), then `start_career` adopts the save and `online_campaign_entered`
  tells the server. The round restarts at tick 0 with everyone; mission progress since the
  last autosave is lost for all.

## A lost connection [CONFIRMED by reading]

- The server knew no identity across connections: a fresh random user id at every sign-in;
  the same socket address with a new nonce dropped the old connection at once, but the client
  binds a new port every time, so that never matched; an old member went only at the
  transport's 10 s or the campaign's 15 s. The client retried the handshake only; an
  established connection lost became `Disconnected`, and the session sent the player to the
  main menu with the career unloaded.
- M8: `online-config.json` holds a `player_id` made once; it rides the hello (protocol 121); a
  sign-in with a player id the server already has drops that old connection at once. A console
  whose connection drops in a campaign keeps its career up but held, signs in again at once
  and rejoins the campaign lobby (`online_campaign_dropped`, the rejoin flag taken at the next
  `SignedIn`); the join brings the server's save and, when the others' round runs, a reload
  for everyone.
