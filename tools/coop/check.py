#!/usr/bin/env python3
"""The co-op fork's gate: lints and tests, run before a milestone is tagged.

usage: check.py [--lint] [--test]   (both when neither is given)

Lint: `cargo clippy --no-deps` over skate2-core and skate2-net (pedantic, as the workspace sets
it); fails when a warning falls on a line the fork added (`git diff` against upstream/main), and
reports upstream's own warnings as a count alone: they are upstream's to fix. Test: `cargo test`
over both crates; fails when a test fails that is not in tools/coop/known-upstream-failures.txt
(upstream's own failing tests at the version merged, by name), and names any listed test that
now passes, so the list is pruned when upstream fixes one.
"""
import collections
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
KNOWN = os.path.join(ROOT, 'tools', 'coop', 'known-upstream-failures.txt')
CRATES = ['-p', 'skate2-core', '-p', 'skate2-net']


def cargo(*args):
    return subprocess.run(['cargo', *args], cwd=ROOT, capture_output=True, text=True)


def fork_lines():
    added = collections.defaultdict(set)
    diff = subprocess.run(['git', 'diff', '-U0', 'upstream/main', '--', 'crates', 'apps'], cwd=ROOT, capture_output=True, text=True).stdout
    path = None
    for line in diff.split('\n'):
        if line.startswith('+++ b/'):
            path = line[6:]
        elif (m := re.match(r'@@ -\S+ \+(\d+)(?:,(\d+))? @@', line)) and path:
            start, count = int(m.group(1)), int(m.group(2) or 1)
            added[path].update(range(start, start + count))
    return added


def lint():
    out = cargo('clippy', *CRATES, '--no-deps', '--message-format=short')
    warnings = re.findall(r'^((?:crates|apps)/[^:]+):(\d+):\d+: (?:warning|error): (.*)$', out.stdout + out.stderr, re.M)
    if not warnings and out.returncode != 0:
        print((out.stdout + out.stderr)[-3000:])
        return False
    added = fork_lines()
    ours = [(os.path.normpath(path), line, text) for path, line, text in warnings if int(line) in added.get(os.path.normpath(path), ())]
    print(f'lint: {len(warnings)} warnings, {len(ours)} on the fork\'s lines, {len(warnings) - len(ours)} upstream\'s')
    for path, line, text in ours:
        print(f'  {path}:{line}: {text}')
    return not ours


def test():
    out = cargo('test', '-q', *CRATES)
    text = out.stdout + out.stderr
    failed = set(re.findall(r'^    ([a-z_0-9:]+)$', text, re.M))
    results = re.findall(r'^test result: \w+\. (\d+) passed; (\d+) failed', text, re.M)
    if not results:
        print(text[-3000:])
        return False
    known = {line.strip() for line in open(KNOWN) if line.strip()}
    new, fixed = sorted(failed - known), sorted(known - failed)
    passed = sum(int(p) for p, _ in results)
    print(f'test: {passed} passed, {len(failed)} failed ({len(failed & known)} upstream\'s known, {len(new)} new)')
    for name in new:
        print(f'  new failure: {name}')
    for name in fixed:
        print(f'  a known upstream failure passes now (prune it from the list): {name}')
    return not new


if __name__ == '__main__':
    want = set(sys.argv[1:]) or {'--lint', '--test'}
    ok = True
    if '--lint' in want:
        ok &= lint()
    if '--test' in want:
        ok &= test()
    print('gate:', 'pass' if ok else 'FAIL')
    sys.exit(0 if ok else 1)
