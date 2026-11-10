#!/usr/bin/env python3
"""The synchronization audit: the two seats' census dumps (`SKATE2_TRACE_COOP_CENSUS=1`, one
line per pedestrian every 60 round ticks) paired by world id and round tick, each field
compared, and a verdict per field and per entity: agreeing within tolerance, diverging (how far,
since when), or present on one seat only.

    python3 tools/coop/sync_audit.py ../bench/seat-1/seat.log ../bench/seat-2/seat.log [--all]

A census line:
    coop census <round tick>: <wid> <own|puppet of N> e<epoch> <x> <y> <z> yaw <deg> v <m/s> f<flags> m<model> g<graph> moods<n> <down|up>
The inventory (`docs/coop/plan/sync-inventory.md`) says which fields are meant to agree:
position, yaw, speed, flags, model and down are streamed or derived; the graph behaviour and the
moods are local by design and reported as "not shared" rather than as divergence."""
import re
import sys
from collections import defaultdict

LINE = re.compile(r'coop census (\d+): ([0-9a-f]{8}) (own|puppet of \d+) e(\d+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) yaw (-?[\d.]+) v ([\d.]+) f(\d+) m([0-9a-f]+) g(\d+) moods(\d+) (down|up)')
SHARED = {'position': 0.5, 'yaw': 5.0, 'speed': 0.5, 'flags': 0, 'model': 0, 'down': 0}
LOCAL = ['graph', 'moods']


def read(path):
    out = {}
    for line in open(path, encoding='utf-8', errors='replace'):
        m = LINE.search(line)
        if not m:
            continue
        tick, wid, owner, epoch, x, y, z, yaw, speed, flags, model, graph, moods, down = m.groups()
        out[(int(tick), wid)] = {
            'owner': owner, 'epoch': int(epoch), 'position': (float(x), float(y), float(z)), 'yaw': float(yaw),
            'speed': float(speed), 'flags': int(flags), 'model': model, 'graph': int(graph), 'moods': int(moods), 'down': down,
        }
    return out


def gap(field, a, b):
    if field == 'position':
        return sum((p - q) ** 2 for p, q in zip(a, b)) ** 0.5
    if field == 'yaw':
        d = abs(a - b) % 360.0
        return min(d, 360.0 - d)
    if field in ('speed',):
        return abs(a - b)
    return 0.0 if a == b else 1.0


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    if len(args) != 2:
        print(__doc__)
        sys.exit(2)
    a, b = read(args[0]), read(args[1])
    ticks = sorted({t for t, _ in a} | {t for t, _ in b})
    if not ticks:
        print('no census lines: run the seats with SKATE2_TRACE_COOP_CENSUS=1')
        sys.exit(1)
    paired = both = only_a = only_b = 0
    worst = {f: (0.0, None) for f in SHARED}
    bad = {f: 0 for f in SHARED}
    local_differs = {f: 0 for f in LOCAL}
    since = {}
    owner_disagree = 0
    for t in ticks:
        wids_a = {w for tt, w in a if tt == t}
        wids_b = {w for tt, w in b if tt == t}
        only_a += len(wids_a - wids_b)
        only_b += len(wids_b - wids_a)
        for w in wids_a & wids_b:
            paired += 1
            ra, rb = a[(t, w)], b[(t, w)]
            if (ra['owner'] == 'own') == (rb['owner'] == 'own'):
                owner_disagree += 1
            for f, tol in SHARED.items():
                g = gap(f, ra[f], rb[f])
                if g > tol:
                    bad[f] += 1
                    since.setdefault((w, f), t)
                    if g > worst[f][0]:
                        worst[f] = (g, (t, w))
                else:
                    since.pop((w, f), None)
            for f in LOCAL:
                if ra[f] != rb[f]:
                    local_differs[f] += 1
    print(f'census ticks {ticks[0]}..{ticks[-1]} ({len(ticks)} dumps); entities paired {paired}, on seat 1 only {only_a}, on seat 2 only {only_b}')
    print(f'ownership: {owner_disagree} pairs where both seats claim the same role (should be 0)')
    for f, tol in SHARED.items():
        w, at = worst[f]
        where = f' worst {w:.2f} at tick {at[0]} entity {at[1]}' if at else ''
        print(f'shared {f:9s}: {paired - bad[f]} of {paired} within {tol}{where}')
    for f in LOCAL:
        print(f'local  {f:9s}: differs in {local_differs[f]} of {paired} pairs (by design: not shared)')
    lasting = sorted(((t, w, f) for (w, f), t in since.items()), key=lambda x: x[0])
    if lasting and '--all' in sys.argv:
        print('still diverging at the last dump:')
        for t, w, f in lasting:
            print(f'  {w} {f} since tick {t}')
    elif lasting:
        print(f'{len(lasting)} field divergences still open at the last dump (--all lists them)')


if __name__ == '__main__':
    main()
