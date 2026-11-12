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
  tick. Each console's ambient skaters are each console's own (open item 53) and, since M10.3,
  out of the scripts' frames altogether.
- **Creation and destruction are the frames'.** An AI slot's first frame in a tick's frames is
  `cMsgSkaterCreated` to the scripts, its last `cMsgSkaterDestroyed`, with the AI's ordinal as
  the scripts number it (`online_round_ai_messages`), the same tick on every console, the
  owner's included; the local challenge queue's skater messages are dropped in a campaign. Each
  console's own ambient skaters are no frame of the scripts' in a campaign (M10.3, open item
  69: their creation had reached the scripts as AI players at each console's own ticks and
  slots), and no skater but the humans is a tester of the box tests (68).
- **The body is a puppet.** The owner sends each AI skater's look once per actor
  (`OwnedRecipe`, kept by the server for joiners) and its replay frames in runs of the cadence
  its own skater's take (`OwnedFrames`, relayed as `RemoteFrames` under the marked slot
  `owned::frames_slot`, acknowledged under `OwnedFrameAck`); the other consoles drive a
  `RemoteSkater` of it (`protocol::owned::skater_id`), shown while the round runs, hidden with
  its last facts frame, dropped with the round. Its frames carry the actor's audio physics
  record (M11.1: the wheels and landings, relative to its root), so the puppet sounds as a
  remote player does and the microphone follows it when the console does; no camera, no trick
  HUD, and not the AI's own call-out words (open item 66).
- **The natives.** A script's AI natives on a non-owner reach the manager (the row, the slot)
  and no actor: a teleport, reposition or request of a slot without a controller does nothing
  there (`try_teleport_ai_player` false, the controller map empty at the slot), and the owner's
  frames bring the outcome.

## The hand-off (2026-11-11, M11)

- **The owner is the lowest player still in the round.** `CampaignWorld.players` shrinks by a
  leaver at the tick every console delivers the leave (open item 64), so `ai_owner` moves to
  the next player on every console at once; `CampaignWorld.seats` keeps the slot boundary.
- **The AI stands through the hand-off.** The round keeps the leaver's last owned frames in
  the tick's frames until the next owner's facts carry the AI slots, 600 ticks at most
  (`Round.ai_handoff`); no console's scripts see a destruction.
- **The new owner spawns them afresh.** Every console kept the spawn commands it dropped as
  the old owner's (`SessionAi.streamed_spawns`); the new owner replays them at the standing
  frames' positions, drops its puppets of those slots, and from its next facts carries their
  frames and streams their bodies as the old owner did. The AI's routine starts over. Proof:
  `../bench/evidence/probe/2026-11-11-m11-handoff.md` (seat 1 quitting at round tick 2700 of
  the S.K.A.T.E.; seat 2 owned the pros from 2822 and played four more rounds).

## Not shared, recorded

- The AI's call-outs on the non-owner (its audio words do not travel; its physics sound does
  since M11.1, open item 66).
- The puppet's scoring body (`RemoteScoringSkater`) is collected as a remote player's; whether
  a trick over the pro scores alike on both consoles is play's to say.
- A prop the owner's AI knocks over is claimed for the owner as its own skater's touch is
  (read in the logs, M10.1) and the puppet follows it since M10.1; the puppet AI has no body
  to hit anything with (open item 67: what stays unclaimed is a car's or a pedestrian's push).

## History

- 2026-11-11 M9: the competing kinds settled through outcome notes while each console ran its
  own AI ([kinds](kinds.md)); the AI's run itself stayed each console's own.
- 2026-11-11 M10: one owner, the AI in the facts, puppets elsewhere (this page).
