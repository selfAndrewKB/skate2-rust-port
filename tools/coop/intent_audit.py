#!/usr/bin/env python3
"""Pairs each owned pedestrian's census line with its puppet's at the same round tick (the two
seats' logs) and counts where the motion graph's intents differ (M20).

usage: intent_audit.py SEAT1.log SEAT2.log

A census line (SKATE2_TRACE_COOP_CENSUS=1) ends `i q<queued> <intents joined by />`, the
active stage of its intent generator; lines without it come from a build before M20 and are
skipped. A `census <tick>: <id> plays <names>` line (M20's fix on) gives what its motion graph is
given, compared too where both copies have one.
"""
import collections
import re
import sys

LINE = re.compile(
    r"census (\d+): ([0-9a-f]{8}) (own|puppet of \d+) e\d+ \S+ \S+ \S+ yaw \S+ v (\S+) .* (up|down) i q(\d+) (\S*)$")


PLAYS = re.compile(r"census (\d+): ([0-9a-f]{8}) plays (\S*)$")


def read(path):
    rows, plays = {}, {}
    for line in open(path, errors="replace"):
        line = line.rstrip()
        m = LINE.search(line)
        if m:
            tick, wid, owner, v, _, queued, intents = m.groups()
            rows[(int(tick), wid)] = (owner == "own", float(v), int(queued), intents)
            continue
        m = PLAYS.search(line)
        if m:
            plays[(int(m.group(1)), m.group(2))] = m.group(3)
    return rows, plays


def main(a, b):
    read_back = [read(a), read(b)]
    seats = [rows for rows, _ in read_back]
    plays = [played for _, played in read_back]
    pairs = same = 0
    played_pairs = played_same = major_same = 0
    differ = collections.Counter()
    played_differ = collections.Counter()
    for one, other in ((0, 1), (1, 0)):
        for key, (own, v, queued, intents) in seats[one].items():
            if not own or key not in seats[other]:
                continue
            p_own, p_v, p_queued, p_intents = seats[other][key]
            if p_own:
                continue
            pairs += 1
            if intents == p_intents:
                same += 1
            else:
                differ[(intents, p_intents)] += 1
            if key in plays[one] and key in plays[other]:
                played_pairs += 1
                own_played, puppet_played = plays[one][key], plays[other][key]
                played_same += own_played == puppet_played
                major_same += own_played.split("/")[0] == puppet_played.split("/")[0]
                if own_played != puppet_played:
                    played_differ[(own_played, puppet_played)] += 1
    print(f"{pairs} owner/puppet pairs, {same} with the same intents ({100 * same / max(pairs, 1):.1f}%)")
    for (owner, puppet), count in differ.most_common(20):
        print(f"{count:6}  owner {owner:40} puppet {puppet}")
    if played_pairs:
        print(f"motion graph given: {played_pairs} pairs, {played_same} the same ({100 * played_same / played_pairs:.1f}%), "
              f"the same major intent {major_same} ({100 * major_same / played_pairs:.1f}%)")
        for (owner, puppet), count in played_differ.most_common(15):
            print(f"{count:6}  owner plays {owner:34} puppet plays {puppet}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
