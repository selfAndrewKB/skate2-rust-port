//! Co-op fork: the campaign's shared living world (route D of `docs/coop/design.md` §3.6), the
//! parts that stand apart from the live session: the ownership rule and the world stream's
//! wire format. The session glue (what is sent each tick, the puppets, the contacts) is
//! `session/online/coop_world.rs`; the relay is `skate2-net`'s `ClientMessage::World`.

#[path = "diagnostics.rs"]
pub mod diagnostics;
#[path = "outcomes.rs"]
pub mod outcomes;
#[path = "ownership.rs"]
pub mod ownership;
#[path = "pace.rs"]
pub mod pace;
#[path = "prompts.rs"]
pub mod prompts;
#[path = "round_hash.rs"]
pub mod round_hash;
#[path = "rules.rs"]
pub mod rules;
#[path = "switches.rs"]
mod switches;
#[path = "sync_check.rs"]
pub mod sync_check;
#[path = "world_stream.rs"]
pub mod world_stream;

/// The co-op logs a console keeps in its `coop_logs` folder, newest first.
const KEPT_LOGS: usize = 20;

/// The shared world's decisions and measurements, in every build: on by default;
/// `SKATE2_TRACE_COOP=0` silences them. Each line goes to
/// stderr and to this session's `coop_logs/coop-<unix seconds>.log` beside the saves (where the
/// crash reports go), so a session the game was opened for from the desktop, whose stderr goes
/// nowhere, still leaves its log; `SKATE2_COOP_LOG=0` keeps them to stderr (the bench, whose seat
/// logs are stderr already).
pub fn trace(message: std::fmt::Arguments<'_>) {
    if on() {
        emit(&format!("coop: {message}"));
    }
}

/// A measurement or decision the tools read (M26): one line, `coop-event ` and a JSON object
/// whose `e` names the kind, beside the trace's prose and switched with it. The tools in
/// `tools/coop` read these with a JSON parser (`tools/coop/events.py`), never the prose, so
/// rewording a trace line breaks no tool; a field a tool reads is renamed only with the tool.
pub fn event(kind: &str, fields: serde_json::Value) {
    if on() {
        emit(&event_line(kind, fields));
    }
}

/// An event's line: `coop-event ` and the object of `e` (the kind) and `fields`' members.
fn event_line(kind: &str, fields: serde_json::Value) -> String {
    let mut record = serde_json::Map::new();
    record.insert("e".to_owned(), serde_json::Value::from(kind));
    if let serde_json::Value::Object(fields) = fields {
        record.extend(fields);
    }
    format!("coop-event {}", serde_json::Value::Object(record))
}

/// A non-negative figure as a trace prints it with `{:.0}` (halves to even), for an event's
/// field that a tool reads as the prose's whole number.
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a trace's rates and angles, far inside i64")]
pub fn whole(value: f64) -> i64 {
    value.round_ties_even() as i64
}

fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("SKATE2_TRACE_COOP").map_or(true, |value| value.trim() != "0"))
}

/// A line to stderr and to the session's co-op log.
fn emit(line: &str) {
    static FILE: std::sync::OnceLock<Option<std::sync::Mutex<std::io::LineWriter<std::fs::File>>>> = std::sync::OnceLock::new();
    eprintln!("{line}");
    let file = FILE.get_or_init(|| {
        if std::env::var("SKATE2_COOP_LOG").is_ok_and(|value| value.trim() == "0") {
            return None;
        }
        let file = open_log();
        if let Err(error) = &file {
            eprintln!("coop: no co-op log file ({error}); the lines go to stderr alone");
        }
        file.ok().map(|file| std::sync::Mutex::new(std::io::LineWriter::new(file)))
    });
    if let Some(mut file) = file.as_ref().and_then(|file| file.lock().ok()) {
        use std::io::Write as _;
        let _ = writeln!(file, "{line}");
    }
}

/// This session's co-op log: a new file in `coop_logs` beside the saves, the oldest beyond
/// [`KEPT_LOGS`] removed.
fn open_log() -> std::io::Result<std::fs::File> {
    let profile = crate::edit_skater_profile::EditSkaterProfileStore::default_path()?;
    let mut folder = profile.parent().map_or_else(|| std::path::PathBuf::from("coop_logs"), |saves| saves.join("coop_logs"));
    if folder.is_relative() {
        folder = std::env::current_dir()?.join(folder);
    }
    std::fs::create_dir_all(&folder)?;
    let mut logs: Vec<std::path::PathBuf> = std::fs::read_dir(&folder)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with("coop-") && std::path::Path::new(name).extension().is_some_and(|extension| extension.eq_ignore_ascii_case("log"))))
        .collect();
    logs.sort();
    for old in logs.iter().take((logs.len() + 1).saturating_sub(KEPT_LOGS)) {
        let _ = std::fs::remove_file(old);
    }
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_secs());
    let path = folder.join(format!("coop-{seconds}.log"));
    eprintln!("coop: this session's co-op log is {}", path.display());
    std::fs::OpenOptions::new().create(true).append(true).open(path)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn an_event_line_is_its_kind_then_its_fields() {
        let line = super::event_line("pace", json!({"tick": 3, "from_hz": 60, "to_hz": 62}));
        let (prefix, record) = line.split_at("coop-event ".len());
        assert_eq!(prefix, "coop-event ");
        assert!(!line.contains('\n'));
        let record: serde_json::Value = serde_json::from_str(record).unwrap();
        assert_eq!(record, json!({"e": "pace", "tick": 3, "from_hz": 60, "to_hz": 62}));
    }

    #[test]
    fn an_event_keeps_types_and_nulls() {
        let line = super::event_line("census", json!({"wid": format!("{:08x}", 0xab_u32), "x": 1.25_f32, "owner": true, "queued": null, "parts": [1_u32, 0xff]}));
        let record: serde_json::Value = serde_json::from_str(line.trim_start_matches("coop-event ")).unwrap();
        assert_eq!(record, json!({"e": "census", "wid": "000000ab", "x": 1.25, "owner": true, "queued": null, "parts": [1, 255]}));
    }

    #[test]
    fn whole_rounds_as_the_trace_prints() {
        for value in [0.5, 1.5, 2.5, 2.4999, 1234.6] {
            assert_eq!(super::whole(value).to_string(), format!("{value:.0}"));
        }
    }
}
