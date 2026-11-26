#!/usr/bin/env python3
"""Two seats' round sync check (M23): where their games parted, from the seat logs.

usage: sync_check_audit.py SEAT-A.log SEAT-B.log

Per seat: the checks each other player's facts carried and how many differed (the last
"sync check with" line), every difference reported ("sync check: player P differs at tick T in
PARTS") and the fault lines. Between the seats: the first tick whose whole parts differ, from
the "sync parts @T" windows both printed (a difference prints the ticks around it; the window's
first differing tick is where the state parted, to the tick, and its parts say what), and every
field-dump tick both seats printed ("sync dump @T part: line", from `SKATE2_COOP_SYNC_DUMP` or a
difference) with the lines only one seat has: the fields that differ.
"""
import sys
from collections import defaultdict

import events

NAMES = ["inputs", "activities", "objectives", "timers", "vars"]
KINDS = {"sync_standing", "sync_differs", "sync_unknown", "sync_parts", "sync_dump", "desync_planted"}


def read(path):
    seat = {"standing": {}, "differs": [], "unknown": 0, "parts": {}, "dumps": defaultdict(lambda: defaultdict(set)), "planted": None, "faults": len(events.faults(path))}
    for record in events.read(path, KINDS):
        kind = record["e"]
        if kind == "sync_standing":
            first = "none" if record["first_tick"] is None else f"tick {record['first_tick']} in {'+'.join(record['first_parts'])}"
            seat["standing"][record["user"]] = (record["checked"], record["differed"], first)
        elif kind == "sync_differs":
            seat["differs"].append((record["tick"], record["player"], "+".join(record["parts"]), "the first difference" if record["first"] else "after agreeing again"))
        elif kind == "sync_unknown":
            seat["unknown"] += 1
        elif kind == "sync_parts":
            seat["parts"][record["tick"]] = record["parts"]
        elif kind == "sync_dump":
            seat["dumps"][record["tick"]][record["part"]].add(record["line"])
        else:
            seat["planted"] = (record["activity"], record["tick"])
    return seat


def report(path, seat):
    print(path)
    for user, (checked, differed, first) in seat["standing"].items():
        print(f"  with {user}: {checked} checks, {differed} differed, first difference {first}")
    if not seat["standing"]:
        print("  no sync check standing (not a campaign round, or a build before M23)")
    for tick, player, parts, kind in seat["differs"]:
        print(f"  differs from player {player} at tick {tick} in {parts} ({kind})")
    if seat["unknown"]:
        print(f"  checks naming a tick this seat holds no record of: {seat['unknown']}")
    if seat["planted"]:
        print(f"  planted divergence: {seat['planted'][0]}'s vars at tick {seat['planted'][1]}")
    print(f"  fault lines: {seat['faults']}")


def compare(a, b):
    common = sorted(set(a["parts"]) & set(b["parts"]))
    parted = next((tick for tick in common if a["parts"][tick] != b["parts"][tick]), None)
    if parted is None:
        print(f"between the seats: {len(common)} ticks in both windows, none differ" if common else "between the seats: no window both printed")
    else:
        names = [NAMES[i] for i, (x, y) in enumerate(zip(a["parts"][parted], b["parts"][parted])) if x != y]
        agreed = max((tick for tick in common if tick < parted), default=None)
        print(f"between the seats: first tick whose parts differ {parted} in {'+'.join(names)} (last agreeing tick in the windows: {agreed})")
    for tick in sorted(set(a["dumps"]) & set(b["dumps"])):
        for part in sorted(set(a["dumps"][tick]) | set(b["dumps"][tick])):
            only_a = sorted(a["dumps"][tick][part] - b["dumps"][tick][part])
            only_b = sorted(b["dumps"][tick][part] - a["dumps"][tick][part])
            if not only_a and not only_b:
                print(f"  dump @{tick} {part}: alike ({len(a['dumps'][tick][part])} lines)")
                continue
            print(f"  dump @{tick} {part}: {len(only_a)} lines only in the first seat, {len(only_b)} only in the second")
            for line in only_a[:20]:
                print(f"    first:  {line}")
            for line in only_b[:20]:
                print(f"    second: {line}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    seats = [read(path) for path in sys.argv[1:]]
    for path, seat in zip(sys.argv[1:], seats):
        report(path, seat)
    compare(*seats)
