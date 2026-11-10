//! `Sk8::Challenge` and `ChallengeRuntime`: career progression, missions and
//! challenges. See `docs/career/README.md`.

pub mod activity;
pub mod ai;
pub mod assets;
pub mod attrib_db;
pub mod attrib_lua;
pub mod audio_handlers;
mod binder;
mod binding;
pub mod binding_table;
pub mod callouts;
pub mod cas_unlock_parts;
pub mod characters;
pub mod competitors;
pub mod contacts;
mod contest;
pub mod crowd;
pub mod dmo;
pub mod dpo_hud;
pub mod dynamic_hull;
pub mod effects;
pub mod film;
pub mod follow_the_leader;
pub mod hom;
pub mod host;
pub mod hud;
pub mod indicators;
pub mod invisible_boundary;
pub mod keys;
pub mod leaderboard_hud;
pub mod living_world;
pub mod lua;
pub mod map_icons;
pub mod mission;
pub mod mission_audio;
mod natives;
pub mod objective_data;

/// Co-op fork, the ledger's "nearest player who is up" kind for the boxes the world tests the
/// skater against (sign-up boxes, global activities, discovery volumes, dynamic hulls): in a
/// campaign every player's tester, read from the facts alike on every console; otherwise the
/// local player's. A box any player stands in counts. Story characters are not among them:
/// each console spawns its own where its own skater is (`characters.rs`, M5).
pub(crate) fn tester_bases(world: &system::World) -> Vec<[f32; 3]> {
    if world.coop.is_some() && !world.player_states.is_empty() {
        world.player_states.iter().filter_map(|(_, state)| state.tester_base).collect()
    } else {
        world.player.tester_base.into_iter().collect()
    }
}

/// Co-op fork: whether any player the scripts read is in or just out of a Hall of Meat bail.
pub(crate) fn any_player_hom_bailing(world: &system::World) -> bool {
    if world.coop.is_some() && !world.player_states.is_empty() {
        world.player_states.iter().any(|(_, state)| state.hom_bail_in_progress || state.hom_bail_completed)
    } else {
        world.player.hom_bail_in_progress || world.player.hom_bail_completed
    }
}

/// Co-op fork: the completion rule for an activity's challenge while a campaign round runs, with
/// the campaign's players and the challenge's name; `None` outside a campaign. The one place
/// the rule table is asked (`coop::rules`).
pub(crate) fn coop_rule(world: &system::World, activity: usize) -> Option<(crate::coop::rules::Rule, Vec<i32>, String)> {
    const CHALLENGE_NAME: u64 = 0x942A_B8AE_E4B4_14ED;
    const CHALLENGE_TYPE: u64 = 0x02b8_28cc_db47_85e4;
    let campaign = world.coop.as_ref()?;
    let key = world.activities.activities.get(activity)?.key;
    let name = world.db.attribute(keys::class::CHALLENGES, key, CHALLENGE_NAME).and_then(|name| name.text(0)).unwrap_or_default();
    let family = map_icons::global_i32(world, activity, CHALLENGE_TYPE);
    Some((campaign.rules.rule_for(key, name, family), campaign.players.clone(), name.to_owned()))
}
mod objective_groups;
pub mod objective_hud;
mod objective_natives;
pub mod objectives;
mod online;
mod ots_discovery;
pub mod photo_articles;
pub mod players;
pub mod presentation;
pub mod race;
pub mod replay;
pub mod results;
pub mod runtime;
mod script_fixes;
mod sequence_connector;
mod skate3_natives;
pub mod services;
mod signpost;
pub mod signpost_progress;
pub mod signup;
pub mod signup_hud;
pub mod signup_poster;
pub mod skate;
pub mod slappys_blog;
pub mod sponsor_overlay;
pub(crate) mod sponsors;
pub mod start_hub;
pub mod stuff_menu;
pub mod system;
pub mod teleport;
pub mod teleporter;
pub mod timers;
pub mod tricks;
pub mod trigger_tester;
pub mod turn_hud;
pub mod tutorial_hud;
pub mod unlock_overlay;
pub mod unlockables;
pub mod vd_results;
pub mod vi_presentation;
pub mod visual_director;
pub mod voting;
pub mod world_painter;

#[cfg(test)]
mod tests;

pub use system::{ChallengeSystem, Event, Output};

pub mod hom_results;
pub mod hom_annals;
pub mod hom_save;
