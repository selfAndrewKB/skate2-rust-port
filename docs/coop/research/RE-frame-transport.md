# RE: the port's replay-frame transport (read 2026-11-10)

How a remote skater's frames travel and play out, as the world stream had to fit beside them.
Paths are repository-relative. Read for M1; every claim is from the code at commit `a7859ad`.

## Messages (`crates/skate2-net/src/protocol.rs`)

- Frames travel as `ClientMessage::Frames(EncodedRun)` (tag 13) and come back as
  `ServerMessage::RemoteFrames { slot, run }` (tag 18); `FrameAcks(Vec<(slot, latest, mask)>)`
  (tag 14) and `FrameAck { latest, mask }` (tag 19) acknowledge; `Busy` (tag 17) keeps a member
  alive with no frame to send. Client tags ran 0..=24 (`Stalled` 24), server tags 0..=26, before
  the fork's `World` (25) and `RemoteWorld` (27).
- `EncodedRun { tick, count, frame, frame_before, sample, sample_before, payload }`
  (`frames.rs:198`): the first tick, the count (1..=8), four reference distances back from the
  first tick (0 for none), then the range-coded payload to the end of the message.

## The relay (`server.rs`)

- `handle` (582): `Frames` marks the member alive, takes a token from `FRAME_RATE` (90/s, bursts
  of 60) and calls `relay_frames` (1492). The server does decode the run: it needs the sender's
  recipe first, decodes with the member's `FrameReceiver`, records the member's own ack, and
  re-encodes for every other member against what that receiver holds of this sender
  (`member.holds[user_id]: AckState`, `encode_for`), sent unreliably; a deferred `FrameAck` goes
  back to the sender. A decode error is logged once per member.
- Reliable relays (`Facts`, `Vote`) go through `game_message` and `relay_to_participants`
  without decoding. The fork's `World` is relayed like them but unreliably, with its own bucket.

## The client (`client.rs`, `frames.rs`)

- `ClientCore` holds `own_frames: FrameSender`, `remote_frames: HashMap<u64, FrameReceiver>`,
  `slots`. `send_frames` codes against the acknowledged baseline; a run with no baseline may not
  fit a datagram and goes reliably. `Command::Tick(messages, frames)`: reliable messages first,
  then the run, one datagram where they fit (`Client::send_tick`).
- Coding: `PLAYING_INTERVAL` 2, `INTERVAL` 3, `FRAME_SPAN` 2, `SAMPLE_SPAN` 8, `MAX_RUN` 8,
  `HISTORY` 64; `AckState::baselines` picks the newest acknowledged frame, one at least
  `FRAME_SPAN` older, the newest sample and one `SAMPLE_SPAN` older; the range coder starts
  afresh each run.
- The session builds a run in `online_tick_frame` (`selected_session_online.rs:~1401`): the local
  frame from `runtime.submitted_frame`, the recipe once (2.5 s spacing), `interval` by whether
  the local skater plays a round, a sample every `interval` ticks or at a teleport, the run sent
  at a sample or at 8 frames.

## The receive side (`session/online/remote_skater.rs`)

- `ClientEvent::Frame { user_id, frame, run_end }` → `RemoteSkater::receive(tick, frame, paced,
  run_end)`: paced arrivals feed `Playout` (a weighted least-squares arrival line, target =
  95th-percentile lateness + 3 ticks + the run spacing, pace from the slope, never past the
  newest frame); frames kept in a `BTreeMap` (600 ahead, 120 behind).
- `advance` blends root (slerp/lerp), pose and camera around the shown moment, never across a
  teleport, and restores a `SkaterFrame` through `SkaterRestore::restore` (presentation only, no
  body). Submitted each tick through `online_submitted_frames`, `online_visuals`,
  `online_audio_skaters` (audio id 8 + slot). The trace every 600 ticks: "N of M ticks at or
  past the newest sample, delay, pace, teleports".

## Sizes and limits

- A frame's header is 208 bytes (52 words); words 0-31 never travel; every tick carries the
  quantized root (rotation 1/32768, position 1/1024), board speed, flags word 48, audio and HUD
  words; a sample adds the pose (one `[f32;12]` per bone, 1/1024), twists, audio floats, camera.
- `MAX_DATAGRAM` 1200, `MAX_FRAGMENT` 1000, `MAX_UNRELIABLE` 1164 (`transport.rs`); `MAX_RECIPE`
  128 KiB, `MAX_FACTS` 16 KiB. No general compression. Measured live: about 5.9 KB/s and 21
  datagrams a second per player in lobby skate.

## Adding a message

No procedure section exists in `docs/online/implementation.md`; each wire change is recorded
with its protocol number. `PROTOCOL_VERSION` (`transport.rs:17`) is the fork's own (116), checked
at `server.rs:427`. Tags are hand-written in `encode`/`decode`; the round-trip tests list one
instance of every variant; relay tests use the simulated `Net` in `tests.rs`.

## Players

`OnlineGame { local_slot, players: Vec<OnlinePlayer { user_id, slot, name }>, participants,
remotes: BTreeMap<u64, RemoteSkater> }`; the first player is the host peer. No user-id-to-slot
helper in the session: code reads `remote.slot` or searches `players`.
