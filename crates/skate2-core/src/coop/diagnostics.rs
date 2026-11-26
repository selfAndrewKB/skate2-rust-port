//! Co-op fork: the shapes the bench's `SKATE2_DIAGNOSTIC_COOP_*` switches are written in, parsed
//! once. Each caller names its switch (the literal stays at the read, where
//! `tools/coop/switches.py` finds it) and keeps its own choices: whether a single tick is
//! trimmed, whether a list is sorted. An entry that does not parse is skipped, as a bench typo
//! must not stop a run.

/// `<tick>`, parsed as written: surrounding whitespace makes it no tick.
#[must_use]
pub fn tick(variable: &str) -> Option<u32> {
    std::env::var(variable).ok().and_then(|tick| tick.parse().ok())
}

/// `<tick>`, with surrounding whitespace ignored.
#[must_use]
pub fn tick_trimmed(variable: &str) -> Option<u32> {
    std::env::var(variable).ok().and_then(|tick| tick.trim().parse().ok())
}

/// `<tick>[,<tick>...]`, in the order written; empty when the switch is not set.
#[must_use]
pub fn ticks(variable: &str) -> Vec<u32> {
    std::env::var(variable).ok().map_or_else(Vec::new, |spec| ticks_of(&spec))
}

/// A `<tick>[,<tick>...]` list's ticks, in the order written.
#[must_use]
pub fn ticks_of(spec: &str) -> Vec<u32> {
    spec.split(',').filter_map(|tick| tick.trim().parse().ok()).collect()
}

/// `<tick>:<x>,<y>,<z>[;...]`, in the order written; an entry without exactly three
/// coordinates is skipped.
#[must_use]
pub fn tick_positions(variable: &str) -> Vec<(u32, [f32; 3])> {
    let Ok(spec) = std::env::var(variable) else { return Vec::new() };
    spec.split(';')
        .filter_map(|entry| {
            let (at, position) = entry.split_once(':')?;
            let values: Vec<f32> = position.split(',').filter_map(|v| v.trim().parse().ok()).collect();
            let [x, y, z] = values[..] else { return None };
            Some((at.trim().parse().ok()?, [x, y, z]))
        })
        .collect()
}
