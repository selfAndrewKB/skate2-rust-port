"""A seat log's co-op records, read in one place (M26).

Since M26 the game writes every measurement and decision a tool reads as a structured record
beside the trace's prose: one line, `coop-event ` and a JSON object whose `e` names the kind
(`skate2-core`'s `coop::event`). The tools read those with a JSON parser, never the prose, so a
reworded trace line breaks nothing. Two things are still read from prose, here alone:

- logs written before M26 (the crawl ledger regenerates from every crawl since 2026-11-10):
  each kind's `LEGACY` pattern turns the old trace line into the same record;
- upstream's own lines, whose wording is upstream's (`career graph … enter …`, `ai: …`,
  `diagnostic: …`): `UPSTREAM` patterns, the only prose read for a log of any age.

A log with any `coop-event` line is read from its events alone (its prose is not matched), so
an event and its legacy line are never both counted.

    for record in events.read(path, {'census', 'plays'}):
        record['e'], record['tick'], ...
"""
import json
import re

PREFIX = 'coop-event '

# Each kind: (pattern, fields): the pattern's groups in order, each `name` or `name:type`
# (`int`, `float`, `hex` kept as text, `own` = the census owner: True for `own`, else the slot).
LEGACY = {
    'census': (r'coop: census (\d+): ([0-9a-f]{8}) (own|puppet of \d+) e(\d+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) yaw (-?[\d.]+) v ([\d.]+) f(\d+) m([0-9a-f]+) g(\d+) moods(\d+) (down|up)(?: i (?:q(\d+) (\S*)|none))?$',
               ['tick:int', 'wid', 'owner:own', 'epoch:int', 'x:float', 'y:float', 'z:float', 'yaw:float', 'speed:float', 'flags:int', 'model', 'graph:int', 'moods:int', 'down:down', 'queued:int', 'intents']),
    'plays': (r'coop: census (\d+): ([0-9a-f]{8}) plays (\S*)$', ['tick:int', 'wid', 'intents']),
    'puppet_jerk': (r'coop: puppet jerk at (\d+): (\d+) ticks, mean \S+ mm, (\d+) past 5 mm', ['tick:int', 'ticks:int', 'past5:int']),
    'own_jerk': (r"coop: own pedestrians' jerk at (\d+): (\d+) ticks, mean \S+ mm, (\d+) past 5 mm", ['tick:int', 'ticks:int', 'past5:int']),
    'round_lead': (r'online: produced (\d+), processed (\d+)', ['produced:int', 'processed:int']),
    'pace': (r'coop: pace at tick (\d+): timer (\d+) -> (\d+) Hz', ['tick:int', 'from_hz:int', 'to_hz:int']),
    'notice_shown': (r'coop: waiting for players at tick (\d+)', ['tick:int']),
    'notice_down': (r'coop: waiting for players over at tick (\d+) after (\d+) simulation ticks', ['tick:int', 'waited:int']),
    'sync_standing': (r'coop: sync check with ([0-9a-f]+): (\d+) checks, (\d+) differed, first difference (?:none|tick (\d+) in (\S+))$', ['user', 'checked:int', 'differed:int', 'first_tick:int', 'first_parts:parts']),
    'sync_differs': (r'coop: sync check: player (\d+) differs at tick (\d+) in (\S+) \((the first difference|after agreeing again);', ['player:int', 'tick:int', 'parts:parts', 'first:first']),
    'sync_unknown': (r'coop: sync check of player (\d+) at tick (\d+): no record', ['player:int', 'tick:int']),
    'sync_parts': (r'coop: sync parts @(\d+): \[([0-9a-f, ]+)\]', ['tick:int', 'parts:hexlist']),
    'sync_dump': (r'coop: sync dump @(\d+) (\w+): (.*)$', ['tick:int', 'part', 'line']),
    'desync_planted': (r"coop: diagnostic: a divergence planted in (.+)'s vars at tick (\d+)", ['activity', 'tick:int']),
}

# Upstream's own lines: read as prose in every log, their wording being upstream's.
UPSTREAM = {
    'graph': (r'career graph (\S+?)(?: @(\d+))?: (?:enter (\w+)|(.*))$', ['activity', 'tick:int', 'state', 'what']),
    'clock_dropped': (r'clock: a stall let (\d+) ticks go', ['ticks:int']),
}

FAULT = re.compile(r'panicked|controlled .*failed|Tick\(')
_COMPILED = {kind: (re.compile(pattern), fields) for kind, (pattern, fields) in {**LEGACY, **UPSTREAM}.items()}


def _convert(kind, groups, fields):
    record = {'e': kind}
    for value, field in zip(groups, fields):
        name, _, kind_of = field.partition(':')
        if value is None:
            record[name] = None
        elif kind_of == 'int':
            record[name] = int(value)
        elif kind_of == 'float':
            record[name] = float(value)
        elif kind_of == 'own':
            record[name] = True if value == 'own' else int(value.rsplit(' ', 1)[1])
        elif kind_of == 'down':
            record[name] = value == 'down'
        elif kind_of == 'first':
            record[name] = value == 'the first difference'
        elif kind_of == 'parts':
            record[name] = value.split('+')
        elif kind_of == 'hexlist':
            record[name] = [int(part, 16) for part in value.split(',')]
        else:
            record[name] = value
    return record


def has_events(path):
    with open(path, encoding='utf-8', errors='replace') as log:
        return any(line.startswith(PREFIX) or f' {PREFIX}' in line for line in log)


def read(path, kinds=None):
    """The log's records of `kinds` (all when None), in order: its events if it has any, else its
    legacy lines; upstream's lines always."""
    structured = has_events(path)
    with open(path, encoding='utf-8', errors='replace') as log:
        for line in log:
            line = line.rstrip('\n')
            at = line.find(PREFIX)
            if at >= 0:
                try:
                    record = json.loads(line[at + len(PREFIX):])
                except ValueError:
                    continue
                if kinds is None or record.get('e') in kinds:
                    yield record
                continue
            for kind, (pattern, fields) in _COMPILED.items():
                if (kinds is not None and kind not in kinds) or (structured and kind in LEGACY):
                    continue
                if m := pattern.search(line):
                    yield _convert(kind, m.groups(), fields)
                    break


def faults(path):
    """The log's fault lines (a panic, a failed controlled frame)."""
    with open(path, encoding='utf-8', errors='replace') as log:
        return [line.rstrip('\n') for line in log if FAULT.search(line)]
