# AI skaters: owned and streamed (M10)

**Scope:** who simulates a mission's challenge AI skaters in a campaign (the pro you follow,
the photographer's subject, a demo's judges and competitors, S.K.A.T.E.'s leaders), how the
owner's frames and facts reach the other consoles, what the scripts see of them there, and what
is still each console's own. Front page: [README](../../README.md). Design:
[design](../design.md) §7 ("The AI skaters: one owner"). The day the AI skaters were found
and the index space settled: [the AI day](../reports/2026-11-11-ai-skaters.md),
[kinds](kinds.md).

## State (2026-11-11, M10) [the proof lines name their evidence]

- **One owner.** A campaign round's challenge AI skaters are one console's: the first player's
  (`CampaignWorld::ai_owner`, the table's lowest index; one rule, read nowhere else). The owner
  runs them as the career does (the AI skater manager, the controllers, the paths); every other
  console takes the spawn's row and slot (`AISkaterManager::spawn_challenge_skater`, its
  `post`) and builds no actor (`try_dispatch_ai_messages`, traced "is its owner's: no actor
  here"), so `GetSkaterCount`, the names and the character answer alike everywhere.
- **The facts carry the AI.** The owner's facts of each tick carry its challenge AI skaters'
  completed outputs at their slots (`ClientMessage::Facts.owned`, protocol 123, each slot coded
  by its own `FactsCoder` against its previous tick, as the player's own frame is); every
  console's round delivers them with the owner's frame at that tick (`Round::next_tick`), so the
  scripts of every console read one AI run: the same trick landed, the same letter, the same
  tick. Each console's ambient skaters still join its own frames alone (open item 53).
- **Creation and destruction are the frames'.** An AI slot's first frame in a tick's frames is
  `cMsgSkaterCreated` to the scripts, its last `cMsgSkaterDestroyed`, with the AI's ordinal as
  the scripts number it (`online_round_ai_messages`), the same tick on every console, the
  owner's included; the local challenge queue's skater messages are dropped in a campaign.
- **The body is a puppet.** The owner sends each AI skater's look once per actor
  (`OwnedRecipe`, kept by the server for joiners) and its replay frames in runs of the cadence
  its own skater's take (`OwnedFrames`, relayed as `RemoteFrames` under the marked slot
  `owned::frames_slot`, acknowledged under `OwnedFrameAck`); the other consoles drive a
  `RemoteSkater` of it (`protocol::owned::skater_id`), shown while the round runs, hidden with
  its last facts frame, dropped with the round. No camera, audio record or trick HUD travels
  with it.
- **The natives.** A script's AI natives on a non-owner reach the manager (the row, the slot)
  and no actor: a teleport, reposition or request of a slot without a controller does nothing
  there (`try_teleport_ai_player` false, the controller map empty at the slot), and the owner's
  frames bring the outcome.

## Not shared, recorded

- The AI's sound on the non-owner (no audio record travels with an owned frame).
- A handoff when the owner leaves the round: the AI stops with it (open item 65).
- The puppet's scoring body (`RemoteScoringSkater`) is collected as a remote player's; whether
  a trick over the pro scores alike on both consoles is play's to say.
- A prop the owner's AI knocks over is not claimed for the owner (the prop rule counts only
  the console's own skater's touch), so it stands on the other console and the puppet, which
  has no body, runs through it (open item 67, seen in play).

## History

- 2026-11-11 M9: the competing kinds settled through outcome notes while each console ran its
  own AI ([kinds](kinds.md)); the AI's run itself stayed each console's own.
- 2026-11-11 M10: one owner, the AI in the facts, puppets elsewhere (this page).
