# Tooling: the bench, the switches, the machines

**Scope:** how the co-op fork is built, run and measured on one machine and on two. Front page:
[README](../README.md). Per-machine notes: [macOS](setup-macos.md), [Windows](setup-windows.md).

## The bench

One dedicated server and N game instances ("seats") on one machine, each seat with its own
scratch save root, nickname, environment and log, driven without a human through the port's
diagnostic switches. The script is `tools/coop/bench.sh`; its header lists every knob.

```
./tools/coop/bench.sh prime 2      # once: a scratch profile per seat with the first mission done
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 ./tools/coop/bench.sh start 2
./tools/coop/bench.sh status       # pids and the last line of each log
./tools/coop/bench.sh stop
```

The bench folder (`../bench` beside the repository, never a save root a person plays from) holds
`server/server.log`, `seat-N/seat.log`, `seat-N/seat.env` (the seat's whole environment, so a
switch that did not take is never a guess), `seat-N/prime.log`, `seat-N/SKATER.P` and the seats'
captures `seat-N/capture_<nnn>.png` (BMP data under a .png name, as the port's `--live-capture`
writes them; `sips -s format png` converts one on macOS). What proves a milestone is copied to
`../bench/evidence/<date>-<milestone>-<what>` and named by path from the docs; nothing of it
enters the repository.

Seat windows open inactive and off-screen (`SKATE2_DIAGNOSTIC_BACKGROUND_WINDOW`), so the
desktop stays the person's; `BENCH_VISIBLE=1` shows them. Every seat traces online, career and
coop decisions. Priming needs the career demo (`--live-controls-demo career`) with
`SKATE2_DIAGNOSTIC_MOVIE_SKIP`, a profile name, the first mission completed by
`SKATE2_DIAGNOSTIC_COMPLETE_CHALLENGES` and a quit by `SKATE2_DIAGNOSTIC_CAREER_QUIT_END`; the
autosave writes `SKATER.P` at the save root.

**The seats' switches** (`SEAT_EXTRA_N`): `SKATE2_DIAGNOSTIC_COOP_TELEPORT=<tick>:<x>,<y>,<z>`
puts the skater there (786,-58,882 is the waterfront spawn, a street with pedestrians; the
career's continue point has none); `SKATE2_DIAGNOSTIC_COOP_SHOVE=<tick>` knocks down the nearest
puppet pedestrian as a hit would and sends the contact to its owner. The career's own
`SKATE2_DIAGNOSTIC_CAREER_*` switches run only under the career demo, not on a seat.

**A typical M1 run:**

```
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT=3000:786,-58,882" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT=3000:792,-58,888 SKATE2_DIAGNOSTIC_COOP_SHOVE=4800" \
CAPTURE_FROM=100 CAPTURE_EVERY=20 CAPTURE_COUNT=6 ./tools/coop/bench.sh start 2
```

## Building

`./build-release-macos.sh` (the fork removes the old binary before copying and ad-hoc signs it:
macOS kills a binary overwritten in place). A dev build only talks to a server run from the same
executable (`skate2-pc --server`); two machines need the same commit and the same
`release-secret.txt` (`docs/release/auto-update.md`). Tests: `cargo test -p skate2-net`,
`cargo test -p skate2-core coop`.

## What the bench taught about the port (fixed on this branch)

- The client starts the native front end only without `--live-completions`, and `--live-capture`
  demanded one, so a capturing seat never reached the main menu where the online diagnostic signs
  in. Periodic captures now need no bound; the diagnostic traces why it waits.
- macOS withholds the swapchain of an off-screen window ("Occluded"); a capture then renders to
  an offscreen texture and presents nothing. Some captures catch a frame mid-reload of a hull's
  materials (lower half black); the next is whole.
- The audio "buffer underrun" lines on every seat are two instances sharing one output device.
