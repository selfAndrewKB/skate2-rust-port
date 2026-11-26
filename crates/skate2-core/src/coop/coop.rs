//! Co-op fork: the campaign's shared living world (route D of `docs/coop/design.md` §3.6), the
//! parts that stand apart from the live session: the ownership rule and the world stream's
//! wire format. The session glue (what is sent each tick, the puppets, the contacts) is
//! `session/online/coop_world.rs`; the relay is `skate2-net`'s `ClientMessage::World`.

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
#[path = "sync_check.rs"]
pub mod sync_check;
#[path = "world_stream.rs"]
pub mod world_stream;

/// The co-op logs a console keeps in its `coop_logs` folder, newest first.
const KEPT_LOGS: usize = 20;

/// The shared world's decisions and measurements, in every build: on by default while co-op is
/// played and tested (2026-11-26), `SKATE2_TRACE_COOP=0` silences them. Each line goes to
/// stderr and to this session's `coop_logs/coop-<unix seconds>.log` beside the saves (where the
/// crash reports go), so a session the game was opened for from the desktop, whose stderr goes
/// nowhere, still leaves its log; `SKATE2_COOP_LOG=0` keeps them to stderr (the bench, whose seat
/// logs are stderr already).
pub fn trace(message: std::fmt::Arguments<'_>) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    static FILE: std::sync::OnceLock<Option<std::sync::Mutex<std::io::LineWriter<std::fs::File>>>> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("SKATE2_TRACE_COOP").map_or(true, |value| value.trim() != "0")) {
        return;
    }
    eprintln!("coop: {message}");
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
        let _ = writeln!(file, "coop: {message}");
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
        .filter(|path| path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with("coop-") && name.ends_with(".log")))
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
