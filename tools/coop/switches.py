#!/usr/bin/env python3
"""Every environment switch the game reads, in one list: docs/coop/plan/switches.md.

usage: switches.py [--write | --check]

Scans crates/ and apps/ for the `SKATE2_*` names the code reads (a string literal naming one),
and lists each with whose it is (the co-op fork's when upstream/main has no such literal),
where it is read, and what it does: the first doc or line comment that names it in backticks
(`SKATE2_X`: ...) or, failing that, the comment just above the read. --write regenerates the
page, keeping the hand-written "Notes" section at its end; --check exits 1 when the page lists
another set of names than the code reads (skate2-core's `coop::switches` test runs the same
check, so a new switch cannot go unlisted).
"""
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PAGE = os.path.join(ROOT, 'docs', 'coop', 'plan', 'switches.md')
LITERAL = re.compile(r'"(SKATE2_[A-Z0-9_]+)"')
DOC = re.compile(r'^\s*//[/!]?\s*(.*)$')


def sources():
    for base in ('crates', 'apps'):
        for folder, dirs, files in os.walk(os.path.join(ROOT, base)):
            dirs[:] = [d for d in dirs if d not in ('target', 'fixtures')]
            for name in files:
                if name.endswith('.rs'):
                    yield os.path.join(folder, name)


def upstream_names():
    out = subprocess.run(['git', '-C', ROOT, 'grep', '-ohE', r'"SKATE2_[A-Z0-9_]+"', 'upstream/main', '--', 'crates', 'apps'], capture_output=True, text=True).stdout
    return {line.split(':', 1)[-1].strip('"') for line in out.split() if 'SKATE2_' in line}


def describe(lines, index, name):
    """The comment that says what `name` does: one naming it in backticks anywhere in the file,
    else the comment block just above its read."""
    for line in lines:
        m = DOC.match(line)
        if m and f'`{name}' in m.group(1):
            text = m.group(1)
            return text[text.index(f'`{name}'):].strip()
    block = []
    for line in reversed(lines[max(0, index - 6):index]):
        m = DOC.match(line)
        if not m:
            break
        block.insert(0, m.group(1).strip())
    return ' '.join(block)


def scan():
    found = {}
    for path in sources():
        lines = open(path, encoding='utf-8', errors='replace').read().split('\n')
        for index, line in enumerate(lines):
            for name in LITERAL.findall(line):
                entry = found.setdefault(name, {'files': set(), 'what': ''})
                entry['files'].add(os.path.relpath(path, ROOT))
                if not entry['what']:
                    entry['what'] = describe(lines, index, name)
    return found


def page(found, upstream, notes):
    def row(name):
        entry = found[name]
        what = ' '.join(entry['what'].split()).replace('|', '\\|')[:240] or '—'
        files = ', '.join(f'`{f}`' for f in sorted(entry['files'])[:3]) + (' …' if len(entry['files']) > 3 else '')
        return f'| `{name}` | {what} | {files} |'
    coop = sorted(n for n in found if n not in upstream)
    ours = sorted(n for n in found if n in upstream)
    out = [
        '# Environment switches',
        '',
        '**Scope:** every `SKATE2_*` switch the game and the server read, generated from the code by',
        '`tools/coop/switches.py --write` (do not edit the tables by hand); skate2-core\'s',
        '`coop::switches` test fails when the code reads a switch this page does not list. What each',
        'does is the comment that names it in the code. The co-op fork\'s are those upstream does not',
        f'read. {len(found)} switches: {len(coop)} the fork\'s, {len(ours)} upstream\'s.',
        '',
        '## The co-op fork\'s',
        '',
        '| Switch | What | Read in |',
        '| --- | --- | --- |',
        *(row(n) for n in coop),
        '',
        '## Upstream\'s',
        '',
        '| Switch | What | Read in |',
        '| --- | --- | --- |',
        *(row(n) for n in ours),
        '',
        '## Notes',
        '',
        notes.strip() or '(Hand-written notes on switches go here; the regeneration keeps them.)',
        '',
    ]
    return '\n'.join(out)


def listed():
    if not os.path.exists(PAGE):
        return set(), ''
    text = open(PAGE, encoding='utf-8').read()
    notes = text.split('\n## Notes\n', 1)[1] if '\n## Notes\n' in text else ''
    return set(re.findall(r'^\| `(SKATE2_[A-Z0-9_]+)` \|', text, re.M)), notes


if __name__ == '__main__':
    found = scan()
    names, notes = listed()
    if '--write' in sys.argv:
        open(PAGE, 'w', encoding='utf-8').write(page(found, upstream_names(), notes))
        print(f'{len(found)} switches written to {os.path.relpath(PAGE, ROOT)}')
    else:
        missing, stale = sorted(set(found) - names), sorted(names - set(found))
        for name in missing:
            print(f'unlisted: {name} (read in {", ".join(sorted(found[name]["files"]))})')
        for name in stale:
            print(f'listed but never read: {name}')
        sys.exit(1 if missing or stale else 0)
