//! Co-op fork: the campaign lobby on the live session: the career started with the online game
//! attached, entered, reloaded from the server's save (M8), and dropped and rejoined when the
//! connection is lost. Kept beside upstream's `selected_session_online.rs`, which calls it.

use super::{ClientMessage, LossReason, MatchRequest, OnlineNotice, PlayNowMode, SelectedLiveError, SelectedLiveSession, challenge, coop_campaign, coop_world, error, trace};

/// Co-op fork (M8): the campaign save the server holds, decoded for the career's load.
pub(in super::super) struct AdoptedSave {
    pub(in super::super) session: u32,
    pub(in super::super) tick: u32,
    pub(in super::super) hash: u64,
    pub(in super::super) leaves: Vec<coop_campaign::Leaf>,
}

/// Co-op fork: a campaign lobby's "start", which `start_online_campaign` never reads: the career
/// places its own skater.
pub(super) const CAMPAIGN_START: [f32; 16] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];

impl<T, C> SelectedLiveSession<T, C> {
    /// Co-op fork: the online game is a campaign lobby (`docs/coop/design.md`): the career plays
    /// with the game attached rather than the online free skate.
    pub(in super::super) fn online_campaign(&self) -> bool {
        self.online.game.as_ref().is_some_and(|game| game.attributes.challenge_type == challenge::CAMPAIGN)
    }

    /// Co-op fork: a campaign lobby entered from the main menu or the single player hub. The
    /// career starts as New Career or Continue Career would, with the online game attached;
    /// nothing of the online free skate (its area, its frozen hulls, its hub) applies, the
    /// career's own stand. `start_career` then enters the game (`online_campaign_entered`).
    pub(in super::super) fn start_online_campaign(&mut self) -> Result<(), SelectedLiveError> {
        // A reload (M8) or a rejoin starts over: no round, no shared world, the scripts held.
        self.end_campaign_ai_streams();
        if self.online.round.take().is_some()
            && let Some(input) = self.inner.bound_input.as_ref()
        {
            input.try_set_online_pause(false).map_err(error)?;
        }
        self.online.coop = coop_world::CoopWorld::default();
        self.online.campaign_hold = false;
        self.online.coop_dialog_opened = None;
        self.online.coop_running_since = None;
        if let Some(game) = self.online.game.as_mut() {
            game.entered = false;
        }
        let new_career = Self::profile_is_new()?;
        trace(format_args!("campaign lobby: the career starts as {}", if new_career { "New Career" } else { "Continue Career" }));
        if new_career {
            // EnterGame's flow message 28: the intro movies, then Create-a-Skater, then the career.
            self.begin_new_career_intro();
        } else {
            self.enter_single_player()?;
            self.career_start_pending = true;
        }
        Ok(())
    }

    /// Co-op fork (M8): the server asks every member to reload: the career goes behind the fade
    /// and starts again from the server's save (held in `campaign_adopt`), as a game joined
    /// from play does; the round restarts once every member has loaded.
    pub(super) fn online_campaign_reload(&mut self, session: u32) {
        if !self.online_campaign() {
            return;
        }
        crate::coop::trace(format_args!(
            "campaign: the server asks for a reload after round {session}: the career restarts from the server's save ({})",
            match self.online.campaign_adopt.as_ref() {
                Some(adopt) => format!("session {} tick {}, hash {:016x}", adopt.session, adopt.tick, adopt.hash),
                None => "none held: the local file".to_owned(),
            }
        ));
        self.online.start = Some(CAMPAIGN_START);
        self.frontend_fade.cover();
    }

    /// Co-op fork (M8): the connection was lost in a campaign. The career stays up but held
    /// (its scripts do not tick without the round), the game's state goes, and the next sign-in
    /// rejoins the campaign lobby, whose join brings the server's save and a reload for all.
    pub(super) fn online_campaign_dropped(&mut self, loss: LossReason) -> Result<(), SelectedLiveError> {
        crate::coop::trace(format_args!("campaign: the connection was lost; the career waits, signing in again to rejoin"));
        self.end_campaign_ai_streams();
        if self.online.round.take().is_some()
            && let Some(input) = self.inner.bound_input.as_ref()
        {
            input.try_set_online_pause(false).map_err(error)?;
        }
        self.online.campaign_hold = true;
        self.online.coop = coop_world::CoopWorld::default();
        self.online.game = None;
        self.online.chat = None;
        self.online.start = None;
        self.online.event_log.clear();
        self.online.notices.push(OnlineNotice::Disconnected(loss));
        self.online.campaign_rejoin = true;
        self.online.campaign_rejoin_at = Some(self.completed_frames + 60);
        Ok(())
    }

    /// Co-op fork: the career is up in a campaign lobby: the game counts as entered (the
    /// skater's frames flow, the other players' skaters show) and the recipes held meanwhile
    /// build.
    pub(in super::super) fn online_campaign_entered(&mut self) -> Result<(), SelectedLiveError> {
        if !self.online_campaign() {
            return Ok(());
        }
        trace(format_args!("campaign lobby: entered with the career"));
        if let Some(game) = self.online.game.as_mut() {
            game.entered = true;
        }
        // The career's scripts wait for the server's round start (M2): every console then
        // runs them from the same tick on the same facts.
        self.online.campaign_hold = true;
        self.online.send(ClientMessage::ChallengeLoaded);
        trace(format_args!("campaign lobby: career loaded, told the server; the scripts wait for the round's start"));
        self.online_build_held_recipes()
    }

    /// Co-op fork (M8): a rejoin whose sign-in failed (the server not back yet) tries again
    /// every three seconds until it is.
    pub(in super::super) fn campaign_rejoin_retry(&mut self) {
        if self.online.campaign_rejoin
            && self.online.client.is_none()
            && self.online.campaign_rejoin_at.is_some_and(|at| self.completed_frames >= at)
        {
            self.online.campaign_rejoin_at = None;
            crate::coop::trace(format_args!("campaign: signing in again to rejoin"));
            self.online_sign_in();
        }
    }

    /// Co-op fork (M8): back after a lost connection: the campaign lobby again.
    pub(in super::super) fn campaign_rejoin_signed_in(&mut self) {
        if std::mem::take(&mut self.online.campaign_rejoin) {
            let key = skate2_net::protocol::campaign::KEY;
            crate::coop::trace(format_args!("campaign: signed in again, rejoining the campaign lobby"));
            self.online_play_now(MatchRequest { mode: PlayNowMode::Campaign, ranked: false, challenge_type: challenge::CAMPAIGN, challenge_key: None, private: false, create: false, fallback_key: key });
        }
    }

    /// Co-op fork: `SKATE2_DIAGNOSTIC_ONLINE`'s `create:9` or `join:9` is the campaign lobby, at
    /// its one key: requested, and true; any other challenge type is not this one's.
    pub(in super::super) fn diagnostic_campaign_lobby(&mut self, challenge_type: u8, create: bool) -> bool {
        if challenge_type == challenge::CAMPAIGN {
            let key = skate2_net::protocol::campaign::KEY;
            trace(format_args!("diagnostic {} of the campaign lobby", if create { "create" } else { "join" }));
            self.online.diagnostic_requested = true;
            self.online_play_now(MatchRequest { mode: PlayNowMode::Campaign, ranked: false, challenge_type, challenge_key: create.then_some(key), private: false, create, fallback_key: key });
            return true;
        }
        false
    }

    /// Co-op fork (M8): the campaign's newest save as the server sent it, decoded and held for
    /// the career's next load.
    pub(in super::super) fn online_campaign_save(&mut self, slot: u8, session: u32, tick: u32, hash: u64, data: &[u8]) {
        match coop_campaign::progression_leaves(data) {
            Some(leaves) => {
                crate::coop::trace(format_args!(
                    "campaign: the server's save from slot {slot} at session {session} round tick {tick}: {} B, hash {hash:016x}, {} leaves, held for the career's load",
                    data.len(),
                    leaves.len()
                ));
                self.online.campaign_adopt = Some(AdoptedSave { session, tick, hash, leaves });
            }
            None => crate::coop::trace(format_args!("campaign: the server's save from slot {slot} ({} B, hash {hash:016x}) does not decode: ignored, the local file loads", data.len())),
        }
    }
}
