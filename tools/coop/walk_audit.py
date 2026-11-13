#!/usr/bin/env python3
"""The walk-in-place and pause figures of a two-seat soak (M20), from its two logs.

usage: walk_audit.py SEAT1.log SEAT2.log

Walking in place: of the census intervals (60 round ticks) over which an owner's pedestrian
stood (moved under 0.15 m), how many found its puppet, at the interval's end, moving faster than
0.3 m/s, and (where the census says what each copy's motion graph is given, M20) how many found
the puppet given another major intent than its owner. Stuck collisions: each seat's longest run
of census seconds a puppet stayed colliding (flags bit 2), and the owners' knocked-down seconds
whose puppet was given no collision intent. Pauses: each seat's puppet jerk windows ("puppet jerk at", M19), summed over the run.
"""
import re
import sys

CENSUS = re.compile(
    r"census (\d+): ([0-9a-f]{8}) (own|puppet of \d+) e\d+ (\S+) (\S+) (\S+) yaw \S+ v (\S+) f(\d+) ")
JERK = re.compile(r"puppet jerk at \d+: (\d+) ticks, mean \S+ mm, (\d+) past 5 mm")
PLAYS = re.compile(r"census (\d+): ([0-9a-f]{8}) plays (\S*)$")
OWN_JERK = re.compile(r"own pedestrians' jerk at \d+: (\d+) ticks, mean \S+ mm, (\d+) past 5 mm")


def read(path):
    own, puppet = {}, {}
    jerk = [0, 0]
    own_jerk = [0, 0]
    plays = {}
    for line in open(path, errors="replace"):
        m = PLAYS.search(line.rstrip())
        if m:
            plays[(int(m.group(1)), m.group(2))] = m.group(3).split("/")[0]
            continue
        m = CENSUS.search(line)
        if m:
            tick, wid, owner, x, y, z, v, f = m.groups()
            row = (float(x), float(y), float(z), float(v), int(f))
            (own if owner == "own" else puppet)[(int(tick), wid)] = row
            continue
        # Each window counts the ticks since the last: summed over the run.
        m = JERK.search(line)
        if m:
            jerk = [jerk[0] + int(m.group(1)), jerk[1] + int(m.group(2))]
            continue
        m = OWN_JERK.search(line)
        if m:
            own_jerk = [own_jerk[0] + int(m.group(1)), own_jerk[1] + int(m.group(2))]
    return own, puppet, jerk, own_jerk, plays


def stood_seconds(own, puppet, own_plays, puppet_plays):
    stood = moving = compared = other_major = 0
    for (tick, wid), (x, y, z, _, _) in own.items():
        before = own.get((tick - 60, wid))
        if before is None or (tick, wid) not in puppet:
            continue
        if ((x - before[0]) ** 2 + (y - before[1]) ** 2 + (z - before[2]) ** 2) ** 0.5 >= 0.15:
            continue
        stood += 1
        moving += puppet[(tick, wid)][3] > 0.3
        if (tick, wid) in own_plays and (tick, wid) in puppet_plays:
            compared += 1
            other_major += own_plays[(tick, wid)] != puppet_plays[(tick, wid)]
    return stood, moving, compared, other_major


def stuck(puppet):
    """The longest run of census seconds each puppet stayed colliding, the worst five."""
    longest, run = {}, {}
    for (tick, wid), row in sorted(puppet.items()):
        colliding = row[4] & 2 != 0
        run[wid] = run.get(wid, 0) + 1 if colliding and (tick - 60, wid) in puppet else int(colliding)
        longest[wid] = max(longest.get(wid, 0), run[wid])
    return sorted(((n, wid) for wid, n in longest.items() if n), reverse=True)[:5]


def unplayed_knockdowns(own, puppet_plays):
    down = unplayed = 0
    for key, row in own.items():
        if row[4] & 1 and key in puppet_plays:
            down += 1
            unplayed += puppet_plays[key] != "collision"
    return down, unplayed


def main(a, b):
    seats = [read(a), read(b)]
    for index, (one, other) in enumerate(((0, 1), (1, 0)), start=1):
        stood, moving, compared, other_major = stood_seconds(seats[one][0], seats[other][1], seats[one][4], seats[other][4])
        print(f"seat {index}'s pedestrians: {stood} stood seconds, their puppets moving in {moving} "
              f"({100 * moving / max(stood, 1):.1f} %)" + (
                  f", given another major intent in {other_major} of {compared} ({100 * other_major / compared:.1f} %)" if compared else ""))
    for index, (one, other) in enumerate(((0, 1), (1, 0)), start=1):
        down, unplayed = unplayed_knockdowns(seats[one][0], seats[other][4])
        print(f"seat {index}'s pedestrians knocked down: {down} census seconds, their puppets given no collision intent in {unplayed}")
    for index, seat in enumerate(seats, start=1):
        print(f"seat {index}'s puppets' longest colliding runs (census seconds, id): {stuck(seat[1])}")
    for index, (_, _, jerk, own_jerk, _) in enumerate(seats, start=1):
        print(f"seat {index}: puppet jerk past 5 mm {jerk[1]} of {jerk[0]} ticks ({100 * jerk[1] / max(jerk[0], 1):.2f} %), "
              f"own pedestrians' {own_jerk[1]} of {own_jerk[0]} ({100 * own_jerk[1] / max(own_jerk[0], 1):.2f} %)")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
