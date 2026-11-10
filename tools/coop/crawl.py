#!/usr/bin/env python3
"""The mission crawl (M8): every campaign mission run on the two-seat bench, one bench start per
mission, the saves carried forward, each mission executed by name at round tick 900 and won by
diagnostic at 1800 (`SKATE2_DIAGNOSTIC_COOP_WIN`), and the logs read for the end phase and the
saves' agreement (a mission that sends no save, such as a tutorial, passes as "no save"). The report
says which missions complete in co-op and where each one stopped.

    python3 tools/coop/crawl.py --list                       # the missions (challenge_global_data m_*), by name
    python3 tools/coop/crawl.py --run [--from NAME] [--count N] [--out REPORT.md]

Needs SKATE2_CAREER_DB (the extracted career database) for --list and --run without --missions;
`--missions a,b,c` runs those instead. One mission takes about three minutes."""
import argparse
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BENCH = Path(os.environ.get('BENCH', ROOT.parent / 'bench'))
EXECUTE_AT, WIN_AT = 900, 1800
WAIT_S, SETTLE_S = 170, 10


def missions():
    sys.path.insert(0, str(ROOT / 'tools/career'))
    from vault_dump import Db  # noqa: E402
    db = Db()
    rows = []
    for col in db.collections_of('challenge_global_data'):
        name = db.name(col)
        if name and name.startswith('m_'):
            rec = db.record('challenge_global_data', col)
            rows.append((name, int(rec.get('ChallengeType') or '0', 16)))
    return sorted(rows)


def bench(*args, env=None):
    full = dict(os.environ)
    full.update(env or {})
    return subprocess.run(['./tools/coop/bench.sh', *args], cwd=ROOT, env=full, capture_output=True, text=True).stdout


def grep(path, pattern):
    try:
        return [line.rstrip('\n') for line in open(path, encoding='utf-8', errors='replace') if re.search(pattern, line)]
    except OSError:
        return []


def run_one(name, keep):
    bench('stop')
    subprocess.run(['pkill', '-f', 'dist/macos/skate2-pc'], capture_output=True)
    time.sleep(2)
    env = {
        'SEAT_ONLINE_1': 'create:9', 'SEAT_ONLINE_2': 'join:9', 'CAPTURE_COUNT': '0',
        'SEAT_EXTRA': f'SKATE2_DIAGNOSTIC_COOP_ANSWER=no SKATE2_DIAGNOSTIC_COOP_EXECUTE={EXECUTE_AT}:{name} SKATE2_DIAGNOSTIC_COOP_WIN={WIN_AT}',
    }
    if keep:
        env['BENCH_KEEP'] = '1'
    bench('start', '2', env=env)
    logs = [BENCH / f'seat-{s}/seat.log' for s in (1, 2)]
    deadline = time.time() + WAIT_S
    while time.time() < deadline:
        time.sleep(5)
        if all(grep(log, 'enter challenge_cleanup|panicked|kicked|disconnected') for log in logs):
            break
    time.sleep(SETTLE_S)
    result = {'name': name}
    for s, log in zip((1, 2), logs):
        executed = grep(log, f'diagnostic: execute {re.escape(name)} at round tick')
        won = grep(log, 'diagnostic: won the main challenge')
        cleanup = grep(log, 'enter challenge_cleanup')
        saved = grep(log, 'campaign save at')
        failed = grep(log, 'panicked|kicked|disconnected')
        last = grep(log, 'career graph .* enter ')
        result[s] = {
            'executed': bool(executed) and executed[-1].rstrip().endswith('true'),
            'won': won[-1].split(': ')[-1] if won else '',
            'cleanup': bool(cleanup),
            'save': saved[-1].split('hash ')[-1].split(',')[0] if saved else '',
            'failed': failed[-1][:120] if failed else '',
            'last_state': last[-1].split('career graph ')[-1][:80] if last else '',
        }
    server = grep(BENCH / 'server/server.log', 'campaign save from')
    result['agree'] = bool(server) and 'agrees' in server[-1]
    bench('stop')
    subprocess.run(['pkill', '-f', 'dist/macos/skate2-pc'], capture_output=True)
    for s, log in zip((1, 2), logs):
        try:
            os.replace(log, BENCH / f'evidence/crawl-{name}-seat-{s}.log')
        except OSError:
            pass
    return result


def verdict(r):
    both = all(r[s]['executed'] and r[s]['cleanup'] for s in (1, 2))
    if both and r['agree']:
        return 'PASS'
    if not all(r[s]['executed'] for s in (1, 2)):
        return 'NOT EXECUTED'
    if any(r[s]['failed'] for s in (1, 2)):
        return 'CRASHED'
    if both and not any(r[s]['save'] for s in (1, 2)):
        return 'PASS (no save)'
    if both:
        return 'PASS (saves differ)'
    return 'STOPPED'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--list', action='store_true')
    ap.add_argument('--run', action='store_true')
    ap.add_argument('--missions', default='')
    ap.add_argument('--from', dest='start', default='')
    ap.add_argument('--count', type=int, default=0)
    ap.add_argument('--out', default=str(BENCH / 'evidence' / f'{time.strftime("%Y-%m-%d")}-crawl.md'))
    args = ap.parse_args()
    names = [n for n in args.missions.split(',') if n] if args.missions else [n for n, _ in missions()]
    if args.list:
        for n in names:
            print(n)
        return
    if not args.run:
        ap.print_help()
        return
    if args.start and args.start in names:
        names = names[names.index(args.start):]
    if args.count:
        names = names[:args.count]
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    header = not out.exists()
    with open(out, 'a', encoding='utf-8') as f:
        if header:
            f.write('# The mission crawl\n\n| Mission | Verdict | Seat 1 last state | Seat 2 last state | Won | Save |\n| --- | --- | --- | --- | --- | --- |\n')
        for i, name in enumerate(names):
            r = run_one(name, keep=i > 0 or bool(args.start))
            v = verdict(r)
            line = f"| `{name}` | {v} | {r[1]['last_state']} | {r[2]['last_state']} | {r[1]['won']} | {'agree' if r['agree'] else r[1]['save'] + ' / ' + r[2]['save']} |"
            print(line, flush=True)
            f.write(line + '\n')
            f.flush()


if __name__ == '__main__':
    main()
