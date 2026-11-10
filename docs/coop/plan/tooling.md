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
./tools/coop/bench.sh server       # the server alone (M8: it reads campaign-<key>.save back)
SEAT_ONLINE_3=join:9 ./tools/coop/bench.sh seat 3   # one seat alone, joining the running server
./tools/coop/bench.sh stop
```

`BENCH_KEEP=1` keeps a seat's last save instead of restoring the primed profile (a crawl, a
resume). A seat's `player_id` (its stable identity, M8) survives the `online-config.json` the
bench rewrites.

`start` returns at once: every process it starts is detached from the caller's stdio, so a
script or a tool that ran it never waits on the games (they lingered as background tasks until
the games were closed by hand, 2026-11-10); `stop` reaches them by pid.

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
desktop stays the person's; `BENCH_VISIBLE=1` shows them tiled side by side
(`SKATE2_DIAGNOSTIC_WINDOW_POSITION`, seat N at x = (N-1) × width; `WINDOW_SIZE=720x405` fits two
on a 1470-wide display), and a pad drives the focused window's skater: the way to play. Every seat traces online, career and
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
and sends the contact to its owner; `SKATE2_DIAGNOSTIC_COOP_EXECUTE=<round tick>:<name>[;...]` runs
named challenges; `SKATE2_DIAGNOSTIC_COOP_WIN=<round tick>[;...]` completes every objective of
the main challenge as a win would (points objectives included), so the real win, outro and
`challenge_cleanup` play: the crawl's way through a mission whose objectives need tricks; `SKATE2_DIAGNOSTIC_COOP_COMPLETE=<round tick>[@<player>][;...]` completes the main
challenge's active objective of every group (or of that player's group alone, M7) as its script
would; `SKATE2_DIAGNOSTIC_COOP_ANSWER=
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
seat. The completion rules: `coop-rules.json` beside the saves (a seat's folder on the bench) or
`SKATE2_COOP_RULES=<file>`, e.g. `{"prefixes": {"m_1a_": "Anyone"}}` to let the first seat's
completion end the opening tutorial for both ([kinds](topics/kinds.md)); every seat reads its
own file, so every seat must read the same one or the round splits (the hash traced at the
round start on each console says; sending the creator's table is owed,
[open items](open-items.md) 37). `COOP_EXECUTE` and `COOP_COMPLETE` act on the seat that carries them only (a mission a
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

**A typical M7 run** (the opening tutorial under a rules file that makes it `Anyone`'s, player
0's copies alone completed on both consoles; the logs show "1 of 2 players complete under
Anyone: the activity completes" at the same tick on both):

```
printf '{"prefixes": {"m_1a_": "Anyone"}}' > /tmp/coop-rules-anyone.json
R="1516.5,110.6,-719.6"; M="1569.5,109.3,-693.9"; G="1598.6,107.1,-662.0"
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA="SKATE2_DIAGNOSTIC_COOP_ANSWER=no SKATE2_DIAGNOSTIC_COOP_EXECUTE=900:m_1a_01_goto_slappys SKATE2_COOP_RULES=/tmp/coop-rules-anyone.json SKATE2_DIAGNOSTIC_COOP_COMPLETE=2400@0;2500@0;2600@0;2700@0;2800@0;2900@0;3000@0;3100@0" \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1200:$R;1500:$M;1800:$G" \
CAPTURE_COUNT=0 ./tools/coop/bench.sh start 2
```

**The guard run** (no mission; both seats to the waterfront, then into the no-skate zone the
scan found, `NoSkateZone_Area` at x 591..663, z 640..748; no guard spawned in five minutes,
[open items](open-items.md) 30):

```
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 SEAT_EXTRA="SKATE2_DIAGNOSTIC_COOP_ANSWER=no" \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1000:786,-58,882;1500:662.6,-58,700 SKATE2_DIAGNOSTIC_COOP_ZONES=1800" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=1000:792,-58,888;1600:640,-58,700" \
CAPTURE_COUNT=0 ./tools/coop/bench.sh start 2
```

**A typical M6 run** (shown, so a person can take the pads afterwards: both seats at the
waterfront, seat 2 skating 70 m east and back; the logs show the handoffs on seat 2 and the
promotions on seat 1):

```
BENCH_VISIBLE=1 WINDOW_SIZE=720x405 SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA="SKATE2_DIAGNOSTIC_COOP_ANSWER=no" \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=600:786,-58,882" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=600:792,-58,888;2400:856,-58,888;4200:792,-58,888" \
CAPTURE_COUNT=0 ./tools/coop/bench.sh start 2
```

**A typical M1 run:**

```
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 \
SEAT_EXTRA_1="SKATE2_DIAGNOSTIC_COOP_TELEPORT=3000:786,-58,882" \
SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_COOP_TELEPORT=3000:792,-58,888 SKATE2_DIAGNOSTIC_COOP_SHOVE=4800" \
CAPTURE_FROM=100 CAPTURE_EVERY=20 CAPTURE_COUNT=6 ./tools/coop/bench.sh start 2
```

## The mission crawl

`python3 tools/coop/crawl.py --list` names the campaign's 115 `m_*` missions from the career
database (`SKATE2_CAREER_DB`); `--run [--from NAME] [--count N]` runs them on the bench one
start at a time with the saves carried forward (`BENCH_KEEP`), each executed by name at round
tick 900 and won by diagnostic at 1800, and appends one row per mission to
`../bench/evidence/<date>-crawl.md`: PASS (both seats reached `challenge_cleanup` and the saves
agree), PASS (no save) (a tutorial that autosaves nothing), PASS (saves differ), STOPPED (with
each seat's last state), NOT EXECUTED, CRASHED. Each mission's logs are
kept as `crawl-<name>-seat-N.log`. About three minutes a mission.

## The synchronization audit

What is shared and what is not, as data, not by eye: `tools/coop/sync_inventory.py` generates
[sync-inventory.md](sync-inventory.md) from the structs (a field it does not know fails the
check, so a new field cannot go unclassified); `SKATE2_TRACE_COOP_CENSUS=1` on every seat
writes, every 60 round ticks, one line per pedestrian the seat knows (world id, owner, epoch,
position, yaw, speed, flags, model, graph behaviour, moods, down); and
`tools/coop/sync_audit.py seat-1.log seat-2.log` pairs the seats' lines by world id and round
tick and reports, per field, how many pairs agree within tolerance (position 0.5 m, yaw 5°,
speed 0.5 m/s, flags, model and down exact), the worst gap and where, the divergences still
open at the last dump (`--all` lists them), and for the fields the inventory calls local (the
graph behaviour, the moods) how often they differ, by design. Run it on every recipe and on
play. Live, in every build and on any two machines: the receiving end measures every sample
against the puppet it drives before applying it and writes `live audit at <tick>: N samples;
position within 0.5 m …, yaw within 5° …, speed within 0.5 m/s …, flags equal …` every ten
seconds beside the stream's line, and `live audit: puppet X … from its owner's sample for 20
samples` once per excursion (`SKATE2_TRACE_COOP=1`).

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
