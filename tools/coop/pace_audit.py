#!/usr/bin/env python3
"""A seat log's round pace (M22): the round's lead over its own skater, the timer, the notice.

usage: pace_audit.py SEAT.log [SEAT.log ...]

Lead: `produced − processed` from the round's "produced P, processed Q" line (every 300 produced
ticks); the first, the largest and the last, and the first line from which every later lead
stays under 45 (`kMaxSyncCommands`). Timer: the "pace at tick" changes, how many, the range and
the first and last. Notice: each "waiting for players" shown and taken down, with its tick and
the ticks waited.
"""
import re
import sys

PRODUCED = re.compile(r"online: produced (\d+), processed (\d+)")
PACE = re.compile(r"coop: pace at tick (\d+): timer (\d+) -> (\d+) Hz")
SHOWN = re.compile(r"coop: waiting for players at tick (\d+)")
DOWN = re.compile(r"coop: waiting for players over at tick (\d+) after (\d+) simulation ticks")
FAULT = re.compile(r"panicked|controlled .*failed|Tick\(")
WINDOW = 45


def audit(path):
    leads, changes, notices, faults = [], [], [], 0
    for line in open(path, errors="replace"):
        if m := PRODUCED.search(line):
            leads.append((int(m.group(1)), int(m.group(1)) - int(m.group(2))))
        elif m := PACE.search(line):
            changes.append((int(m.group(1)), int(m.group(2)), int(m.group(3))))
        elif m := SHOWN.search(line):
            notices.append(("shown", int(m.group(1)), None))
        elif m := DOWN.search(line):
            notices.append(("down", int(m.group(1)), int(m.group(2))))
        elif FAULT.search(line):
            faults += 1
    print(path)
    if leads:
        settled = next((produced for index, (produced, _) in enumerate(leads)
                        if all(lead < WINDOW for _, lead in leads[index:])), None)
        print(f"  lead: first {leads[0][1]}, largest {max(lead for _, lead in leads)}, last {leads[-1][1]} "
              f"(at produced {leads[-1][0]}); under {WINDOW} from produced {settled}")
    else:
        print("  lead: no round lines")
    if changes:
        rates = [hz for change in changes for hz in change[1:]]
        print(f"  timer: {len(changes)} changes, {min(rates)}..{max(rates)} Hz, "
              f"first {changes[0]}, last {changes[-1]}")
    else:
        print("  timer: no change")
    for kind, tick, waited in notices:
        print(f"  notice {kind} at tick {tick}" + (f" after {waited} ticks" if waited is not None else ""))
    print(f"  fault lines: {faults}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for log in sys.argv[1:]:
        audit(log)
