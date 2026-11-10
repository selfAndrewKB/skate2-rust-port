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

Every `start` restores each seat's `SKATER.P` from `SKATER.P.primed` (kept by `prime`) and
removes the run's `SKATER.P.*.bak`, so a run never continues from the autosaves of the last one
(a replayed opening tutorial once waited forever on a hull state the previous run had saved).

Seat windows open inactive and off-screen (`SKATE2_DIAGNOSTIC_BACKGROUND_WINDOW`), so the
desktop stays the person's; `BENCH_VISIBLE=1` shows them. Every seat traces online, career and
coop decisions. Priming needs the career demo (`--live-controls-demo career`) with
`SKATE2_DIAGNOSTIC_MOVIE_SKIP`, a profile name, the first mission completed by
`SKATE2_DIAGNOSTIC_COMPLETE_CHALLENGES` and a quit by `SKATE2_DIAGNOSTIC_CAREER_QUIT_END`; the
autosave writes `SKATER.P` at the save root.

**The seats' switches** (`SEAT_EXTRA_N`; ticks are the seat's own completed frames unless
said otherwise, round ticks are the campaign round's, the same on every console):
`SKATE2_DIAGNOSTIC_COOP_TELEPORT=<tick>:<x>,<y>,<z>[;...]` puts the skater there (786,-58,882 is
the waterfront spawn, a street with pedestrians; the career's continue point has none);
`SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=<round tick>:<x>,<y>,<z>[;...]` the same at round ticks;
`SKATE2_DIAGNOSTIC_COOP_SHOVE=<tick>` knocks down the nearest puppet pedestrian as a hit would
and sends the contact to its owner; `SKATE2_DIAGNOSTIC_COOP_EXECUTE=<round tick>:<name>` runs a
named challenge; `SKATE2_DIAGNOSTIC_COOP_COMPLETE=<round tick>[;...]` completes the main
challenge's active objective of every group as its script would; `SKATE2_DIAGNOSTIC_COOP_ANSWER=
yes|no[:<round ticks>]` answers the progression's Yes/No dialog; `SKATE2_DIAGNOSTIC_COOP_TRIGGERS=
<round tick>:<radius>` lists the trigger volumes around the skater and every objective's state,
boundary and condition (the volume ids match the conditions' floats through `skate3::id_float`),
with the sign-up boxes and every player's tester; `SKATE2_DIAGNOSTIC_COOP_PROMPT=<round tick>:
<answer>[;...]` presses as the pad would (`yes`, `no`, `option:<n>`, `skate101`, `unlock`,
`signup` for the D-pad press, `release`, `choice:<n>`, `cancel`, `accept`, `decline`,
`service:<n>`, `service_cancel`), so the press goes through the same funnel as a player's and
the answer rides the facts as a note; `SKATE2_DIAGNOSTIC_COOP_NUDGE=<tick>` gives the nearest
prop within 30 m a velocity and counts it as touched by this seat's skater (its claim and
stream follow); `SKATE2_DIAGNOSTIC_COOP_ZONES=<round tick>` scans the security guard areas
within 1500 m of the skater and traces the nearest point of each (the bench's way into a
chase).
The career's own `SKATE2_DIAGNOSTIC_CAREER_*` switches run only under the career demo, not on a
seat. `COOP_EXECUTE` and `COOP_COMPLETE` act on the seat that carries them only (a mission a
diagnostic starts is not a note, unlike a sign-up's accept), so a mission run belongs in
`SEAT_EXTRA`, every seat's: with it on one seat alone the mission ran there alone while the
other seat idled in the same round (2026-11-10, a run not kept).

**A typical M2 run** (the opening tutorial, the three go-to exits by each player's own entry,
the rest at the same tick):

```
R="1516.5,110.6,-719.6"; M="1569.5,109.3,-693.9"; G="1598.6,107.1,-662.0"
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA="SKATE2_DIAGNOSTIC_COOP_ANSWER=no SKATE2_DIAGNOSTIC_COOP_EXECUTE=900:m_1a_01_goto_slappys SKATE2_DIAGNOSTIC_COOP_COMPLETE=2400;2500;2600;2700;2800;2900;3000;3100" \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1200:$R;1500:$M;1800:$G" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1300:$R;1600:$M;1900:$G" \
CAPTURE_COUNT=0 ./tools/coop/bench.sh start 2
```

**A typical M3 run** (M2's opening tutorial, then both at the flip-tricks sign-up box, seat 2
signing up with a press, a release and a press after the mission's end phase, seat 1 accepting
the poster; the sign-up is withheld until the main challenge ends, around round tick 3500 to
5300, so the presses are late):

```
R="1516.5,110.6,-719.6"; M="1569.5,109.3,-693.9"; G="1598.6,107.1,-662.0"; B="1640.4,100.7,-522.4"
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA="SKATE2_DIAGNOSTIC_COOP_ANSWER=no SKATE2_DIAGNOSTIC_COOP_EXECUTE=900:m_1a_01_goto_slappys SKATE2_DIAGNOSTIC_COOP_COMPLETE=2400;2500;2600;2700;2800;2900;3000;3100" \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1200:$R;1500:$M;1800:$G;3300:$B SKATE2_DIAGNOSTIC_COOP_PROMPT=6300:accept" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1300:$R;1600:$M;1900:$G;3400:$B SKATE2_DIAGNOSTIC_COOP_PROMPT=5500:signup;5520:release;5560:signup" \
CAPTURE_COUNT=0 ./tools/coop/bench.sh start 2
```

**A typical M4 run** (M3's, with the sign-up teleports after the mission's end phase, which
now waits for its outro scene and ends around round tick 4240; the seats' logs then show every
scene and speech end `seen here`, `announced with the facts of tick N` and `delivered to the
scripts at tick N+1` on both seats, and each objective's presentation `plays` once):

```
R="1516.5,110.6,-719.6"; M="1569.5,109.3,-693.9"; G="1598.6,107.1,-662.0"; B="1640.4,100.7,-522.4"
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA="SKATE2_DIAGNOSTIC_COOP_ANSWER=no SKATE2_DIAGNOSTIC_COOP_EXECUTE=900:m_1a_01_goto_slappys SKATE2_DIAGNOSTIC_COOP_COMPLETE=2400;2500;2600;2700;2800;2900;3000;3100" \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1200:$R;1500:$M;1800:$G;4600:$B;5300:$B SKATE2_DIAGNOSTIC_COOP_PROMPT=6500:accept" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1300:$R;1600:$M;1900:$G;4700:$B;5400:$B SKATE2_DIAGNOSTIC_COOP_PROMPT=5700:signup;5720:release;5760:signup" \
CAPTURE_COUNT=0 ./tools/coop/bench.sh start 2
```

**A typical M5 run** (M4's, with a prop nudged on seat 1 while both stand at the tutorial's
third go-to, the zone scan, and a service bought by seat 2 after the flip-tricks sign-up; the
seats' logs show the prop's claim on seat 1, its puppet on seat 2, and the service decision at
the same tick on both):

```
R="1516.5,110.6,-719.6"; M="1569.5,109.3,-693.9"; G="1598.6,107.1,-662.0"; B="1640.4,100.7,-522.4"
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA="SKATE2_DIAGNOSTIC_COOP_ANSWER=no SKATE2_DIAGNOSTIC_COOP_EXECUTE=900:m_1a_01_goto_slappys SKATE2_DIAGNOSTIC_COOP_COMPLETE=2400;2500;2600;2700;2800;2900;3000;3100" \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1200:$R;1500:$M;1800:$G;4600:$B;5300:$B SKATE2_DIAGNOSTIC_COOP_PROMPT=6500:accept SKATE2_DIAGNOSTIC_COOP_NUDGE=2700 SKATE2_DIAGNOSTIC_COOP_ZONES=1000" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1300:$R;1600:$M;1900:$G;4700:$B;5400:$B SKATE2_DIAGNOSTIC_COOP_PROMPT=5700:signup;5720:release;5760:signup;7200:service:0" \
CAPTURE_COUNT=0 ./tools/coop/bench.sh start 2
```

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
