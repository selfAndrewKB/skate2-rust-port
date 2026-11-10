# RE: what the bench found in the port (2026-11-10)

Findings about the port itself that the co-op bench turned up, each with where it was fixed or
left. Paths are repository-relative.

1. **The native front end needs no completion bound.** `apps/skate2-pc/src/live_skating_client.rs`
   starts the main menu only when `controls_demo.is_none() && !ollie_demo &&
   completion_limit.is_none()` (or under a menu/career demo); `--live-capture` required
   `--live-frames` or `--live-completions`. A seat that captured therefore sat forever in the
   direct-gameplay path without a main menu, and `SKATE2_DIAGNOSTIC_ONLINE` (gated on the menu
   being settled) never fired. Fixed: periodic captures stand on `SKATE2_DIAGNOSTIC_CAPTURE_EVERY`
   alone and write no completion capture; the diagnostic traces its wait.
2. **Captures of an occluded window.** `GpuRenderer::render_frame` acquires a swapchain image
   even for a capture; macOS answers `Occluded` for an off-screen window. Fixed: a capture of an
   occluded surface renders to an offscreen texture of the surface's shape and presents nothing.
3. **A hull reload stalls the game thread past the inactivity kick.** Continue Career unloads
   and reloads dynamic hulls (about 160 ticks on both seats); the server kicks a member silent
   for 2 s and the client's `Busy` timer cannot run during a synchronous stall. Fixed on the
   server for campaigns (15 s). Left: the stall itself.
4. **A joined online game switches the census off.** `update_network_pause` sets the game-info
   online flag for any joined game; `try_is_network_game()` reads it; `try_reset_game` derives
   `IsLWSpawnEnabled` from it; 28 sites in all take online behaviour from it. Fixed: a campaign
   keeps the offline flags.
5. **The career diagnostics run only under the career demo.** `SKATE2_DIAGNOSTIC_CAREER_PAD`,
   `_TELEPORT`, `_STICKS` and the rest live in the `controls_demo` branch of the input fill, so a
   seat on `--live-skating` never sees them (M0's "push" never happened). The fork's
   `SKATE2_DIAGNOSTIC_COOP_TELEPORT` and `_SHOVE` run on the online tick instead.
6. **No pedestrians at the career's start.** Offline, the census at the continue point
   ([1491.8, 109.7, -743.3]) spawned none in 1700 calls (the initial spawn's models were not
   resident; later attempts found nothing); the waterfront boot spawn fills 12 in 20 s. Left;
   the bench teleports there.
7. **Binaries overwritten in place are killed by macOS** (SIGKILL at launch after `cp` over a
   previously run executable). Fixed in `build-release-macos.sh`: remove, copy, ad-hoc sign.
