"""A seat log's co-op records, read in one place (M26).

Since M26 the game writes every measurement and decision a tool reads as a structured record
beside the trace's prose: one line, `coop-event ` and a JSON object whose `e` names the kind
(`skate2-core`'s `coop::event`). The tools read those with a JSON parser, never the prose, so a
reworded trace line breaks nothing. Two things are still read from prose, here alone:

- logs written before M26 (the crawl ledger regenerates from every crawl since 2026-11-10):
  each kind's `LEGACY` pattern turns the old trace line into the same record;
- upstream's own lines, whose wording is upstream's (`career graph … enter …`, `ai: …`,
  `diagnostic: …`): `UPSTREAM` patterns, the only prose read for a log of any age.

Kind by kind: a kind the log has any `coop-event` of is read from its events alone (its legacy
line is not matched), so an event and its legacy line are never both counted; a kind with no
event in the log is read from its legacy line, so a kind the game does not emit yet still reads.

Besides the records, the diagnostics that are searches over the prose by nature (a seat that went
down, the career script's errors, the words that mark something unported) are here too, so the
tools (the audits, crawl.py, crawl_ledger.py) hold no log pattern of their own.

    for record in events.read(path, {'census', 'plays'}):
        record['e'], record['tick'], ...
    events.read(path, {'graph'}, stop=events.CRAWL_STOP)   # a crawl seat log, without its teardown
"""
import json
import re
from collections import Counter

PREFIX = 'coop-event '

# Each kind: (pattern, fields): the pattern's groups in order, each `name` or `name:type`
# (`int`, `float`, `bool` = True for `true`, `hexlist` = a bracketed list of hex numbers, `hex` kept
# as text, `own` = the census owner: True for `own`, else the slot). A kind with no fields is a
# line counted.
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
    # the crawl's drivers and what the campaign did with them (crawl.py, crawl_ledger.py); a
    # completion's result is logged since coop-v0.9.1 (None before: the 2026-11-10 crawl)
    'complete_challenge': (r'coop: diagnostic: complete challenge (\S+) at round tick (\d+)(?:: (\w+))?', ['activity', 'tick:int', 'result:bool']),
    'execute': (r'coop: diagnostic: execute (\S+) at round tick (\d+): (\w+)', ['activity', 'tick:int', 'result:bool']),
    'won': (r'coop: diagnostic: won the main challenge at round tick (\d+): (\d+) objectives completed', ['tick:int', 'objectives:int']),
    'campaign_save': (r'coop: campaign save at session (\d+) round tick (\d+): (\d+) B, hash ([0-9a-f]+)', ['session:int', 'tick:int', 'bytes:int', 'hash']),
    'rule': (r'coop: rule: (\S+) \(([0-9a-f]+)\) is (\w+(?: \{[^}]*\})?): objectives for players \[([^\]]*)\]', ['activity', 'key', 'rule', 'players']),
    'objective': (r'coop: objective ([0-9a-f]+) of player (\d+): (\w+) -> (\w+)', ['objective', 'player:int', 'before', 'after']),
    'withheld': (r'coop: (.+): withheld$', ['what']),
    'presentation': (r'coop: presentation group (\d+): event (\d+) of (\d+), kind (\d+)', ['group:int', 'event:int', 'events:int', 'event_kind:int']),
    'presentation_finished': (r'coop: presentation group (\d+) finished', ['group:int']),
    'scene_start': (r'coop: scene (\S+) starts at tick (\d+)', ['scene', 'tick:int']),
    'scene_characters': (r'coop: scene (\S+) instanced (\d+) characters', ['scene', 'characters:int']),
    'trigger': (r'coop: trigger ([0-9a-f]+) entered by player (\d+)', ['trigger', 'player:int']),
    'prompt_applied': (r'coop: (?:prompt|note) .*applied|answer .* applied', []),
    # the AI manager's decisions (SKATE2_TRACE_AI, since 2026-11-11; the fork's, under `ai: `)
    'demo_characters': (r'ai: SelectDemoCharacters: (\d+) competitors', ['competitors:int']),
    'select_characters': (r'ai: SelectCharacters for activity (0x[0-9a-f]+): (\d+) competitors, characters \[([^\]]*)\]', ['activity', 'competitors:int', 'characters:hexlist']),
    # synchronization windows
    'live_audit': (r'coop: live audit at (\d+): (\d+) samples; position within [\d.]+ m (\d+) \(worst ([\d.]+) m, \w+\), yaw within \d+° (\d+) \(worst (\d+)°, \w+\), speed within [\d.]+ m/s (\d+), flags equal (\d+)',
                   ['tick:int', 'samples:int', 'pos_ok:int', 'worst_pos_m:float', 'yaw_ok:int', 'worst_yaw_deg:int', 'speed_ok:int', 'flags_ok:int']),
    # `refused` is logged since the stand-up fix (None before)
    'world_stream': (r'coop: world stream at (\d+): (\d+) runs, (\d+) entity samples, (\d+) prop samples, (\d+) B \((\d+) B/s, (\d+) B/s per streamed pedestrian\), (\d+) owned in range of (\d+) pedestrians here, (\d+) puppets, (\d+) props owned of (\d+) known, contacts sent (\d+) applied (\d+) confirmed (\d+)(?: refused (\d+))?, (\d+) records ignored',
                     ['tick:int', 'runs:int', 'entity_samples:int', 'prop_samples:int', 'bytes:int', 'bps:int', 'bps_per_pedestrian:int', 'owned:int', 'pedestrians:int', 'puppets:int',
                      'props_owned:int', 'props_known:int', 'contacts_sent:int', 'contacts_applied:int', 'contacts_confirmed:int', 'contacts_refused:int', 'ignored:int']),
    # ownership and puppets, counted
    'handed_over': (r'coop: .*handed to', []),
    'promoted': (r'coop: .*promoted', []),
    'demoted': (r'coop: .*demoted', []),
    'knocked_down': (r'coop: .*knocked down', []),
    # the dedicated server's log: the campaign round's start and each save it heard
    'server_round': (r'^(?=.*started with).*?(campaign round.*)$', ['line']),
    'server_save': (r'^(?=.*campaign save from).*?(campaign save.*)$', ['line']),
}

# Upstream's own lines: read as prose in every log, their wording being upstream's.
UPSTREAM = {
    'graph': (r'career graph (\S+?)(?: @(\d+))?: (?:enter (\w+)|(.*))$', ['activity', 'tick:int', 'state', 'what']),
    'clock_dropped': (r'clock: a stall let (\d+) ticks go', ['ticks:int']),
    'ai_player': (r'career: AI player (\d+) character (0x[0-9a-f]+) names .* => \("([^"]*)"', ['player:int', 'character', 'name']),
    'round_stall': (r'online: round at tick (\d+): (\d+) ready, stalled (\d+) of the last (\d+) ticks', ['tick:int', 'ready:int', 'stalled:int', 'window:int']),
    'remote_skater': (r'online: (\w+): (\d+) of (\d+) ticks at or past the newest sample, delay ([\d.]+), pace ([\d.]+), (\d+) teleport frames', ['user', 'held:int', 'shown:int', 'delay:float', 'pace:float', 'teleports:int']),
    'server_build': (r'build ([0-9a-f]{8})', ['build']),
}

# The crawl's marker, written into each seat log when a mission's run ends (crawl.py): what
# follows is the bench's teardown.
CRAWL_STOP = 'crawl: stop'

FAULT = re.compile(r'panicked|controlled .*failed|Tick\(')
DROPPED = re.compile(r'panicked|kicked')
SCRIPT_ERROR = 'career script error'
TELEPORT_UNKNOWN = 'teleport to unknown location'
NOISE = re.compile(r'controlled frame|unlit diffuse|CONTROLLED FIRST|first-test boot|coop: world stream at|coop: live audit|DMO models')
UNEXPECTED = re.compile(r'unknown|unported|unsupported|unhandled|not ported|not implemented|unimplemented|script error|ScriptError|TODO|ignored|provisional', re.I)
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
        elif kind_of == 'bool':
            record[name] = value == 'true'
        elif kind_of == 'own':
            record[name] = True if value == 'own' else int(value.rsplit(' ', 1)[1])
        elif kind_of == 'down':
            record[name] = value == 'down'
        elif kind_of == 'first':
            record[name] = value == 'the first difference'
        elif kind_of == 'parts':
            record[name] = value.split('+')
        elif kind_of == 'hexlist':
            record[name] = [int(part, 16) for part in value.split(',') if part.strip()]
        else:
            record[name] = value
    return record


def _lines(path, stop=None):
    """The log's lines, up to the first that starts with `stop` (all when None)."""
    with open(path, encoding='utf-8', errors='replace') as log:
        for line in log:
            line = line.rstrip('\n')
            if stop is not None and line.startswith(stop):
                return
            yield line


def lines(path, stop=None):
    """The log's lines as a list (none when it cannot be read), up to `stop` as `read` takes it."""
    try:
        return list(_lines(path, stop))
    except OSError:
        return []


def event_kinds(path, stop=None):
    """The kinds the log has structured records of."""
    kinds = set()
    for line in _lines(path, stop):
        at = line.find(PREFIX)
        if at >= 0:
            try:
                kinds.add(json.loads(line[at + len(PREFIX):]).get('e'))
            except ValueError:
                continue
    return kinds


def has_events(path, stop=None):
    return bool(event_kinds(path, stop))


def read(path, kinds=None, stop=None):
    """The log's records of `kinds` (all when None), in order: each kind from its events where the
    log has any of it, else from its legacy lines; upstream's lines always. A line yields a record
    for every kind it matches. `stop`: a line prefix; the log is read up to the first line that
    starts with it."""
    structured = event_kinds(path, stop)
    active = [(kind, pattern, fields) for kind, (pattern, fields) in _COMPILED.items()
              if (kinds is None or kind in kinds) and not (kind in structured and kind in LEGACY)]
    for line in _lines(path, stop):
        at = line.find(PREFIX)
        if at >= 0:
            try:
                record = json.loads(line[at + len(PREFIX):])
            except ValueError:
                continue
            if kinds is None or record.get('e') in kinds:
                yield record
            continue
        for kind, pattern, fields in active:
            if m := pattern.search(line):
                yield _convert(kind, m.groups(), fields)


# Diagnostics over the prose: searches for words, not fields, so they read every log's text
# whatever its age. Each takes the log's lines (`lines`), cut where the caller cuts them.

def faults(path):
    """The log's fault lines (a panic, a failed controlled frame)."""
    with open(path, encoding='utf-8', errors='replace') as log:
        return [line.rstrip('\n') for line in log if FAULT.search(line)]


def dropped(log, tail=0):
    """The lines that say a seat went down: a panic, a kick, a disconnect; a disconnect in the last
    `tail` lines is not counted (a log without a stop marker ends in the teardown's). The crawl's
    own stop line is never one."""
    return [line for index, line in enumerate(log) if CRAWL_STOP not in line
            and (DROPPED.search(line) or ('disconnected' in line and len(log) - index > tail))]


def normalize(line):
    """A line with its numbers and hashes blanked, so the same message counts once."""
    line = re.sub(r'0x[0-9a-f]+|\b[0-9a-f]{8,}\b', '#', line)
    line = re.sub(r'(?<=[ (:])-?\d+(\.\d+)?', 'N', line)
    return line[:200]


def script_errors(log):
    """The career script's errors (upstream's `career script error in <context>: <error>`), each
    from its context on, normalized, counted."""
    return Counter(normalize(re.sub(r'^.*career script error in ', '', line)) for line in log if SCRIPT_ERROR in line)


def teleports_unknown(log):
    """The lines naming a teleport to a location the port does not know (upstream's `career: …`)."""
    return [line for line in log if TELEPORT_UNKNOWN in line]


def unexpected(log):
    """Lines with a word that marks something unported or unknown (besides the script errors and
    the windows' routine lines), normalized, counted: a finding each."""
    return Counter(normalize(line) for line in log
                   if UNEXPECTED.search(line) and not NOISE.search(line) and SCRIPT_ERROR not in line and 'ScriptError' not in line)


def compare(path):
    """For a log with both: each kind's structured records against what its legacy lines give,
    in order (M26's proof that an event says what its prose says). Floats compare to the prose's
    printed precision. Returns {kind: (events, legacy, first mismatch or None)}."""
    structured = event_kinds(path)
    out = {}
    for kind in sorted(structured & set(LEGACY)):
        from_events = [record for record in read(path, {kind}) if record.get('e') == kind]
        pattern, fields = _COMPILED[kind]
        from_prose = []
        for line in _lines(path, None):
            if line.find(PREFIX) < 0 and (m := pattern.search(line)):
                from_prose.append(_convert(kind, m.groups(), fields))
        mismatch = None
        for index, (event, prose) in enumerate(zip(from_events, from_prose)):
            for name, value in prose.items():
                got = event.get(name)
                same = abs(got - value) <= 0.051 * max(1.0, abs(value)) if isinstance(value, float) and isinstance(got, (int, float)) else got == value
                if not same:
                    mismatch = (index, name, value, got)
                    break
            if mismatch:
                break
        if mismatch is None and len(from_events) != len(from_prose):
            mismatch = ('count', len(from_events), len(from_prose))
        out[kind] = (len(from_events), len(from_prose), mismatch)
    return out


if __name__ == '__main__':
    import sys
    if len(sys.argv) != 3 or sys.argv[1] != '--compare':
        sys.exit('usage: events.py --compare SEAT.log')
    bad = 0
    for kind, (n_events, n_prose, mismatch) in compare(sys.argv[2]).items():
        bad += mismatch is not None
        print(f'{kind}: {n_events} events, {n_prose} prose lines' + (f', MISMATCH {mismatch}' if mismatch else ', alike'))
    sys.exit(1 if bad else 0)
