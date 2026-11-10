//! Co-op fork: the campaign's shared living world (route D of `docs/coop/design.md` §3.6), the
//! parts that stand apart from the live session: the ownership rule and the world stream's
//! wire format. The session glue (what is sent each tick, the puppets, the contacts) is
//! `session/online/coop_world.rs`; the relay is `skate2-net`'s `ClientMessage::World`.

#[path = "ownership.rs"]
pub mod ownership;
#[path = "rules.rs"]
pub mod rules;
#[path = "world_stream.rs"]
pub mod world_stream;

/// `SKATE2_TRACE_COOP`: the shared world's decisions and measurements, in every build.
pub fn trace(message: std::fmt::Arguments<'_>) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("SKATE2_TRACE_COOP").is_some()) {
        eprintln!("coop: {message}");
    }
}
