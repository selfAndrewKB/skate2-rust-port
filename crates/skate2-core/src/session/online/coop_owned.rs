//! Co-op fork (M10, M11, M12): the owned AI skaters' streams on the live session: their replay
//! frames as this console sends them, and their puppets' lifecycle where they arrive. Kept
//! beside upstream's `selected_session_online.rs`, which calls it.

use super::{Arc, BTreeMap, NetFrame, SelectedLiveError, SelectedLiveSession, error, net_header, owned, remote_skater};

/// Co-op fork (M12): an owned skater's puppet hides after this many local ticks without a
/// frame (its owner despawned it, or left); the runs come every `INTERVAL` ticks at most.
pub(super) const OWNED_STALE_TICKS: u32 = 180;

/// Co-op fork (M10): whether a run of replay frames ends at `tick` for runs of `interval` ticks.
pub(super) const fn ends_at(tick: u32, interval: u32) -> bool {
    tick % interval == 0
}

/// Co-op fork (M10): an owned skater's frame as it goes out: its slot, its recipe, the look to
/// send first (when that recipe's has not gone for the slot), and the frame.
pub(super) type OwnedNetFrame = (u8, u64, Option<Vec<u8>>, NetFrame);

/// Co-op fork (M10): the replay frames of the challenge AI skaters this console owns (the
/// slots after the campaign's `humans` with a challenge controller), each as its own skater's
/// go out: the header, the pose on a sample tick, the twist ratios, and the actor's audio record
/// (its wheels and landings, relative to its root as the player's own travels; M11.1); no
/// camera and no trick HUD (a puppet has none to show); a teleporting tick is a sample, as the
/// local skater's is. With each the skater's recipe and, when the look of that recipe has not
/// gone for the slot yet (`sent`), the look to send first.
pub(super) fn owned_ai_net_frames<T, C>(
    inner: &mut super::super::SelectedSession<T, C>,
    humans: usize,
    tick: u32,
    sample: bool,
    vault: &Arc<crate::attribute_runtime::AttributeRuntimeVault>,
    sent: &BTreeMap<u8, u64>,
    challenge: bool,
) -> Result<Vec<OwnedNetFrame>, SelectedLiveError> {
    let challenge_slots: Vec<u32> = inner
        .ai_skaters
        .iter()
        .map(crate::selected_session_boot_owner::party_skaters::PartySkater::player_index)
        .filter(|&slot| usize::try_from(slot).is_ok_and(|slot| slot >= humans))
        .filter(|&slot| (inner.ai.as_ref().and_then(|ai| ai.client_type(i32::try_from(slot).unwrap_or(-1))) == Some(crate::ai::actor::AiClientType::Challenge)) == challenge)
        .collect();
    let mut frames = Vec::with_capacity(challenge_slots.len());
    for skater in inner.ai_skaters.iter_mut().filter(|skater| challenge_slots.contains(&skater.player_index())) {
        let Ok(slot) = u8::try_from(skater.player_index()) else { continue };
        if skater.presentation_runtime.borrowed() {
            continue;
        }
        let Some(frame) = skater.presentation_runtime.submitted_frame(&skater.presentation, vault).map_err(error)? else { continue };
        let recipe_id = skater.recipe_load.asset_id().unwrap_or(0);
        let recipe = if sent.get(&slot) == Some(&recipe_id) {
            None
        } else {
            match remote_skater::encode_character(&frame) {
                Ok(data) => Some(data),
                Err(e) => {
                    crate::coop::trace(format_args!("owned AI skater at slot {slot}: its look cannot be sent: {e}"));
                    continue;
                }
            }
        };
        let header = net_header(&frame.header);
        let teleport = header[48] & 0x1000 != 0;
        let pose = (sample || teleport).then(|| {
            frame
                .pose
                .iter()
                .map(|sqt| {
                    let mut lanes = [0.0; 12];
                    lanes[0..4].copy_from_slice(&sqt.scale);
                    lanes[4..8].copy_from_slice(&sqt.rotation);
                    lanes[8..12].copy_from_slice(&sqt.translation);
                    lanes
                })
                .collect()
        });
        let root = [header[36], header[37], header[38]].map(f32::from_bits);
        let audio = skater
            .actor
            .cached_physical_allocation()
            .ok()
            .and_then(|physical| physical.owner().phys_out().lock().ok().and_then(|output| output.try_audio_phys_out().ok()));
        let (audio_floats, audio_words) = audio.map(|record| crate::online::audio_record::flatten(&crate::online::audio_record::relative_to_root(record, root))).unwrap_or_default();
        frames.push((slot, recipe_id, recipe, NetFrame { tick, header, pose, twist: frame.twist_ratios, camera: Vec::new(), audio_floats, audio_words, hud: Vec::new() }));
    }
    Ok(frames)
}

impl<T, C> SelectedLiveSession<T, C> {
    /// Co-op fork (M10): an owned challenge AI skater's puppet shows no more: its owner
    /// destroyed the skater (the round's frames stopped naming its slot).
    pub(in super::super) fn online_hide_owned_puppet(&mut self, owner: u8, slot: i32) {
        if let Some(remote) = u8::try_from(slot).ok().and_then(|slot| self.online.game.as_mut()?.remotes.get_mut(&owned::skater_id(owner, slot))) {
            crate::coop::trace(format_args!("owned AI skater at slot {slot} of player {owner}: its puppet hides"));
            remote.hide();
        }
    }

    /// Co-op fork (M11): this console takes an owned AI skater over: its puppet goes, its actor
    /// comes (the hand-off, open item 65).
    pub(in super::super) fn online_drop_owned_puppet(&mut self, slot: i32) {
        let Ok(slot) = u8::try_from(slot) else { return };
        self.online.owned_runs.remove(&slot);
        self.online.owned_teleported.remove(&slot);
        if let Some(game) = self.online.game.as_mut() {
            // The old owner's puppet of the slot, whoever it was.
            game.remotes.retain(|&id, _| owned::slot_of_id(id).is_none_or(|(_, owned)| owned != slot));
            game.owned_sent.remove(&slot);
        }
    }

    /// Co-op fork (M10): an owned skater's slot was destroyed, the round's frames say: its
    /// unsent frames go (a destroyed skater's run never completes), and the look of the next
    /// skater at the slot goes at its first frame, whatever its recipe (M11.1: on the
    /// destruction alone; the creation's clearing sent every look twice).
    pub(in super::super) fn online_owned_slot_changed(&mut self, slot: i32) {
        let Ok(slot) = u8::try_from(slot) else { return };
        self.online.owned_runs.remove(&slot);
        self.online.owned_teleported.remove(&slot);
        if let Some(game) = self.online.game.as_mut() {
            game.owned_sent.remove(&slot);
        }
    }

    /// Co-op fork (M10): the owned skaters' puppets and the record of the looks sent go with
    /// the campaign's round.
    pub(in super::super) fn online_drop_owned_puppets(&mut self) {
        self.online.owned_runs.clear();
        self.online.owned_teleported.clear();
        self.online.owned_sound.clear();
        if let Some(game) = self.online.game.as_mut() {
            game.remotes.retain(|&user_id, _| owned::slot_of_id(user_id).is_none());
            game.owned_sent.clear();
        }
    }
}
