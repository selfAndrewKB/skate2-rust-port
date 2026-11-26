#!/usr/bin/env python3
"""A seat log's round pace (M22): the round's lead over its own skater, the timer, the notice.

usage: pace_audit.py SEAT.log [SEAT.log ...]

Lead: `produced − processed` from the round's "produced P, processed Q" line (every 300 produced
ticks); the first, the largest and the last, and the first line from which every later lead
stays under 45 (`kMaxSyncCommands`). Timer: the "pace at tick" changes, how many, the range and
the first and last. Notice: each "waiting for players" shown and taken down, with its tick and
the ticks waited.
"""
import sys

import events

WINDOW = 45


def audit(path):
    leads, changes, notices = [], [], []
    for record in events.read(path, {"round_lead", "pace", "notice_shown", "notice_down"}):
        kind = record["e"]
        if kind == "round_lead":
            leads.append((record["produced"], record["produced"] - record["processed"]))
        elif kind == "pace":
            changes.append((record["tick"], record["from_hz"], record["to_hz"]))
        elif kind == "notice_shown":
            notices.append(("shown", record["tick"], None))
        else:
            notices.append(("down", record["tick"], record["waited"]))
    faults = len(events.faults(path))
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
