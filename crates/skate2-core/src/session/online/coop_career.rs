//! Co-op fork: the career's campaign methods, beside upstream's `selected_session_career.rs`
//! (its child module, so they read the career's own state): a prompt's answer as a note on
//! the round's facts, the campaign's teleport fallback, this console's AI skaters' frames, and
//! the bench's diagnostics.

use super::{Arc, CareerSession, DialogOwner, Event, MemcardLeaf, MissionFrame, PausingState, PosterAccept, SelectedLiveError, SelectedLiveSession, dynamic_hull, error, hash_value32, signpost_progress, unlockables};

/// One security region in `diagnostic_security_zones`' scan: the nearest point's squared
/// distance and the point, the points, and the box they span.
type ZoneScan = (f32, [f32; 2], u32, [f32; 4]);

/// Co-op fork: the round's online state while the career plays a campaign; none in a plain
/// career or an online challenge outside one.
pub(super) fn campaign_online(world: &crate::challenge::system::World) -> Option<&crate::challenge::players::OnlineChallenge> {
    world.online.as_ref().filter(|_| world.coop.is_some())
}

impl CareerSession {
    /// Co-op fork, the ledger's "the one who did it": a prompt answered on this console. In a
    /// campaign round it is queued for the next facts and applied at that tick everywhere
    /// (`apply_prompt`); otherwise applied now.
    pub(in super::super) fn prompt(&mut self, answer: crate::coop::prompts::PromptAnswer) {
        if self.prompt_notes_on {
            self.pending_prompts.push(answer);
        } else {
            self.apply_prompt(answer);
        }
    }

    /// Co-op fork: the answers given here since the last call.
    pub(in super::super) fn take_pending_prompts(&mut self) -> Vec<crate::coop::prompts::PromptAnswer> {
        std::mem::take(&mut self.pending_prompts)
    }

    /// Co-op fork: a prompt's answer applied: what the pad's press did before, closing the
    /// prompt where it is still up (on a console that mirrors it) and giving the scripts what
    /// they were given.
    pub(in super::super) fn apply_prompt(&mut self, answer: crate::coop::prompts::PromptAnswer) {
        use crate::coop::prompts::PromptAnswer as A;
        match answer {
            A::Yes | A::No => {
                if self.hud.dialog.is_some() && matches!(self.dialog_owner, DialogOwner::Progression | DialogOwner::Script { generic: false, .. }) {
                    self.hud.dialog = None;
                    self.hud.dialog_options = None;
                    self.dialog_owner = DialogOwner::Progression;
                }
                self.system.post(if answer == A::Yes { Event::HudYes } else { Event::HudNo });
            }
            A::Option(option) => {
                if self.hud.dialog.is_some() && matches!(self.dialog_owner, DialogOwner::Script { generic: true, .. }) {
                    self.hud.dialog = None;
                    self.hud.dialog_options = None;
                    self.dialog_owner = DialogOwner::Progression;
                }
                self.system.post(crate::challenge::teleporter::popup_option(usize::try_from(option).unwrap_or(0)));
            }
            A::Skate101Continue => {
                if self.hud.skate101.take().is_some() && matches!(self.pausing.last(), Some(PausingState::Skate101)) {
                    self.pausing.pop();
                }
                self.system.post(Event::HudContinue);
                eprintln!("career: cMsgPinPlayer(0) not applied");
            }
            A::UnlockOverlayContinue => {
                if self.hud.unlock_overlay.take().is_some() && matches!(self.pausing.last(), Some(PausingState::Unlock)) {
                    self.pausing.pop();
                }
                self.system.unlock_overlay_continue();
            }
            A::SignUpDpadUp => self.sign_up_choice_paused |= self.system.sign_up_dpad_up(),
            A::SignUpDpadReleased => self.system.sign_up_dpad_released(),
            A::SignUpChoice(choice) => {
                self.hud.sign_up_choice = None;
                self.sign_up_choice_paused = false;
                self.system.sign_up_choose(choice.and_then(|choice| usize::try_from(choice).ok()));
            }
            A::PosterAccept => match self.system.sign_up_accept() {
                PosterAccept::Accepted => {
                    self.poster_blocked = true;
                    self.hud.poster = None;
                }
                // The accept blocked the state's input, then core_error and
                // OpenPopup(eSk8FEPopup_ChallengeInfo_FeeFailed).
                PosterAccept::FeeFailed => {
                    self.poster_input_blocked = true;
                    self.sounds.push("FE_SOUND_CORE_ERROR");
                    self.dialog_owner = DialogOwner::FeeFailed;
                    self.hud.dialog_options = Some(vec!["ID_COMMON_OK".to_owned()]);
                    self.hud.dialog = Some(("ID_COMMON_WARNING_TITLE".to_owned(), "ID_CHALLENGEINFO_FEE_FAILED".to_owned(), 0));
                    self.sounds.push("FE_SOUND_CORE_POPUP");
                }
                PosterAccept::NotEnoughForMinBet => self.sounds.push("FE_SOUND_CORE_ERROR"),
                PosterAccept::Unported => {}
            },
            A::PosterDecline => {
                self.system.sign_up_decline();
                self.poster_blocked = true;
                self.hud.poster = None;
            }
            A::Service(service) => {
                let world = self.system.world_mut();
                let cash = crate::challenge::signpost_progress::cash(world);
                let cost = crate::challenge::services::cost(service);
                let available = service == 0 || crate::challenge::services::available(world, service);
                if available && cash >= cost {
                    crate::challenge::services::activate(world, service);
                    crate::coop::trace(format_args!("service {service} bought: {cost} of {cash} cash, every console's scripts start it"));
                } else {
                    crate::coop::trace(format_args!("service {service} not bought here: available {available}, cash {cash} of {cost}"));
                }
            }
            A::ServiceCancel => {
                let world = self.system.world_mut();
                if world.services.big_black_active {
                    crate::challenge::services::deactivate_security(world);
                    crate::coop::trace(format_args!("security service cancelled: every console's protector goes"));
                }
            }
        }
    }

    /// Co-op fork: whether the progression's Yes/No dialog is up (a script's `ShowDialog`).
    pub(in super::super) fn progression_dialog_open(&self) -> bool {
        self.hud.dialog.is_some() && matches!(self.dialog_owner, DialogOwner::Progression)
    }

    /// Co-op fork: the progression's Yes/No dialog answered without a pad, as `popup_input`'s A
    /// answers it; false when none is up.
    pub(in super::super) fn answer_progression_dialog(&mut self, yes: bool) -> bool {
        if !self.progression_dialog_open() {
            return false;
        }
        self.prompt(if yes { crate::coop::prompts::PromptAnswer::Yes } else { crate::coop::prompts::PromptAnswer::No });
        true
    }

    /// `cTeleportManager::TeleportPlayer` 0x828a90f8: the location's spawn point for the
    /// player's position (`GetSpawnLocation`); online, a position past the sixth without a spawn
    /// point of its own takes the sixth's. A location the registered arenas do not hold (open
    /// item 46: a mission's `*_challengelocator01` far from the streamed world) falls back to
    /// the simulation location table (`BAM_Sim.loc`, the world's own list, as the challenge
    /// system's `World::location` does), moved 5 m along -z per position as `GetSpawnLocation`
    /// moves a location without spawn points. An unknown location is traced and moves nobody.
    pub(super) fn teleport_transform(&self, location: &str, pos_id: i32, online: bool) -> Result<Option<[f32; 16]>, SelectedLiveError> {
        let hash = hash_value32(location);
        let locations = self.locations.lock().map_err(|_| error("location manager lock poisoned"))?;
        let registered = if online { locations.online_spawn_location(hash, pos_id) } else { locations.spawn_location(hash, pos_id) };
        if registered.is_some() {
            return Ok(registered);
        }
        if self.system.world().coop.is_none() {
            eprintln!("career: teleport to unknown location {location} ({pos_id}), held {:?}", locations.counts());
            return Ok(None);
        }
        // Online a position past the sixth takes the sixth's, as `online_spawn_location`.
        let position = if online { pos_id.min(5) } else { pos_id };
        let table = self.system.world().locations.get(&hash).copied().map(|mut transform| {
            #[expect(clippy::cast_precision_loss, reason = "the native (float) conversion")]
            let offset = position as f32 * -5.0;
            transform[14] += offset;
            transform
        });
        if std::env::var_os("SKATE2_TRACE_CAREER").is_some() {
            match table {
                Some(transform) => eprintln!(
                    "career: teleport to {location} ({pos_id}) from the location table at {:?}, not registered (held {:?})",
                    [transform[12], transform[13], transform[14]],
                    locations.counts()
                ),
                None => eprintln!("career: teleport to unknown location {location} ({pos_id}), held {:?}", locations.counts()),
            }
        } else if table.is_none() {
            eprintln!("career: teleport to unknown location {location} ({pos_id}), held {:?}", locations.counts());
        }
        Ok(table)
    }
}

impl<T, C> SelectedLiveSession<T, C> {
    /// Co-op fork (open item 45, M10): this console's AI skaters' completed outputs at the slots
    /// after a campaign round's humans: with `challenge` the challenge AI skaters it owns, whose
    /// frames ride its facts to every console; without it the ambient skaters, each console's
    /// own, which are no frame of the scripts' (M10.3) and are counted for the trace alone.
    pub(in super::super) fn career_ai_skater_frames(&self, humans: usize, challenge: bool) -> Result<Vec<MissionFrame>, SelectedLiveError> {
        let mut frames = Vec::with_capacity(self.inner.ai_skaters.len());
        for skater in self.career_ai_skaters(humans, challenge) {
            frames.push(self.mission_frame_of(&skater.actor, -1)?);
        }
        Ok(frames)
    }

    /// The AI skaters seated after a campaign round's humans, the challenge ones or the ambient.
    pub(in super::super) fn career_ai_skaters(&self, humans: usize, challenge: bool) -> impl Iterator<Item = &crate::selected_session_boot_owner::party_skaters::PartySkater> {
        self.inner.ai_skaters.iter().filter(move |skater| {
            let slot = skater.player_index();
            let is_challenge = i32::try_from(slot).ok().and_then(|slot| self.inner.ai.as_ref().and_then(|ai| ai.client_type(slot))) == Some(crate::ai::actor::AiClientType::Challenge);
            usize::try_from(slot).is_ok_and(|slot| slot >= humans) && is_challenge == challenge
        })
    }

    /// This console's `ChaseMan`: whether its skater is chased, and whether it is being jacked.
    pub(super) fn local_chase_state(&self) -> (bool, bool) {
        let Some(world) = self.inner.world.as_ref() else { return (false, false) };
        let Ok(world) = world.world.lock() else { return (false, false) };
        let owner = Arc::clone(world.grind_services.living_world());
        drop(world);
        let Ok(living) = owner.lock() else { return (false, false) };
        living.chase.as_ref().map_or((false, false), |chase| (chase.chase_is_on(), chase.is_skater_being_jacked()))
    }

    /// Co-op fork (M5), `SKATE2_DIAGNOSTIC_COOP_ZONES`: the security guard areas (layer 6)
    /// within `radius` of the local skater, scanned at `step`, the nearest point of each
    /// region traced (1500 m at 12 m is 62 501 lookups, a moment once).
    #[expect(clippy::cast_possible_truncation, clippy::cast_precision_loss, reason = "the scan's grid is a few hundred whole steps each way")]
    pub(in super::super) fn diagnostic_security_zones(&self, radius: f32, step: f32) {
        use crate::challenge::world_painter::layer;
        let Some(frame) = self.career.as_ref().and_then(|career| career.system.world().skaters.iter().find(|frame| frame.local).cloned()) else {
            crate::coop::trace(format_args!("diagnostic: no local skater frame to scan the security zones around"));
            return;
        };
        let [cx, _, cz] = frame.tester_base;
        let Some(world) = self.inner.world.as_ref() else { return };
        let Some(terrain) = self.inner.terrain.as_ref().and_then(|terrain| terrain.terrain_for(&world.world).ok()) else { return };
        let Ok(layers) = terrain.layers.lock() else { return };
        let mut regions: std::collections::BTreeMap<&'static str, ZoneScan> = std::collections::BTreeMap::new();
        let mut scanned = 0_u32;
        let steps = (radius / step) as i32;
        for i in -steps..=steps {
            for j in -steps..=steps {
                let point = [cx + i as f32 * step, cz + j as f32 * step];
                scanned += 1;
                let Ok(Some(name)) = layers.try_region_name(layer::SECURITY_GUARD_AREAS, point) else { continue };
                let d2 = (point[0] - cx).powi(2) + (point[1] - cz).powi(2);
                let entry = regions.entry(name).or_insert((d2, point, 0, [point[0], point[1], point[0], point[1]]));
                if d2 < entry.0 {
                    entry.0 = d2;
                    entry.1 = point;
                }
                entry.2 += 1;
                entry.3 = [entry.3[0].min(point[0]), entry.3[1].min(point[1]), entry.3[2].max(point[0]), entry.3[3].max(point[1])];
            }
        }
        crate::coop::trace(format_args!("diagnostic: security zones within {radius} m of [{cx:.1}, {cz:.1}]: {} regions in {scanned} points", regions.len()));
        for (name, (d2, point, count, [x0, z0, x1, z1])) in regions {
            crate::coop::trace(format_args!("diagnostic: security zone {name}: nearest point x {:.1} z {:.1} at {:.0} m; {count} points spanning x {x0:.0}..{x1:.0} z {z0:.0}..{z1:.0}", point[0], point[1], d2.sqrt()));
        }
    }

    /// Co-op fork (M13), diagnostics: the local skater made to wipe out, so the bench proves
    /// what a bail does to the scripts without steering a skater into anything: its deck is
    /// shot out from under it sideways before the next solve (the deck's acceleration is the
    /// bail; an upward launch of 18 m/s landed clean). A request on the physics' wipeout ledger
    /// from here is cleared by the next system-logic stage before the state selection reads
    /// it, so the cause must be physical.
    pub fn diagnostic_bail_local_player(&mut self) -> bool {
        self.diagnostic_set_board_velocity([30.0, 2.0, 0.0])
    }

    /// Co-op fork (M8): in a campaign the server's save is the truth: its progression
    /// leaves take the local file's places (the skater's own orientation stays the file's),
    /// so two players resume at the same progression whatever each file held.
    /// A new profile's career (the intro, Create-a-Skater) starts from nothing; the server's
    /// save is for a continued one.
    pub(super) fn adopt_campaign_save(&mut self, continued: bool, unlocks: &mut Option<MemcardLeaf>, hulls: &mut Option<MemcardLeaf>, saved: &mut [Option<MemcardLeaf>; 3]) {
        if self.online_campaign()
            && continued
            && let Some(adopt) = self.online.campaign_adopt.take()
        {
            let local: Vec<(&'static str, u32, Vec<Vec<u8>>)> = [
                (signpost_progress::DATA, &saved[0]),
                (signpost_progress::MESSAGES, &saved[2]),
                (unlockables::DATA, &*unlocks),
                (dynamic_hull::DATA, &*hulls),
            ]
            .into_iter()
            .filter_map(|(name, leaf)| leaf.as_ref().map(|l| (name, l.version, l.tuples.clone())))
            .collect();
            let local_hash = Self::campaign_blob_hash(&local);
            crate::coop::trace(format_args!(
                "campaign: the server's save adopted (session {} tick {}, hash {:016x}); the local file's progression {}",
                adopt.session,
                adopt.tick,
                adopt.hash,
                if local_hash == adopt.hash { "agreed".to_owned() } else { format!("differed (hash {local_hash:016x})") }
            ));
            let take = |name: &str| adopt.leaves.iter().find(|leaf| leaf.0 == name).map(|leaf| MemcardLeaf { version: leaf.1, tuples: leaf.2.clone() });
            if let Some(leaf) = take(unlockables::DATA) { *unlocks = Some(leaf); }
            if let Some(leaf) = take(dynamic_hull::DATA) { *hulls = Some(leaf); }
            if let Some(leaf) = take(signpost_progress::DATA) { saved[0] = Some(leaf); }
            if let Some(leaf) = take(signpost_progress::MESSAGES) { saved[2] = Some(leaf); }
        }
    }

    /// Co-op fork: a campaign's scripts wait for the server's round start: no update, no
    /// event flushed (the events the load queued run at the round's tick 0 on every console);
    /// only the load's own outputs (its hulls) are applied and the HUD kept.
    pub(super) fn update_held_campaign(&mut self) -> Result<(), SelectedLiveError> {
        self.apply_career_outputs()?;
        self.collect_career_hud();
        Ok(())
    }
}
