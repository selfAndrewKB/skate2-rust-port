#!/usr/bin/env python3
"""The mission crawl (M8): every campaign mission run on the two-seat bench, one bench start per
mission, the saves carried forward, each driven as its kind needs and the logs read for the end
phase and the saves' agreement. The report says which missions complete in co-op and where each
one stopped; the ledger (`crawl_ledger.py`) keeps every run.

The driver, by `challenge_global_data.Global`:
  3 or 4 (a main challenge): executed by name at round tick 900 (`SKATE2_DIAGNOSTIC_COOP_EXECUTE`),
    won by diagnostic at 1800 (`SKATE2_DIAGNOSTIC_COOP_WIN`); passes through `challenge_cleanup`.
  2 (a location challenge: the go-tos, the Training Facility teleports): executed at 900 (the
    teleport to its start location) and completed by name at 1800
    (`SKATE2_DIAGNOSTIC_COOP_COMPLETE_CHALLENGE`), as its arrival would; passes on the completion.
  0 (no global type: the `_ol` variants and a few others): completed by name at 1800 alone.
A mission that sends no save (a tutorial) passes as "no save". A stall the logs can name (the
script looping between two states for 30 s after the win) stops the mission early.

    python3 tools/coop/crawl.py --list                       # the missions (challenge_global_data m_*), by name
    python3 tools/coop/crawl.py --run [--from NAME] [--count N] [--missions a,b] [--out REPORT.md]

Needs SKATE2_CAREER_DB (the extracted career database). A mission takes one to three minutes."""
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
WAIT_S, SETTLE_S, LOOP_S = 170, 10, 30
GLOBAL_FIELD = '122db7b0f026dbec'  # challenge_global_data.Global


def missions():
    """(name, ChallengeType, Global) for every `m_*` entry of `challenge_global_data`, by name."""
    sys.path.insert(0, str(ROOT / 'tools/career'))
    from vault_dump import Db  # noqa: E402
    db = Db()
    rows = []
    for col in db.collections_of('challenge_global_data'):
        name = db.name(col)
        if name and name.startswith('m_'):
            rec = db.record('challenge_global_data', col)
            rows.append((name, int(rec.get('ChallengeType') or '0', 16), int(rec.get(GLOBAL_FIELD) or '0', 16)))
    return sorted(rows)


def driver(global_type):
    return 'main' if global_type in (3, 4) else 'location' if global_type == 2 else 'none'


def bench(*args, env=None):
    full = dict(os.environ)
    full.update(env or {})
    return subprocess.run(['./tools/coop/bench.sh', *args], cwd=ROOT, env=full, capture_output=True, text=True).stdout


def grep(path, pattern):
    try:
        return [line.rstrip('\n') for line in open(path, encoding='utf-8', errors='replace') if re.search(pattern, line)]
    except OSError:
        return []


def states(log, name):
    return [m.group(1) for m in (re.search(rf'career graph {re.escape(name)} @\d+: enter (\w+)', l) for l in grep(log, f'career graph {re.escape(name)} @')) if m]


def run_one(name, kind, keep):
    bench('stop')
    subprocess.run(['pkill', '-f', 'dist/macos/skate2-pc'], capture_output=True)
    time.sleep(2)
    extra = ['SKATE2_DIAGNOSTIC_COOP_ANSWER=no', 'SKATE2_TRACE_COOP_CENSUS=1']  # the census dump feeds the offline audit per mission
    if kind == 'main':
        extra += [f'SKATE2_DIAGNOSTIC_COOP_EXECUTE={EXECUTE_AT}:{name}', f'SKATE2_DIAGNOSTIC_COOP_WIN={WIN_AT}']
        done = 'enter challenge_cleanup|panicked|kicked|disconnected'
    elif kind == 'location':
        extra += [f'SKATE2_DIAGNOSTIC_COOP_EXECUTE={EXECUTE_AT}:{name}', f'SKATE2_DIAGNOSTIC_COOP_COMPLETE_CHALLENGE={WIN_AT}:{name}']
        done = 'diagnostic: complete challenge|panicked|kicked|disconnected'
    else:
        extra += [f'SKATE2_DIAGNOSTIC_COOP_COMPLETE_CHALLENGE={WIN_AT}:{name}']
        done = 'diagnostic: complete challenge|panicked|kicked|disconnected'
    env = {'SEAT_ONLINE_1': 'create:9', 'SEAT_ONLINE_2': 'join:9', 'CAPTURE_COUNT': '0', 'SEAT_EXTRA': ' '.join(extra)}
    if keep:
        env['BENCH_KEEP'] = '1'
    bench('start', '2', env=env)
    logs = [BENCH / f'seat-{s}/seat.log' for s in (1, 2)]
    deadline = time.time() + WAIT_S
    reason, loop_since = 'deadline', None
    while time.time() < deadline:
        time.sleep(5)
        if all(grep(log, done) for log in logs):
            reason = 'done'
            break
        # the known stall: both scripts cycling between two states after the win
        tails = [states(log, name)[-8:] for log in logs]
        if all(len(t) == 8 and len(set(t)) <= 2 for t in tails) and all(grep(log, rf'round tick {WIN_AT}\b') for log in logs):
            loop_since = loop_since or time.time()
            if time.time() - loop_since >= LOOP_S:
                reason = 'loop ' + '→'.join(dict.fromkeys(tails[0][-2:]))
                break
        else:
            loop_since = None
    if reason == 'done':
        time.sleep(SETTLE_S)  # the autosave after the end phase
    for log in logs:
        try:
            with open(log, 'a', encoding='utf-8') as f:
                f.write(f'crawl: stop ({reason}) after {WAIT_S if reason == "deadline" else int(time.time() - deadline + WAIT_S)} s\n')
        except OSError:
            pass
    result = {'name': name, 'kind': kind, 'reason': reason}
    for s, log in zip((1, 2), logs):
        executed = grep(log, f'diagnostic: execute {re.escape(name)} at round tick')
        won = grep(log, 'diagnostic: won the main challenge')
        completed = grep(log, f'diagnostic: complete challenge {re.escape(name)}')
        cleanup = grep(log, 'enter challenge_cleanup')
        saved = grep(log, 'campaign save at')
        failed = [l for l in grep(log, 'panicked|kicked|disconnected') if 'crawl: stop' not in l][:1]
        last = grep(log, 'career graph .* enter ')
        result[s] = {
            'executed': bool(executed) and executed[-1].rstrip().endswith('true'),
            'won': won[-1].split(': ')[-1] if won else '',
            'completed': bool(completed),
            'cleanup': bool(cleanup),
            'save': saved[-1].split('hash ')[-1].split(',')[0] if saved else '',
            'failed': failed[0][:120] if failed else '',
            'last_state': last[-1].split('career graph ')[-1][:80] if last else '',
        }
    server = grep(BENCH / 'server/server.log', 'campaign save from')
    result['agree'] = bool(server) and 'agrees' in server[-1]
    bench('stop')
    subprocess.run(['pkill', '-f', 'dist/macos/skate2-pc'], capture_output=True)
    kept = [(log, BENCH / f'evidence/crawl-{name}-seat-{s}.log') for s, log in zip((1, 2), logs)]
    kept.append((BENCH / 'server/server.log', BENCH / f'evidence/crawl-{name}-server.log'))
    for src, dst in kept:
        try:
            os.replace(src, dst)
        except OSError:
            pass
    return result


def verdict(r):
    if r['kind'] == 'main':
        both = all(r[s]['executed'] and r[s]['cleanup'] for s in (1, 2))
    else:
        both = all(r[s]['completed'] for s in (1, 2))
    saves = [r[s]['save'] for s in (1, 2)]
    if both and r['agree']:
        return 'PASS'
    if r['kind'] == 'main' and not all(r[s]['executed'] for s in (1, 2)):
        return 'NOT EXECUTED'
    if any('panicked' in r[s]['failed'] for s in (1, 2)):
        return 'CRASHED'
    if any(r[s]['failed'] for s in (1, 2)):
        return 'DROPPED'
    if both and not any(saves):
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
    ap.add_argument('--keep', action='store_true', help='carry the saves in from the first mission (a continued crawl)')
    ap.add_argument('--out', default=str(BENCH / 'evidence' / f'{time.strftime("%Y-%m-%d")}-crawl.md'))
    args = ap.parse_args()
    table = {n: (t, g) for n, t, g in missions()}
    names = [n for n in args.missions.split(',') if n] if args.missions else sorted(table)
    if args.list:
        for n in names:
            t, g = table.get(n, (0, 0))
            print(f'{n}\ttype {t}\tglobal {g}\t{driver(g)}')
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
            f.write('# The mission crawl\n\n| Mission | Verdict | Seat 1 last state | Seat 2 last state | Won | Save | Driver, stop |\n| --- | --- | --- | --- | --- | --- | --- |\n')
        for i, name in enumerate(names):
            kind = driver(table.get(name, (0, 0))[1])
            r = run_one(name, kind, keep=i > 0 or bool(args.start) or args.keep)
            v = verdict(r)
            won = r[1]['won'] if kind == 'main' else ('completed by name' if r[1]['completed'] else 'not completed')
            line = f"| `{name}` | {v} | {r[1]['last_state']} | {r[2]['last_state']} | {won} | {'agree' if r['agree'] else r[1]['save'] + ' / ' + r[2]['save']} | {kind}, {r['reason']} |"
            print(line, flush=True)
            f.write(line + '\n')
            f.flush()
    print(f'{len(names)} missions appended to {out}; regenerate the ledger: python3 tools/coop/crawl_ledger.py', flush=True)


if __name__ == '__main__':
    main()
