# Environment switches

**Scope:** every `SKATE2_*` switch the game and the server read, generated from the code by
`tools/coop/switches.py --write` (do not edit the tables by hand); skate2-core's
`coop::switches` test fails when the code reads a switch this page does not list. What each
does is the comment that names it in the code. The co-op fork's are those upstream does not
read. 387 switches: 27 the fork's, 360 upstream's.

## The co-op fork's

| Switch | What | Read in |
| --- | --- | --- |
| `SKATE2_COOP_LOG` | `SKATE2_COOP_LOG=0` keeps them to stderr (the bench, whose seat | `crates/skate2-core/src/coop/coop.rs` |
| `SKATE2_COOP_RULES` | `SKATE2_COOP_RULES=<file>`; the built-in table when neither exists. A file that does not | `crates/skate2-core/src/coop/rules.rs` |
| `SKATE2_COOP_SYNC_DUMP` | `SKATE2_COOP_SYNC_DUMP=<ticks>`, every that many ticks) or a difference did; and | `crates/skate2-core/src/session/online/online_challenge.rs` |
| `SKATE2_DIAGNOSTIC_COOP_ANSWER` | `SKATE2_DIAGNOSTIC_COOP_ANSWER=yes\|no[:<ticks>]`: the progression's Yes/No dialog answered that | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_BAIL_ROUND` | `SKATE2_DIAGNOSTIC_COOP_BAIL_ROUND=<tick>[,<tick>...]` (M13): at each round tick the local | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_COMPLETE` | `SKATE2_DIAGNOSTIC_COOP_COMPLETE=<tick>[;<tick>...]`: at each round tick the first active | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_COMPLETE_CHALLENGE` | `SKATE2_DIAGNOSTIC_COOP_COMPLETE_CHALLENGE=<round tick>:<name>[;...]`: at each round tick the | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_EXECUTE` | `SKATE2_DIAGNOSTIC_COOP_EXECUTE=<tick>:<name>[;...]`: at each round tick the named challenge | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_LEAVE` | `SKATE2_DIAGNOSTIC_COOP_LEAVE=<round tick>`: this seat leaves the online game at that round | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_NUDGE` | `SKATE2_DIAGNOSTIC_COOP_NUDGE=<tick>[,<tick>...]`: at each tick the nearest prop with a | `crates/skate2-core/src/session/online/coop_world.rs` |
| `SKATE2_DIAGNOSTIC_COOP_NUDGE_ROUND` | `SKATE2_DIAGNOSTIC_COOP_NUDGE_ROUND` counts in round ticks instead, so two | `crates/skate2-core/src/session/online/coop_world.rs` |
| `SKATE2_DIAGNOSTIC_COOP_PRELOAD` | `SKATE2_DIAGNOSTIC_COOP_PRELOAD=<round tick>:<name>[;...]`: at each round tick the named | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_PROMPT` | `SKATE2_DIAGNOSTIC_COOP_PROMPT=<tick>:<answer>[;...]`: at that round tick this console answers | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_SHOVE` | `SKATE2_DIAGNOSTIC_COOP_SHOVE=<tick>[,<tick>...]`: at each tick a puppet is knocked down here | `crates/skate2-core/src/session/online/coop_world.rs` |
| `SKATE2_DIAGNOSTIC_COOP_TELEPORT` | `SKATE2_DIAGNOSTIC_COOP_TELEPORT=<tick>:<x>,<y>,<z>[;<tick>:<x>,<y>,<z>...]`: at each tick the | `crates/skate2-core/src/session/online/coop_world.rs` |
| `SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND` | `SKATE2_DIAGNOSTIC_COOP_TELEPORT_ROUND=<tick>:<x>,<y>,<z>[;...]`: as `SKATE2_DIAGNOSTIC_COOP_TELEPORT`, | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_TRIGGERS` | `SKATE2_DIAGNOSTIC_COOP_TRIGGERS=<tick>:<radius>`: at that round tick, the trigger volumes | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_WIN` | `SKATE2_DIAGNOSTIC_COOP_WIN=<round tick>\|running+<ticks>[;...]`: at each round tick named, | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_COOP_ZONES` | `SKATE2_DIAGNOSTIC_COOP_ZONES=<round tick>`: at that round tick the security guard areas | `crates/skate2-core/src/session/online/coop_campaign.rs` |
| `SKATE2_DIAGNOSTIC_DESYNC` | `SKATE2_DIAGNOSTIC_DESYNC=<tick>`, a planted divergence of this console's vars at the tick. | `crates/skate2-core/src/session/online/online_challenge.rs` |
| `SKATE2_DIAGNOSTIC_DUMP_LOOK` | M17's measure: the look's bytes to a file, to weigh its compression. | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_DIAGNOSTIC_MUTE` | `SKATE2_DIAGNOSTIC_MUTE`: the device gets silence, the mix runs as ever. | `apps/skate2-pc/src/game_audio.rs` |
| `SKATE2_DIAGNOSTIC_PUPPET_BLEND` | `SKATE2_DIAGNOSTIC_PUPPET_BLEND=linear` drives this seat's puppets | `crates/skate2-core/src/living_world/pedestrians/pedestrian_puppet.rs` |
| `SKATE2_DIAGNOSTIC_WINDOW_POSITION` | SKATE2_DIAGNOSTIC_WINDOW_POSITION=X,Y: a scripted run's logical window position (the co-op bench tiles its seats side by side when they are shown). | `apps/skate2-pc/src/render_window.rs` |
| `SKATE2_TRACE_COOP` | `SKATE2_TRACE_COOP=0` silences them. Each line goes to | `crates/skate2-core/src/coop/coop.rs` |
| `SKATE2_TRACE_COOP_CENSUS` | `SKATE2_TRACE_COOP_CENSUS=1`: every 60 round ticks, one line per pedestrian this console | `crates/skate2-core/src/session/online/coop_world.rs` |
| `SKATE2_TRACE_COOP_PUPPET` | `SKATE2_TRACE_COOP_PUPPET=1` traces a puppet's colliding state for the bench. | `crates/skate2-core/src/living_world/pedestrians/pedestrian_graph_colliding.rs` |

## Upstream's

| Switch | What | Read in |
| --- | --- | --- |
| `SKATE2_AUDIO_DUMP` | `SKATE2_AUDIO_DUMP`: instead of the device, the mix rendered in step with the simulation into | `apps/skate2-pc/src/game_audio.rs` |
| `SKATE2_AUDIO_WAV` | — | `apps/skate2-pc/src/game_audio.rs` |
| `SKATE2_AUDIT_GRIND_STREAMING` | — | `crates/skate2-core/src/physics/skate3_grinds.rs` |
| `SKATE2_AUDIT_GROUND_FEATURES` | — | `crates/skate2-core/src/physics/collision/static_ground_export.rs` |
| `SKATE2_BRIGHTNESS` | — | `apps/skate2-pc/src/gpu_renderer/final_pass.rs` |
| `SKATE2_BUILD_SECRET` | This executable's build (its build script's secret): its server and its client meet only the same build. | `apps/skate2-pc/src/main.rs`, `apps/skate2-server/src/main.rs` |
| `SKATE2_CAS_DB` | — | `crates/skate2-formats/tests/skate3_district_streams.rs` |
| `SKATE2_CAVEMAN_MOUNT` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_CAVEMAN_SPRINT` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_CENSUS_HOLDERS` | — | `apps/skate2-pc/src/render_material_census.rs` |
| `SKATE2_CENSUS_LABELS` | SKATE2_CENSUS_LABELS (effect names, comma separated): those effects' materials by name, with their Vault key and scalar channels. | `apps/skate2-pc/src/render_material_census.rs` |
| `SKATE2_CENSUS_TEXTURES` | — | `apps/skate2-pc/src/render_material_census.rs` |
| `SKATE2_CENSUS_TYPES` | — | `apps/skate2-pc/src/render_material_census.rs` |
| `SKATE2_CENSUS_WORLD` | `SKATE2_CENSUS_WORLD` names a downloadable content world of the packages in | `apps/skate2-pc/src/render_material_census.rs` |
| `SKATE2_CHARACTER_LIGHT_VIEW` | — | `apps/skate2-pc/src/gpu_renderer/character_stamp.rs` |
| `SKATE2_CUSTOM_SONG` | `SKATE2_CUSTOM_SONG` names the | `crates/skate2-audio/src/custom_song.rs` |
| `SKATE2_CUSTOM_SONG_PCM` | `SKATE2_CUSTOM_SONG_PCM` the file its 16-bit little-endian samples are written to. | `crates/skate2-audio/src/custom_song.rs` |
| `SKATE2_DEFAULT_SERVER` | `SKATE2_DEFAULT_SERVER=<host:port>` (a release's public server), else this machine's. The | `crates/skate2-core/src/ui/online_config.rs` |
| `SKATE2_DIAGNOSTIC_ACTIVATE_CHALLENGE` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_ACTIVATE_TICK` | SKATE2_DIAGNOSTIC_ACTIVATE_CHALLENGE=<title> at SKATE2_DIAGNOSTIC_ACTIVATE_TICK: the progression's ActivateChallengeByTitle. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_APPROACH_RB` | SKATE2_DIAGNOSTIC_APPROACH_RB: RB held on the approach (Skate 3's shove). | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_BACKGROUND_WINDOW` | SKATE2_DIAGNOSTIC_BACKGROUND_WINDOW: a scripted run's window opens inactive, left of every monitor, so it draws and captures without covering the desktop. | `apps/skate2-pc/src/render_window.rs` |
| `SKATE2_DIAGNOSTIC_BOARD_ROUTE` | SKATE2_DIAGNOSTIC_BOARD_ROUTE=<m/s>:<frame>:<height>:<x>,<z>[;<x>,<z>...]: from that frame the deck flies the looping route at that speed and height, over the city's collision, crossing the chunks as a speed glitch does. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_BUY_PROPERTY` | SKATE2_DIAGNOSTIC_BUY_PROPERTY=<tick>[;<tick>...]: from each tick the menus buy the Stuff menu's first Property (buy_property_step), one press every 40 ticks. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAPTURE_COUNT` | SKATE2_DIAGNOSTIC_CAPTURE_COUNT: the periodic captures stop after that many. | `apps/skate2-pc/src/render_window.rs` |
| `SKATE2_DIAGNOSTIC_CAPTURE_EVERY` | SKATE2_DIAGNOSTIC_CAPTURE_EVERY: besides the completion capture, one every given number of seconds, written beside it as <capture>_<n>.png. | `apps/skate2-pc/src/live_skating_client.rs`, `apps/skate2-pc/src/render_window.rs` |
| `SKATE2_DIAGNOSTIC_CAPTURE_FROM` | SKATE2_DIAGNOSTIC_CAPTURE_FROM: the periodic captures start that many seconds in. | `apps/skate2-pc/src/render_window.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_ANSWER` | A at the given tick: the "New to skate?" popup's default No. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_PAD` | SKATE2_DIAGNOSTIC_CAREER_PAD=<from>-<to>[:<hex buttons>][;...]: the buttons (A, a push, by default) held over those ticks. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_PAD_AFTER` | SKATE2_DIAGNOSTIC_CAREER_PAD_AFTER=<from>-<to>:<hex buttons>[;...]: the buttons held over those ticks. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_QUIT` | Quit the career from its pause menu at the given tick, then the main menu's free skate. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_QUIT_END` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_STICKS` | Raw stick input for repeatable bail/prop interaction inspection; ranges use the same career clock as PAD_AFTER and TRIGGERS. SKATE2_DIAGNOSTIC_CAREER_STICKS=from-to:lx,ly,rx,ry[;...]. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_TELEPORT` | Bounded diagnostic route through authored global-activity boxes. Each tick:x,y,z uses the same actor teleport as the existing race diagnostic; normal layer/trigger/activity updates perform the subsequent transitions. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_TRIGGERS` | SKATE2_DIAGNOSTIC_CAREER_TRIGGERS=<from>-<to>:<l\|r>[;...]: the left or right trigger pulled over those ticks. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAREER_YES` | SKATE2_DIAGNOSTIC_CAREER_YES: Yes on the popup, 90 ticks after it opens. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CASH` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CASH_TICK` | SKATE2_DIAGNOSTIC_CASH=<amount> at SKATE2_DIAGNOSTIC_CASH_TICK: the career's AddCash. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CAS_DUMP` | — | `apps/skate2-pc/src/edit_skater_room.rs` |
| `SKATE2_DIAGNOSTIC_CAS_HIDE` | SKATE2_DIAGNOSTIC_CAS_HIDE=<component>[:<effect>],...: those meshes are not drawn. Set at all, every part's meshes are listed with their effects. | `crates/skate2-core/src/rendering/character_material_admission.rs` |
| `SKATE2_DIAGNOSTIC_CAS_LIMBS` | `SKATE2_DIAGNOSTIC_CAS_LIMBS`): every arm and leg part, and the arm and leg kinds | `crates/skate2-core/src/session/edit_skater_record.rs` |
| `SKATE2_DIAGNOSTIC_CAS_TEXTURES` | `SKATE2_DIAGNOSTIC_CAS_TEXTURES`): a CAS material's maps, each with its format, size | `apps/skate2-pc/src/selected_character_material.rs` |
| `SKATE2_DIAGNOSTIC_CHALLENGE` | SKATE2_DIAGNOSTIC_CHALLENGE at SKATE2_DIAGNOSTIC_CHALLENGE_TICK: sign the named challenge up, then A on its poster. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CHALLENGE2` | SKATE2_DIAGNOSTIC_CHALLENGE2 at SKATE2_DIAGNOSTIC_CHALLENGE2_TICK: a second challenge signed up the same way, then A on its poster. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CHALLENGE2_TICK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CHALLENGE_SWEEP` | SKATE2_DIAGNOSTIC_CHALLENGE_SWEEP=a,b,...: from SKATE2_DIAGNOSTIC_CHALLENGE_SWEEP_TICK (default 3000), one challenge per SKATE2_DIAGNOSTIC_CHALLENGE_SWEEP_PERIOD ticks (default 1200): the running main challenge is abandoned, the next is sig | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CHALLENGE_SWEEP_PERIOD` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CHALLENGE_SWEEP_TICK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CHALLENGE_TICK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_CHANGELOG` | SKATE2_DIAGNOSTIC_CHANGELOG=<file>: that file's changes instead of this release's. | `apps/skate2-pc/src/game_update.rs` |
| `SKATE2_DIAGNOSTIC_COLOUR_FILTER` | SKATE2_DIAGNOSTIC_COLOUR_FILTER: what each change does to the colour filters. | `crates/skate2-core/src/session/edit_skater_colour_filter.rs`, `crates/skate2-core/src/session/edit_skater_session.rs` |
| `SKATE2_DIAGNOSTIC_COMPLETE_CHALLENGES` | SKATE2_DIAGNOSTIC_COMPLETE_CHALLENGES=<tick>:<name>[,<name>...][;...]: the signpost's CompleteChallenge of each named progression challenge at the tick. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_DARKCATCH` | SKATE2_DIAGNOSTIC_DARKCATCH=<from>:<period>[:<rb end>]: from that frame, every period frames, the corpus darkcatch (right stick down, a flick up and right, RB held to step 155 or the given step, e.g. through the landing). | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_DISPLAY_WALL` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_DISTRICT_PROXIES_SHOWN` | SKATE2_DIAGNOSTIC_DISTRICT_PROXIES_SHOWN: no high tile hides a proxy, so every district proxy stays drawn (a comparison for holes in the distance). | `apps/skate2-pc/src/city_streaming.rs` |
| `SKATE2_DIAGNOSTIC_DUMP_TRIGGERS` | SKATE2_DIAGNOSTIC_DUMP_TRIGGERS=<radius>: the trigger volumes around the local skater as an online challenge first unpins it. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_EDIT_SKATER_APPLY` | An apply script (SKATE2_DIAGNOSTIC_EDIT_SKATER_APPLY) may finish the editor. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_EDIT_SKATER_END` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_EDIT_SKATER_FEMALE` | SKATE2_DIAGNOSTIC_EDIT_SKATER_FEMALE (the plain demo): Body Mods, Gender and its acceptance make the skater female, as edit-skater-gender does, then B goes back to the main menu, and the path below starts from there at 1020. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_EDIT_SKATER_FIRST` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_EDIT_SKATER_PART` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_EDIT_SKATER_PATH` | SKATE2_DIAGNOSTIC_EDIT_SKATER_PATH browses until SKATE2_DIAGNOSTIC_EDIT_SKATER_END (default 1800). | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_EDIT_SKATER_PICK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FE_RIGHT_STICK` | SKATE2_DIAGNOSTIC_FE_RIGHT_STICK=<from>:<ticks>:<y>[;...]: from that front-end tick, the right stick's up at y (-1 to 1) for those ticks. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FILM_WIN` | Quit Challenge yes sends it, QuitChallengeCallback 0x8265e868). 60 ticks after the end-of-challenge options' screen is ready, the first one takes the given button (A, by default: Continue after a win, Retry after a loss; B: Retry after a wi | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FORCE_OBJECTIVES` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FORCE_OBJECTIVES_END` | SKATE2_DIAGNOSTIC_FORCE_OBJECTIVES: from the given tick, every 240 ticks the main challenge's active objectives are forced complete; A halfway between for popups. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FRAME_RATE_CAP` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREESKATE_AI_OFF` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREESKATE_AREA` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREESKATE_BACK` | Back instead: B at the screen, then the main menu after its NIS. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREESKATE_LOCATION` | `SKATE2_DIAGNOSTIC_FREESKATE_LOCATION`): whether its | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREESKATE_QUIT` | Quit from the free skate pause menu, then a second free skate with the defaults. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREESKATE_QUIT_TICK` | SKATE2_DIAGNOSTIC_FREESKATE_QUIT_TICK: from that tick the skater quits to the main menu through the pause menu (Start, Quit, Yes). | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREESKATE_SCREEN` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FREE_SKATE_HERE` | SKATE2_DIAGNOSTIC_FREE_SKATE_HERE=tick[;tick...]: at each tick, the free skate the online hub's Free Skate Here would ask for from where the skater is. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FRONTEND_BUTTONS` | SKATE2_DIAGNOSTIC_FRONTEND_BUTTONS=<tick>:<hex buttons>[;...]: these presses in place of the demo's own, until SKATE2_DIAGNOSTIC_FRONTEND_END ticks. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FRONTEND_END` | SKATE2_DIAGNOSTIC_FRONTEND_END ends these demos earlier too. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_FUZZ` | `SKATE2_DIAGNOSTIC_FUZZ=<seed>`). | `apps/skate2-pc/src/input_fuzz.rs` |
| `SKATE2_DIAGNOSTIC_FUZZ_FROM` | `SKATE2_DIAGNOSTIC_FUZZ_FROM=<tick>` holds the pad neutral until that | `apps/skate2-pc/src/input_fuzz.rs` |
| `SKATE2_DIAGNOSTIC_GESTURE_RB` | SKATE2_DIAGNOSTIC_GESTURE_RB=<from>:<period>[:<rb from>[:<rb to>[:<D-pad frames>]]]: from that frame, every period frames, D-pad up for 6 frames (or the given count: a held gesture), then RB from step 40 to 160 (or the given steps). | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_GRIP` | `SKATE2_DIAGNOSTIC_GRIP_TINT=<red>,<green>,<blue>`: every deck's griptape wears that tint (a | `crates/skate2-core/src/rendering/character_material_admission.rs` |
| `SKATE2_DIAGNOSTIC_GRIP_TINT` | `SKATE2_DIAGNOSTIC_GRIP_TINT=<red>,<green>,<blue>`: every deck's griptape wears that tint (a | `crates/skate2-core/src/rendering/character_material_admission.rs` |
| `SKATE2_DIAGNOSTIC_GROUND_EXPORT` | — | `crates/skate2-core/src/physics/skate3_player.rs` |
| `SKATE2_DIAGNOSTIC_HOLD` | SKATE2_DIAGNOSTIC_HOLD=<button>:<from>:<period>:<frames>[;...]: from that frame, every period frames, the button (a, b, x, y, lb, rb) held for the given frames. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_HOM_SCREEN` | Original-data UI inspection: open a HoM screen using the recorded bail. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_LAST_EYE` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_LEFT_STICK` | SKATE2_DIAGNOSTIC_LEFT_STICK=<from>:<to>:<x>:<y>: the left stick at those raw values from that frame until the other. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MAIN_MENU_SCRIPT` | SKATE2_DIAGNOSTIC_MAIN_MENU_SCRIPT=<steps>: from 360, one step each 30 ticks, logged with the Extras page: A, B, U, D, L, R, S (a capture), 0 (Up unless row 0 is selected), anything else a wait. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MAIN_MENU_TICK` | SKATE2_DIAGNOSTIC_MAIN_MENU_TICK: back to the main menu at that tick, as a lost online game goes. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MAP_PATH` | Scoped menu inspection through ordinary pad input: a comma-separated path of category/HAL strings, stopping on the info panel for capture/Teleport. A '*' part selects the current row where nothing else matches; SKATE2_DIAGNOSTIC_MAP_TELEPOR | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MAP_TELEPORT` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MAP_TICK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MISSING_PART` | SKATE2_DIAGNOSTIC_MISSING_PART=<slot>: that slot's model is one no archive holds, as a Skate 3 part is once the Skate 3 data is unlinked. | `crates/skate2-core/src/session/player_mounted_recipe_load.rs` |
| `SKATE2_DIAGNOSTIC_MISSION_MUSIC` | SKATE2_DIAGNOSTIC_MISSION_MUSIC=<tick>:<trigger>:<challenge>[;...]: the named challenge's ChallengeMusic.PlayMusic(trigger) at the tick. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MOVIE_SKIP` | SKATE2_DIAGNOSTIC_MOVIE_SKIP: A every given number of ticks while FE state 28 holds the front end, counted from when it opened. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MSAA` | `SKATE2_DIAGNOSTIC_MSAA=<frame>:<samples>[;...]`: from each frame on, that Anti-Aliasing (1, 2 | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MUSIC_PLAYER` | StartHub -> Settings -> the music player (or Audio Settings), then each screen's input. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_MUSIC_PLAYER_END` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_NET` | `SKATE2_DIAGNOSTIC_NET=<delay ms>:<jitter ms>:<loss %>`: each datagram either way waits the | `crates/skate2-net/src/client.rs` |
| `SKATE2_DIAGNOSTIC_NO_BOARD_GROUPS` | — | `crates/skate2-core/src/physics/modes/skate3/physics/riding_outputs.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE` | `SKATE2_DIAGNOSTIC_ONLINE` made its request. | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_ACTIVITY` | `SKATE2_DIAGNOSTIC_ONLINE_ACTIVITY=<ticks>[:<index>]`: after that many lobby ticks with | `crates/skate2-core/src/session/online/activity.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_CHAT` | `SKATE2_DIAGNOSTIC_ONLINE_CHAT` sent its line. | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_FRAME_DUMP` | `SKATE2_DIAGNOSTIC_ONLINE_FRAME_DUMP`: the local frames sent, written raw. | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_HUB_MAKE_HOST` | `SKATE2_DIAGNOSTIC_ONLINE_HUB_MAKE_HOST=<updates>`: once the host's game has shown every | `crates/skate2-core/src/session/online/online_hub.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_HUB_SPECTATE` | `SKATE2_DIAGNOSTIC_ONLINE_HUB_SPECTATE=<updates>` walks a host's or a guest's hub | `crates/skate2-core/src/session/online/online_hub.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_MAKE_HOST` | `SKATE2_DIAGNOSTIC_ONLINE_MAKE_HOST`: completed ticks in lobby skate before the host hands | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_NICKNAME` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_START` | `SKATE2_DIAGNOSTIC_ONLINE_START`: completed ticks in lobby skate before the host starts | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_DIAGNOSTIC_ONLINE_VOTE` | `SKATE2_DIAGNOSTIC_ONLINE_VOTE` voted on the turn the HUD shows. | `crates/skate2-core/src/session/online/online_challenge.rs` |
| `SKATE2_DIAGNOSTIC_OPTIONS_PAGE` | `SKATE2_DIAGNOSTIC_OPTIONS_PAGE`): whether its page took its right. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY` | Main menu row 3, three players join (the second picks the next skater), S.K.A.T.E., the first location (the snakes spot), then A to start. | `apps/skate2-pc/src/live_skating_client.rs`, `crates/skate2-core/src/session/selected_session_party_play.rs`, `crates/skate2-core/src/session/selected_session_party_skaters.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_BACK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_CAREER` | SKATE2_DIAGNOSTIC_PARTY_PLAY_CAREER keeps the first player on the first skater, the created one when the profile has it. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_CHANGE` | SKATE2_DIAGNOSTIC_PARTY_PLAY_CHANGE: X on the results opens Change Game, which steps to S.K.A.T.E. and its second location, goes back, opens again, returns to the running game and restarts it with A; `view` stays on the screen after the ste | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_DISTINCT` | SKATE2_DIAGNOSTIC_PARTY_PLAY_DISTINCT: the third player steps two skaters on, so the two later players load different skaters. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_GAME` | SKATE2_DIAGNOSTIC_PARTY_PLAY_GAME picks Spot Battle ("spot") or Hall of Meat ("hom") instead of S.K.A.T.E. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_LIST` | — | `crates/skate2-core/src/session/selected_session_party_play.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_LOCATION` | SKATE2_DIAGNOSTIC_PARTY_PLAY_LOCATION=<text>: right through the locations until the location's title ID holds the text, then the start. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_QUIT` | SKATE2_DIAGNOSTIC_PARTY_PLAY_QUIT: Start at the given tick, Quit and its Yes, then the main menu's free skate with its first skater, who pushes off; SKATE2_DIAGNOSTIC_PARTY_PLAY_QUIT_ROW=0 takes the career row instead. SKATE2_DIAGNOSTIC_PAR | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_QUIT_CHALLENGE` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_QUIT_PLAYER` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_QUIT_ROW` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_SETTINGS` | SKATE2_DIAGNOSTIC_PARTY_PLAY_SETTINGS=<tick>: from there Start, Game Settings, Audio Settings, SFX down to 80%, then B out of both pages, which saves the profile. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PARTY_PLAY_TWO` | SKATE2_DIAGNOSTIC_PARTY_PLAY_TWO: two players (no right on the player count, no third join), so a game of three rounds ends after six turns. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PHOTO_PICK` | SKATE2_DIAGNOSTIC_PHOTO_PICK: while a photo outro's photo selection is up, the right trigger 60 ticks after it opens, then Y (the pick) 60 ticks later. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PHOTO_PUSH` | SKATE2_DIAGNOSTIC_PHOTO_PUSH: from the teleport until the sequence completes, A held 12 ticks of every 30, so the replay has the skater rolling. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PHOTO_SEQUENCE` | SKATE2_DIAGNOSTIC_PHOTO_SEQUENCE: from the given tick, the local player's physics messages of m_1a_05's rail photo (as the headless test sends them): the sequence begins, the spot volume is entered, a 3 m grind of the rail, the sequence com | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PHOTO_SPOT` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PHYSICS` | `SKATE2_DIAGNOSTIC_PHYSICS=<physics>[:<difficulty>]` (Physics Settings labels, spaces | `crates/skate2-core/src/ui/gameplay_modes.rs` |
| `SKATE2_DIAGNOSTIC_PIN` | `SKATE2_DIAGNOSTIC_PIN=<from>:<to>`: from the `from`th tick to the `to`th the local skater is | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_DIAGNOSTIC_PINNED_GESTURE` | SKATE2_DIAGNOSTIC_PINNED_GESTURE=<period>: while an online challenge pins the local skater, D-pad up for one tick every period ticks. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_POSTER_MANUAL` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_POSTFX_OFF` | `SKATE2_DIAGNOSTIC_POSTFX_OFF`): the display settings with PostFX off. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PRESSES` | `SKATE2_DIAGNOSTIC_PRESSES=tick:button[,tick:button...]`, buttons up, down, left, | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PROFILE_NAME` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PROP_GRAB` | SKATE2_DIAGNOSTIC_PROP_GRAB=<frame>[:<hold>[:<gap>]]: at that frame, the local skater teleported off its board `gap` metres (default 0.8) short of the nearest prop, facing it; from 30 frames later RB held `hold` frames. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_PROP_LOCATIONS` | — | `crates/skate2-core/src/session/living_world/selected_living_world_dmo.rs` |
| `SKATE2_DIAGNOSTIC_PUSHING_FOLLOW` | — | `crates/skate2-core/src/physics/skate3_player.rs` |
| `SKATE2_DIAGNOSTIC_PUSHING_HAND_IK` | Skeleton::SetLeft/RightHandIKTargetPosition (0x82bd9728/0x82bd97d0): each hand's target within kDistComToBoard (0.65) of its shoulder, with the pushing blend. | `crates/skate2-core/src/physics/skate3_player.rs` |
| `SKATE2_DIAGNOSTIC_PUSH_OLLIE` | SKATE2_DIAGNOSTIC_PUSH_OLLIE=<from>:<period>[:<left stick>[:<left from>[:<crouch>]]]: from that frame, every period frames, the left stick held up (an easy-mode automatic push; 0 leaves it centred) from the given step (0) to 120, with the r | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_RACE` | ticks in (as the start hub's Quit Challenge yes, QuitChallengeCallback 0x8265e868) and lose leaves the race to the AI and the time limit. 60 ticks after the end-of-challenge options' screen is ready, the first one takes the given button (A  | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_RACE_GATES` | `SKATE2_DIAGNOSTIC_RACE_GATES=<ticks>[:<metres>]`: every so many completed | `crates/skate2-core/src/session/selected_session_career.rs` |
| `SKATE2_DIAGNOSTIC_RAIN` | — | `apps/skate2-pc/src/rain_lightning.rs`, `apps/skate2-pc/src/weather_audio.rs`, `apps/skate2-pc/src/weather_rain.rs` … |
| `SKATE2_DIAGNOSTIC_RAIN_FIELD_DUMP` | — | `apps/skate2-pc/src/weather_hydrology.rs` |
| `SKATE2_DIAGNOSTIC_RAIN_POOL` | — | `apps/skate2-pc/src/weather_rain.rs` |
| `SKATE2_DIAGNOSTIC_RAIN_SHADER` | — | `apps/skate2-pc/src/gpu_renderer/rain.rs` |
| `SKATE2_DIAGNOSTIC_READ_DELAY_MS` | `SKATE2_DIAGNOSTIC_READ_DELAY_MS`: every read | `crates/skate2-core/src/diagnostics/read_trace.rs` |
| `SKATE2_DIAGNOSTIC_REPLAY_ENTRY` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_REPLAY_MEDIA` | — | `apps/skate2-pc/src/replay_media_probe.rs` |
| `SKATE2_DIAGNOSTIC_RESOLUTION_SCALE` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_RIGHT_STICK` | SKATE2_DIAGNOSTIC_RIGHT_STICK=<from>:<period>:<start>:<end>:<x>:<y>[;<start>:<end>:<x>:<y>...]: from that frame, every period frames, the right stick held at those raw values from the start step until the end step (over SPIN_OLLIE's crouch  | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_S3_HEAD_FIT` | — | `crates/skate2-core/src/skate3/skate3_head_fit.rs` |
| `SKATE2_DIAGNOSTIC_S3_HELPER_PARENT` | `SKATE2_DIAGNOSTIC_S3_HELPER_PARENT` gives each the joint's parent side instead (the clavicle, | `crates/skate2-core/src/skate3/skate3.rs` |
| `SKATE2_DIAGNOSTIC_S3_NO_CLOTH` | SKATE2_DIAGNOSTIC_S3_NO_CLOTH: Skate 3's clothing skins as plain models, with no cloth. | `crates/skate2-core/src/skate3/skate3.rs` |
| `SKATE2_DIAGNOSTIC_S3_TATTOOS` | `SKATE2_DIAGNOSTIC_S3_TATTOOS=off` leaves them as they are; any other value logs | `crates/skate2-core/src/skate3/skate3_tattoos.rs` |
| `SKATE2_DIAGNOSTIC_SHOP` | SKATE2_DIAGNOSTIC_SHOP=<Slappy's row>: the editor shops instead of leaving at once; SKATE2_DIAGNOSTIC_SHOP_REOPEN=<tick> opens it again from the hub. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SHOP_EDIT_CART` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SHOP_POOL` | With Skate 3 data linked: SKATE2_DIAGNOSTIC_SHOP_POOL=3 picks the Skate 3 row. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SHOP_REOPEN` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SHOP_SPREE` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SHUV_BOUNCE` | `SKATE2_DIAGNOSTIC_SHUV_BOUNCE=on\|off`: Pre-patch Shuvit Bounce from boot, whatever the | `crates/skate2-core/src/ui/gameplay_modes.rs` |
| `SKATE2_DIAGNOSTIC_SIGNPOST_MESSAGE` | SKATE2_DIAGNOSTIC_SIGNPOST_MESSAGE=<tick>:<message>[;...]: the progression message sent at its tick, after the text setting SKATE2_DIAGNOSTIC_SIGNPOST_SETTING=<setting>=<text>. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SIGNPOST_SETTING` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SIGN_UP_PROMPT` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SIGN_UP_PROMPT_TICK` | SKATE2_DIAGNOSTIC_SIGN_UP_PROMPT=<challenge>,<challenge>[,...] at SKATE2_DIAGNOSTIC_SIGN_UP_PROMPT_TICK: the sign-up prompt for those challenges. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SKATE2_LIMBS` | SKATE2_DIAGNOSTIC_SKATE2_LIMBS: every garment takes the first that fits, Skate 2's first. | `crates/skate2-core/src/session/edit_skater_record.rs` |
| `SKATE2_DIAGNOSTIC_SKATE3_GRAFT` | SKATE2_DIAGNOSTIC_SKATE3_GRAFT: the online games' grafts onto a fresh challenge database, which report what joined. | `crates/skate2-core/src/session/selected_session_career.rs` |
| `SKATE2_DIAGNOSTIC_SKATE3_SPOT` | SKATE2_DIAGNOSTIC_SKATE3_SPOT=name[;name...] at SKATE2_DIAGNOSTIC_SKATE3_SPOT_TICK= tick[;tick...]: the challenge map's travel to a linked Skate 3 teleport. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SKATE3_SPOT_TICK` | SKATE2_DIAGNOSTIC_SKATE3_SPOT=name[;name...] at SKATE2_DIAGNOSTIC_SKATE3_SPOT_TICK= tick[;tick...]: the challenge map's travel to a linked Skate 3 teleport. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SKATE3_WATER_SCAN` | `SKATE2_DIAGNOSTIC_SKATE3_WATER_SCAN`): 300 ticks into the first district, the | `crates/skate2-core/src/session/selected_session_district_water.rs` |
| `SKATE2_DIAGNOSTIC_SKATE_ATTEMPT` | Scoped controller replay for career S.K.A.T.E. Each comma-separated value is the pop tick within one human attempt; the last value repeats. Only pad input changes: triggers, scoring and the original Lua decide whether the trick counts and w | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SKATE_STEER` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SKIN_MASK_OFF` | SKATE2_DIAGNOSTIC_SKIN_MASK_OFF: every skin takes its mask from its diffuse alpha, for comparison. | `apps/skate2-pc/src/selected_character_material.rs` |
| `SKATE2_DIAGNOSTIC_SKITCH` | SKATE2_DIAGNOSTIC_SKITCH=<frame>[:<hold>[:<gap>]]: at that frame, the local skater teleported on its board `gap` metres (default 0.6) behind the nearest vehicle's rear grab handle; from 30 frames later both hand grabs (the triggers) held `h | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SOLVER_FAULT` | — | `crates/skate2-core/src/physics/modes/skate3/physics/solve/diagnostics.rs` |
| `SKATE2_DIAGNOSTIC_SPAWN_LOCATION` | — | `crates/skate2-core/src/session/player.rs` |
| `SKATE2_DIAGNOSTIC_SPAWN_SKATER` | — | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_DIAGNOSTIC_SPAWN_SKATER_AT` | SKATE2_DIAGNOSTIC_SPAWN_SKATER_AT=dx,dz: the spawned skater's offset from the spawn. | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_DIAGNOSTIC_SPIN_OLLIE` | SKATE2_DIAGNOSTIC_SPIN_OLLIE=<from>:<period>:<swing>[:<release>[:<crouch>[:<pop>[:<x>]]]]: from that frame, every period frames, an ollie (the right stick down for the crouch frames (15) before the pop step (85), then up for 3 frames) with  | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_STICK` | SKATE2_DIAGNOSTIC_STICK=<from>:<frames>:<x>,<y>[;...]: the left stick at (x, y), each from -1 to 1 with y forward, for those frames. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SUBTITLES_OPTION` | SKATE2_DIAGNOSTIC_SUBTITLES_OPTION[=<tick>]: the main menu's Options (row 5), Video Settings (A at 600), its fifth row (Subtitles), right, Back twice (the profile save), then New Career at 900; a tick ends the demo there. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_SUBTITLE_SHOT` | SKATE2_DIAGNOSTIC_SUBTITLE_SHOT=<n>[,<ms>]: the career demo completes, and captures, the given milliseconds (500 by default) after the n-th front-end subtitle line comes up, shown or not. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TELEPORT_HERE` | SKATE2_DIAGNOSTIC_TELEPORT_HERE=<frame>[:on\|off[:<yaw>]][;...]: at that frame, the local skater teleported where it is, on its board or off it: a respawn while still moving. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TELEPORT_TO` | SKATE2_DIAGNOSTIC_TELEPORT_TO=<frame>:on\|off:<yaw>:<x>,<y>,<z>[;...]: at that frame, the local skater teleported to that point facing yaw radians, on its board or off it. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TIMER_HZ` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TRAJECTORY_MAX_TIME` | SKATE2_DIAGNOSTIC_TRAJECTORY_MAX_TIME: a shorter landing query, so ordinary launches take the no-landing air. | `crates/skate2-core/src/physics/modes/skate3/physics/air_trajectory/mod.rs` |
| `SKATE2_DIAGNOSTIC_TRAVEL` | SKATE2_DIAGNOSTIC_TRAVEL=x,y,z,yaw[;x,y,z,yaw...] at SKATE2_DIAGNOSTIC_TRAVEL_TICK= tick[;tick...]: the map's travel to each place at its tick, facing yaw radians from +z. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TRAVEL_TICK` | SKATE2_DIAGNOSTIC_TRAVEL=x,y,z,yaw[;x,y,z,yaw...] at SKATE2_DIAGNOSTIC_TRAVEL_TICK= tick[;tick...]: the map's travel to each place at its tick, facing yaw radians from +z. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TRICK_CYCLE` | SKATE2_DIAGNOSTIC_TRICK_CYCLE=<tick>[:<push ticks>:<flick tick>]: from that completed tick, every five seconds a push, then a kickflip flick (right stick down for 30 ticks, then up-left) at the flick tick. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TRIGGER_TAPS` | SKATE2_DIAGNOSTIC_TRIGGER_TAPS=<from>:<period>:<frames>[:<l\|r\|b>[:<count>]][;...]: from that frame, every period frames, the right trigger (l the left, b both) pulled fully for the given frames, `count` times (without end by default). | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TURN_BAIL` | a volume (a trigger's local id, hex) that many ticks after the skater enters it. The flicks, one per turn in turn and `/` apart, are right-stick paths of compass points such as `dr-d-ul`: the first held up to 30 ticks, the others 2, the las | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_TURN_BAIL_LIMIT` | — | `crates/skate2-core/src/physics/skate3_player.rs` |
| `SKATE2_DIAGNOSTIC_TURN_TRICK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_VBR_PAIR` | `SKATE2_DIAGNOSTIC_VBR_PAIR=linear` interpolates every pair without the block rule (A/B tests). | `crates/skate2-core/src/physics/modes/skate3/animation_pose.rs`, `crates/skate2-formats/src/andale.rs` |
| `SKATE2_DIAGNOSTIC_VSYNC` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_WEATHER` | `SKATE2_DIAGNOSTIC_WEATHER=<weather>[@<tick>]` (a Weather Settings label, spaces optional): | `crates/skate2-core/src/ui/gameplay_modes.rs` |
| `SKATE2_DIAGNOSTIC_WINDOW_SIZE` | SKATE2_DIAGNOSTIC_WINDOW_SIZE=WxH: a scripted run's logical window size. | `apps/skate2-pc/src/render_window.rs` |
| `SKATE2_DIAGNOSTIC_WORLD_TRAVEL` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DIAGNOSTIC_WORLD_TRAVEL_TICK` | SKATE2_DIAGNOSTIC_WORLD_TRAVEL=<key hex>[:<location>] at SKATE2_DIAGNOSTIC_WORLD_TRAVEL_TICK: a travel into that world location. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_DLC_DIR` | `SKATE2_DLC_DIR` names a folder of them: every root file reads. | `apps/skate2-pc/src/render_environment.rs`, `apps/skate2-pc/src/render_material_census.rs`, `crates/skate2-formats/src/ice_camera.rs` … |
| `SKATE2_FFMPEG` | — | `apps/skate2-pc/src/media_encoder.rs` |
| `SKATE2_FLIP_FAMILY_RELEASE_STEP` | — | `crates/skate2-core/src/session/selected_session_public_facade_corpus.rs` |
| `SKATE2_FOOTPLANT_STICK` | Diagnostic raw pad: push off, then hold LT (grab) with the left stick held. Footplant: the push route's kerb launches the board at frame ~254; A held in that air extends a foot, which plants on landing. Boneless: A tapped while grabbing on  | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_FRAME_BONES` | — | `crates/skate2-net/examples/frame_lab.rs` |
| `SKATE2_FRAME_FIELDS` | `SKATE2_FRAME_FIELDS=1`. | `crates/skate2-net/examples/frame_stats.rs` |
| `SKATE2_FRAME_PROFILE` | — | `apps/skate2-pc/src/frame_profile.rs`, `apps/skate2-pc/src/gpu_renderer.rs`, `crates/skate2-core/src/diagnostics/frame_profile.rs` |
| `SKATE2_GAME_ROOT` | The borrowed entity skins the scene's pose buffer with its own inverse bind pose, bone for bone (EvalCurrAnim 0x82866efc..0x82866f38). | `apps/skate2-pc/src/game_audio.rs`, `apps/skate2-pc/src/game_root.rs`, `apps/skate2-pc/src/gpu_renderer/character_tests.rs` … |
| `SKATE2_GPU_PROFILE` | — | `apps/skate2-pc/src/gpu_renderer.rs` |
| `SKATE2_HIPPY_SPIN` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_HIPPY_STICK` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_HIPPY_WIND_UP` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_KICKFLIP_HOLD_END_STEP` | — | `crates/skate2-core/src/session/selected_session_public_facade_corpus.rs` |
| `SKATE2_KICKTURN_STICK` | Diagnostic raw pad: standing on the board, hold the stick sideways for 18 s (kickturns in place), release for 1 s, then hold it again for 10 s. SKATE2_KICKTURN_STICK is the stick's X (default full left). | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_LEAN_TRACE` | — | `crates/skate2-core/src/diagnostics/lean_trace.rs` |
| `SKATE2_LOG_ELAPSED` | `SKATE2_LOG_ELAPSED`: each diagnostic line begins with the seconds since [`start`]. | `crates/skate2-core/src/diagnostics/diagnostics.rs` |
| `SKATE2_MAP_FILE` | — | `crates/skate2-symbols/src/main.rs` |
| `SKATE2_MAP_THREADS` | `SKATE2_MAP_THREADS`). | `apps/skate2-pc/src/gpu_renderer.rs` |
| `SKATE2_MATERIAL_AUDIT` | — | `apps/skate2-pc/src/render_inspection.rs` |
| `SKATE2_MOTION_GRAPH_PATH` | — | `apps/skate2-pc/src/motion_graph_typed_corpus.rs` |
| `SKATE2_MOTION_GRAPH_TEST_PATH` | — | `apps/skate2-pc/src/motion_graph_test_probe.rs` |
| `SKATE2_NATIVE_PRESENT_CLOCK` | `SKATE2_NATIVE_PRESENT_CLOCK` restores with `RealTimeController` instead. | `crates/skate2-core/src/rendering/presentation_restore.rs` |
| `SKATE2_NEUTRAL_CUBE` | — | `apps/skate2-pc/src/render_postfx_math.rs` |
| `SKATE2_NO_MSAA` | `SKATE2_NO_MSAA`: one sample, whatever Display Settings says. | `apps/skate2-pc/src/gpu_renderer.rs` |
| `SKATE2_NO_PRESENTATION_RESTORE` | `SKATE2_NO_PRESENTATION_RESTORE`: draw each tick as it completes instead, for | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_NO_VSYNC` | `SKATE2_NO_VSYNC`: Unlimited, whatever Video Settings say. | `apps/skate2-pc/src/gpu_renderer.rs` |
| `SKATE2_POOL_GROUND` | `SKATE2_POOL_GROUND`. Without it the build carries none and those pools collect | `apps/skate2-pc/build.rs` |
| `SKATE2_POSTFX_VIEW` | — | `apps/skate2-pc/src/gpu_renderer/postfx.rs` |
| `SKATE2_PRESENT_TRACE` | `SKATE2_PRESENT_TRACE`: each SIM-thread tick's due and consumption times and the end of | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_PROFILE_COLLISION` | — | `crates/skate2-core/src/physics/collision/collision_line_profile.rs` |
| `SKATE2_PROFILE_REDRAW` | — | `apps/skate2-pc/src/live_skating_client.rs`, `apps/skate2-pc/src/render_window.rs`, `crates/skate2-core/src/camera/camera_collision_query.rs` … |
| `SKATE2_REDRAW_WORLD_SHADOWS` | — | `apps/skate2-pc/src/gpu_renderer/shadow_map.rs` |
| `SKATE2_RELEASE_SECRET` | `SKATE2_RELEASE_SECRET` (64 hexadecimal digits, | `apps/build_secret.rs` |
| `SKATE2_RESTORE_TRACE` | `SKATE2_RESTORE_TRACE`: each restored frame's start, the moment it shows (in ticks) | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_SAVE_ROOT` | `SKATE2_SAVE_ROOT` relocates it with the saves; otherwise it sits in the | `crates/skate2-core/src/coop/rules.rs`, `crates/skate2-core/src/session/edit_skater_profile.rs`, `crates/skate2-core/src/ui/controller_config.rs` … |
| `SKATE2_SEPARATE_ISLAND_DRAWS` | — | `apps/skate2-pc/src/gpu_renderer/map_draw.rs` |
| `SKATE2_SEPARATE_MESH_BUFFERS` | — | `apps/skate2-pc/src/gpu_renderer/mesh_arena.rs` |
| `SKATE2_SERIAL_SIM` | `SKATE2_SERIAL_SIM` or a lean trace keeps every tick on this thread. | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_SHADOW_RECEIVER_VIEW` | Opt-in receiver diagnosis, folded out of normal shaders. Raw view separates atlas/depth failures from native normal-map shadow gating. | `apps/skate2-pc/src/gpu_renderer/environment_shading.rs` |
| `SKATE2_TEST_GRIND_PREFIX` | Explicit corpus schedule selection, not a native query-miss substitute. Grinds were parked for the first rendered test. Opt in to reproduce the unfinished native prefix and its documented deck-W refusal. | `crates/skate2-core/src/physics/player/lifecycle/physical_player_update1_corpus.rs` |
| `SKATE2_TICK_TRACE` | `SKATE2_TICK_TRACE`: each consumed tick's due, start, finish and consumption times in | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_TRACE` | A launch with a trace asked for (any SKATE2_TRACE_* variable) writes its output, the player's own launch included. | `apps/skate2-pc/src/main.rs` |
| `SKATE2_TRACE_ADJUSTED_POSE` | — | `crates/skate2-core/src/animation/trees/actor_animation_nblend.rs`, `crates/skate2-core/src/rendering/skater/skater_presentation_write.rs`, `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_ADJUSTED_POSE_FROM` | — | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_ADJUSTED_POSE_THROUGH` | — | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_ADJUSTED_POSE_VELOCITY` | — | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_AEMS_OPS` | — | `crates/skate2-audio/src/aems/interpret.rs` |
| `SKATE2_TRACE_AI` | `SKATE2_TRACE_AI`). | `crates/skate2-core/src/ai/path_controller.rs`, `crates/skate2-core/src/ai/skater_manager.rs`, `crates/skate2-core/src/challenge/ai.rs` … |
| `SKATE2_TRACE_AIR_CAMERA` | — | `crates/skate2-core/src/camera/graph/camera_man_trajectory.rs` |
| `SKATE2_TRACE_AI_SPIN` | — | `crates/skate2-core/src/ai/path_controller.rs` |
| `SKATE2_TRACE_AMBIENCE` | — | `crates/skate2-audio/src/world/environment.rs` |
| `SKATE2_TRACE_AUDIO` | `SKATE2_TRACE_AUDIO` is set (a diagnostic log of what | `crates/skate2-audio/src/splicer.rs` |
| `SKATE2_TRACE_AWAY_TIME` | `SKATE2_TRACE_AWAY_TIME`: the player-input flag each time it changes and `mAwayTime` each | `crates/skate2-audio/src/system.rs` |
| `SKATE2_TRACE_BOUNDARY` | `SKATE2_TRACE_BOUNDARY`: the skater's physics state and category, `mIsOffboard`, and the roots | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_CAMERA_RESTORE` | — | `crates/skate2-core/src/rendering/presentation_restore_camera.rs` |
| `SKATE2_TRACE_CAREER` | `SKATE2_TRACE_CAREER` the names the | `crates/skate2-core/src/challenge/ai.rs`, `crates/skate2-core/src/challenge/hom.rs`, `crates/skate2-core/src/challenge/presentation.rs` … |
| `SKATE2_TRACE_CAREER_LUA` | — | `crates/skate2-core/src/challenge/system.rs` |
| `SKATE2_TRACE_CAREER_NATIVES` | `SKATE2_TRACE_CAREER_NATIVES` logs every native a script calls. | `crates/skate2-core/src/challenge/lua/vm.rs` |
| `SKATE2_TRACE_CAS_FETCH` | — | `crates/skate2-core/src/ui/edit_skater_presentation.rs` |
| `SKATE2_TRACE_CITY_STREAM` | — | `apps/skate2-pc/src/city_streaming.rs`, `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_TRACE_CITY_TEXTURES` | — | `apps/skate2-pc/src/main.rs` |
| `SKATE2_TRACE_DECK_ACCELERATION` | — | `crates/skate2-core/src/physics/player/ground/physical_player_ground_wipeout_leaves.rs` |
| `SKATE2_TRACE_DISTRICT_PROXY` | SKATE2_TRACE_DISTRICT_PROXY: each proxy's hull on the ground, to set against the high tiles that hide it. | `apps/skate2-pc/src/city_streaming.rs`, `apps/skate2-pc/src/proxy_world.rs` |
| `SKATE2_TRACE_DRAWN_SKATER` | — | `apps/skate2-pc/src/live_skating_view.rs` |
| `SKATE2_TRACE_EDIT_SKATER_LAYOUT` | — | `apps/skate2-pc/src/marker_ui/edit_skater_movie.rs` |
| `SKATE2_TRACE_FRONTEND_CAMERA` | — | `apps/skate2-pc/src/live_skating_view.rs` |
| `SKATE2_TRACE_FRONTEND_FADE` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_TRACE_GROUND_RECORDS` | — | `crates/skate2-core/src/physics/board/runtime/skateboard_ground_frame.rs` |
| `SKATE2_TRACE_HAND_TOKENS` | — | `crates/skate2-core/src/animation/hands_face/actor_animation_hands_face_dispatch.rs`, `crates/skate2-core/src/animation/runtime/skate3_gesture_streams.rs` |
| `SKATE2_TRACE_HOM` | — | `crates/skate2-core/src/physics/player/output/conditioning/scoring/wipeout_skate3.rs` |
| `SKATE2_TRACE_HULLS` | — | `apps/skate2-pc/src/hull_presentation.rs` |
| `SKATE2_TRACE_KNOWN_AIR` | — | `crates/skate2-core/src/animation/channels/actor_animation_transition_eval.rs`, `crates/skate2-core/src/animation/channels/actor_animation_transition_time.rs`, `crates/skate2-core/src/animation/graphs/motion/callbacks/actor_motion_manual_angle.rs` … |
| `SKATE2_TRACE_LETTERS` | — | `apps/skate2-pc/src/marker_ui.rs` |
| `SKATE2_TRACE_LIVING_WORLD_COUNTS` | `SKATE2_TRACE_LIVING_WORLD_COUNTS=<updates>`: every that many `Update1`s, the living world's | `crates/skate2-core/src/session/living_world/selected_living_world_dmo.rs` |
| `SKATE2_TRACE_MARKER_UI` | — | `apps/skate2-pc/src/marker_ui.rs`, `crates/skate2-core/src/session/selected_session_props.rs` |
| `SKATE2_TRACE_MAX_OUT` | `SKATE2_TRACE_MAX_OUT` logs the challenges left incomplete. | `crates/skate2-core/src/challenge/dynamic_hull.rs`, `crates/skate2-core/src/challenge/signpost.rs`, `crates/skate2-core/src/session/edit_skater_shop.rs` |
| `SKATE2_TRACE_MOTION_ACTIVE_SET` | — | `crates/skate2-core/src/animation/graphs/motion/generated_motion_runtime_lifecycle.rs` |
| `SKATE2_TRACE_MOVIE` | — | `apps/skate2-pc/src/movie_playback.rs` |
| `SKATE2_TRACE_OFFBOARD_AIR` | — | `crates/skate2-core/src/animation/graphs/motion/callbacks/actor_motion_air_leg.rs`, `crates/skate2-core/src/animation/graphs/motion/callbacks/actor_motion_landing_offboard_air.rs`, `crates/skate2-core/src/animation/graphs/motion/generated_motion_runtime.rs` … |
| `SKATE2_TRACE_OFFBOARD_TURN` | — | `crates/skate2-core/src/physics/player/offboard/physical_player_offboard_feet_ik_session.rs` |
| `SKATE2_TRACE_OFFSTAGE` | `SKATE2_TRACE_OFFSTAGE` traces. | `apps/skate2-pc/src/marker_ui.rs` |
| `SKATE2_TRACE_ONLINE` | — | `crates/skate2-core/src/session/selected_session_online.rs` |
| `SKATE2_TRACE_ONLINE_INPUTS` | `SKATE2_TRACE_ONLINE_INPUTS`: a running hash of the ticks' frames. | `crates/skate2-core/src/session/online/online_challenge.rs` |
| `SKATE2_TRACE_PAD_POLL` | `SKATE2_TRACE_PAD_POLL=1`: every 600 ticks, how long after their timer instants the ticks | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_TRACE_PAINTER` | — | `crates/skate2-core/src/living_world/population/living_world_census_targets.rs`, `crates/skate2-core/src/world/world_painter_layer.rs` |
| `SKATE2_TRACE_PARTICLES` | — | `apps/skate2-pc/src/particle_view.rs` |
| `SKATE2_TRACE_PEDESTRIANS` | `SKATE2_TRACE_PEDESTRIANS`: RecipeMan's state, and the loads delivered after they were due. | `apps/skate2-pc/src/live_skating_view.rs`, `apps/skate2-pc/src/pedestrian_render.rs`, `crates/skate2-core/src/animation/graphs/pedestrian_motion_pose.rs` … |
| `SKATE2_TRACE_PEDESTRIAN_HEIGHT` | — | `crates/skate2-core/src/session/living_world/selected_living_world_pedestrian_target.rs` |
| `SKATE2_TRACE_PHOTOGRAPHER` | — | `crates/skate2-core/src/living_world/pedestrians/pedestrian_graph_marquee.rs` |
| `SKATE2_TRACE_PHYSICS_RECOVERY` | — | `crates/skate2-core/src/physics/player/board/physical_player_board_post_outer_live.rs` |
| `SKATE2_TRACE_POSTFX` | — | `apps/skate2-pc/src/gpu_renderer/postfx.rs` |
| `SKATE2_TRACE_PRESENTATION_ROOT` | — | `crates/skate2-core/src/session/selected_session_frame.rs` |
| `SKATE2_TRACE_PRESENTATION_SIZE` | — | `crates/skate2-core/src/rendering/skater/skater_presentation_animation.rs` |
| `SKATE2_TRACE_PUSHING_MOVE` | — | `crates/skate2-core/src/physics/player/board/physical_player_pushing_move.rs`, `crates/skate2-core/src/physics/player/interaction/physical_player_grab_world.rs`, `crates/skate2-core/src/physics/solver/bodies/simulation_rigid_body_grab_force.rs` |
| `SKATE2_TRACE_PUSH_FORCE` | — | `crates/skate2-core/src/physics/board/forces/skateboard_push_acceleration.rs`, `crates/skate2-core/src/physics/player/ground/physical_player_ground_queue_torque_prefix.rs`, `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_PUSH_INTERRUPTION` | — | `crates/skate2-core/src/animation/graphs/motion/callbacks/actor_motion_break_out_of_push.rs`, `crates/skate2-core/src/animation/graphs/motion/callbacks/actor_motion_runtime_bump_conditions.rs`, `crates/skate2-core/src/animation/graphs/motion/generated_motion_runtime.rs` … |
| `SKATE2_TRACE_RAIN_TIMING` | `SKATE2_TRACE_RAIN_TIMING`: the streak build's drop updates, ordering and vertices, averaged | `apps/skate2-pc/src/gpu_renderer/rain.rs`, `apps/skate2-pc/src/weather_rain.rs` |
| `SKATE2_TRACE_READS` | `SKATE2_TRACE_READS`: every read on | `crates/skate2-core/src/diagnostics/read_trace.rs` |
| `SKATE2_TRACE_RECKONING_NAN` | — | `crates/skate2-core/src/physics/reckoning/reckoning.rs` |
| `SKATE2_TRACE_REPLAY_AUDIO` | — | `apps/skate2-pc/src/game_audio.rs`, `crates/skate2-audio/src/system.rs` |
| `SKATE2_TRACE_RUMBLE` | — | `crates/skate2-core/src/input/input_rumble_manager.rs` |
| `SKATE2_TRACE_SCORE_SEQUENCE` | `SKATE2_TRACE_SCORE_SEQUENCE`: collector changes and sequence endings. | `crates/skate2-core/src/challenge/mission.rs`, `crates/skate2-core/src/challenge/objectives.rs`, `crates/skate2-core/src/physics/player/output/conditioning/scoring/module_finish.rs` |
| `SKATE2_TRACE_SCORING` | — | `crates/skate2-core/src/physics/player/output/conditioning/scoring/conditioner.rs`, `crates/skate2-core/src/physics/player/output/conditioning/scoring/wipeout_skate3.rs`, `crates/skate2-core/src/physics/player/output/conditioning/scoring/wipeout_tweaks.rs` |
| `SKATE2_TRACE_SECURITY_CAPTURE` | — | `crates/skate2-audio/src/music/mod.rs`, `crates/skate2-audio/src/system.rs`, `crates/skate2-core/src/living_world/pedestrians/pedestrian_graph_frame_chase.rs` … |
| `SKATE2_TRACE_SELECTED_STATES` | `SKATE2_TRACE_SELECTED_STATES`): every load is delivered at its due update and every stream | `apps/skate2-pc/src/live_skating_client.rs`, `crates/skate2-core/src/animation/locomotion/actor_animation_locomotion_params.rs`, `crates/skate2-core/src/diagnostics/diagnostics.rs` |
| `SKATE2_TRACE_SESSION_MARKER` | — | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_SHOVE` | `SKATE2_TRACE_SHOVE`: the running channels, GrabWorld and the busy hands, once per change. | `crates/skate2-core/src/living_world/pedestrians/pedestrian_collision_shove.rs`, `crates/skate2-core/src/physics/modes/skate3/graph_host/motion_shove.rs`, `crates/skate2-core/src/physics/modes/skate3/physics/shove_probe.rs` |
| `SKATE2_TRACE_SKATE2_VISUAL_ROOT` | — | `crates/skate2-core/src/physics/skeleton/pose/skeleton_root_transforms.rs` |
| `SKATE2_TRACE_SKATE3` | `SKATE2_TRACE_SKATE3=1`): the Skate 3 player's stages and | `crates/skate2-core/src/diagnostics/test_log.rs` |
| `SKATE2_TRACE_SKATE3_ACTION_EXITS` | SKATE2_TRACE_SKATE3_ACTION_EXITS=<text>: while the ActionGraph's state path holds the text, each transition out of it and its ancestors, with its conditions' activations. | `crates/skate2-core/src/physics/modes/skate3/skater_animation.rs` |
| `SKATE2_TRACE_SKATE3_AIR` | — | `crates/skate2-core/src/physics/modes/skate3/physics/air_phase.rs` |
| `SKATE2_TRACE_SKATE3_BREADCRUMBS` | — | `crates/skate2-core/src/physics/modes/skate3/physics/respawn.rs` |
| `SKATE2_TRACE_SKATE3_CLIP` | `SKATE2_TRACE_SKATE3_CLIP=<name part>`: each sampled clip whose name contains it, with its time, | `crates/skate2-core/src/physics/modes/skate3/animation_pose.rs` |
| `SKATE2_TRACE_SKATE3_COAST` | `SKATE2_TRACE_SKATE3_COAST=1`: once a second on the ground, the speed, the no-input timer and | `crates/skate2-core/src/physics/modes/skate3/physics/ground_runtime/input.rs` |
| `SKATE2_TRACE_SKATE3_DARKCATCH` | `SKATE2_TRACE_SKATE3_DARKCATCH`: each change of the raw scorable with its Skate 3 audio record. | `crates/skate2-audio/src/global_effects.rs`, `crates/skate2-audio/src/player/contacts.rs`, `crates/skate2-audio/src/player/offboard.rs` … |
| `SKATE2_TRACE_SKATE3_DROP_IN` | `SKATE2_TRACE_SKATE3_DROP_IN`: each drop-in sound as it starts. | `crates/skate2-audio/src/player/drop_in.rs` |
| `SKATE2_TRACE_SKATE3_DROP_IN_TICKS` | `SKATE2_TRACE_SKATE3_DROP_IN_TICKS`: one `dropin3` line per tick: the physical state, the | `crates/skate2-core/src/physics/modes/skate3/physics/drop_in_trace.rs` |
| `SKATE2_TRACE_SKATE3_EDGES` | `SKATE2_TRACE_SKATE3_EDGES=<tick>:<radius>`: at that physics tick, one `edge3` line per grind | `crates/skate2-core/src/physics/modes/skate3/physics/drop_in_trace.rs` |
| `SKATE2_TRACE_SKATE3_FOOT_FORCE` | — | `crates/skate2-core/src/physics/modes/skate3/physics/ground_animation/board.rs`, `crates/skate2-core/src/physics/modes/skate3/physics/ground_runtime/input.rs` |
| `SKATE2_TRACE_SKATE3_GRAB` | SKATE2_TRACE_SKATE3_GRAB: candidates before and after the obstruction lines, with each line's hit (fraction, assembly) and the candidates' assemblies. | `crates/skate2-core/src/physics/modes/skate3/physics/biped_ground/grab.rs`, `crates/skate2-core/src/physics/modes/skate3/physics/biped_ground/grab_runtime.rs` |
| `SKATE2_TRACE_SKATE3_GRABS` | SKATE2_TRACE_SKATE3_GRABS: each enabled object's grab handles, their world midpoints and approach directions. | `crates/skate2-core/src/physics/skate3_grabs.rs` |
| `SKATE2_TRACE_SKATE3_GRAB_FRAME` | — | `crates/skate2-core/src/physics/modes/skate3/physics/biped_ground/grab.rs`, `crates/skate2-core/src/physics/modes/skate3/physics/biped_ground/services.rs` |
| `SKATE2_TRACE_SKATE3_GRAB_OBJECT` | — | `crates/skate2-core/src/physics/skate3_grabs.rs` |
| `SKATE2_TRACE_SKATE3_GRIND` | — | `crates/skate2-core/src/physics/player/output/physical_player_phys_out_skate3.rs` |
| `SKATE2_TRACE_SKATE3_GRIND_GRAB` | SKATE2_TRACE_SKATE3_GRIND_GRAB: each tick in a grind, the motion state, the grab and crouch intents, the height the crouched-enough test reads and the hands' heights. | `crates/skate2-core/src/physics/modes/skate3/skater_animation.rs` |
| `SKATE2_TRACE_SKATE3_HANDS` | The face keeps its S2 owner: S3 face cues index S3's own face palette. | `crates/skate2-core/src/animation/hands_face/actor_animation_hands_face_runtime.rs`, `crates/skate2-core/src/animation/runtime/skate3_gesture_streams.rs`, `crates/skate2-core/src/physics/modes/skate3/graph_host/motion_execute.rs` … |
| `SKATE2_TRACE_SKATE3_LAND` | `SKATE2_TRACE_SKATE3_LAND` adds a `land3` line per tick for touchdown comparisons: the deck | `crates/skate2-core/src/physics/modes/skate3/physics/spin_trace.rs` |
| `SKATE2_TRACE_SKATE3_LAUNCH` | — | `crates/skate2-core/src/physics/modes/skate3/physics/air_trajectory/mod.rs` |
| `SKATE2_TRACE_SKATE3_MANUAL_TURN` | — | `crates/skate2-core/src/physics/modes/skate3/physics/ground_runtime/input.rs` |
| `SKATE2_TRACE_SKATE3_PARTS` | `SKATE2_TRACE_SKATE3_PARTS` adds a `parts3` line per tick: each board body's solved linear and | `crates/skate2-core/src/physics/modes/skate3/physics/spin_trace.rs` |
| `SKATE2_TRACE_SKATE3_PUSH_COEFS` | — | `crates/skate2-core/src/physics/modes/skate3/graph_host/pushing.rs` |
| `SKATE2_TRACE_SKATE3_PUSH_TICKS` | — | `crates/skate2-core/src/physics/modes/skate3/physics/frame.rs` |
| `SKATE2_TRACE_SKATE3_RIDER_CONTACTS` | — | `crates/skate2-core/src/physics/modes/skate3/physics/solve/assembly_contacts.rs` |
| `SKATE2_TRACE_SKATE3_SKITCH` | — | `crates/skate2-core/src/physics/modes/skate3/physics/skitching.rs` |
| `SKATE2_TRACE_SKATE3_SPIN` | `SKATE2_TRACE_SKATE3_SPIN`: one stderr line per physics tick with everything that sets the | `crates/skate2-core/src/physics/modes/skate3/physics/spin_trace.rs` |
| `SKATE2_TRACE_SKATE3_STEER` | `SKATE2_TRACE_SKATE3_STEER`: each ground tick's steering inputs, tilt, truck targets and the | `crates/skate2-core/src/physics/modes/skate3/physics/ground_runtime/update.rs` |
| `SKATE2_TRACE_SKATE3_VISUAL_ROOT` | — | `crates/skate2-core/src/physics/skate3_player.rs` |
| `SKATE2_TRACE_SKATE3_WATER` | — | `crates/skate2-core/src/session/selected_session_district_water.rs` |
| `SKATE2_TRACE_SKATE3_WIPEOUT` | `SKATE2_TRACE_SKATE3_WIPEOUT`: each check that requests a wipeout, with its reasons and the | `crates/skate2-core/src/physics/modes/skate3/physics/riding_outputs.rs`, `crates/skate2-core/src/physics/modes/skate3/physics/wipeout.rs` |
| `SKATE2_TRACE_SPEECH` | `SKATE2_TRACE_SPEECH`: the game's speech requests and the audio system's speech messages. | `apps/skate2-pc/src/game_audio.rs` |
| `SKATE2_TRACE_SPEED` | — | `crates/skate2-core/src/animation/graphs/motion/callbacks/motion_crouching.rs`, `crates/skate2-core/src/animation/graphs/motion/callbacks/motion_set_turning.rs`, `crates/skate2-core/src/physics/board/control/skateboard_speed_wobble_calculate.rs` |
| `SKATE2_TRACE_SUBTITLES` | — | `apps/skate2-pc/src/live_skating_client.rs`, `apps/skate2-pc/src/marker_ui/subtitle_movie.rs` |
| `SKATE2_TRACE_TOE_TARGETS` | — | `crates/skate2-core/src/physics/assembly/skeleton/dynamic_assembly_skeleton_drive_update.rs`, `crates/skate2-core/src/physics/skeleton/collision/skeleton_collision_update.rs` |
| `SKATE2_TRACE_TRAJECTORY` | A native hit payload is copied even if its computed time is negative. | `crates/skate2-core/src/physics/player/prediction/physical_player_selector_update.rs`, `crates/skate2-core/src/physics/solver/simulation_world_trajectory_query.rs` |
| `SKATE2_TRACE_TRAVEL` | `SKATE2_TRACE_TRAVEL`: what the map travel last waited for. | `crates/skate2-core/src/session/selected_session_public_facade.rs` |
| `SKATE2_TRACE_VEHICLES` | — | `crates/skate2-core/src/living_world/population/living_world_census_vehicle.rs` |
| `SKATE2_TRACE_VEHICLE_SKATER` | SKATE2_TRACE_VEHICLE_SKATER: each car the skater touches, and whether it stops. | `crates/skate2-core/src/living_world/vehicles/living_world_vehicle_frame.rs` |
| `SKATE2_TRACE_VERT` | — | `crates/skate2-core/src/animation/graphs/motion/callbacks/motion_crouching.rs`, `crates/skate2-core/src/animation/graphs/motion/generated_motion_runtime.rs`, `crates/skate2-core/src/animation/runtime/actor_animation_gesture_writes.rs` … |
| `SKATE2_TRACE_VISUAL_INDICATORS` | — | `apps/skate2-pc/src/visual_indicator_view.rs` |
| `SKATE2_TRACE_WAKE` | — | `apps/skate2-pc/src/live_skating_client.rs` |
| `SKATE2_TRACE_WHEEL_GROUND` | — | `crates/skate2-core/src/physics/collision/simulation_assembly_ground_job.rs`, `crates/skate2-core/src/physics/collision/simulation_ground_contact.rs` |
| `SKATE2_TRACE_XINPUT` | — | `apps/skate2-pc/src/sdl_pad_platform.rs`, `apps/skate2-pc/src/windows_pad_platform.rs` |
| `SKATE2_UPDATE_PUBLIC_KEY` | — | `apps/skate2-pc/src/game_update.rs`, `apps/skate2-server/src/main.rs` |
| `SKATE2_UPDATE_REPOSITORY` | `SKATE2_UPDATE_REPOSITORY`. | `apps/skate2-pc/src/game_update.rs`, `apps/skate2-server/src/main.rs` |
| `SKATE2_VEHICLE_MATERIAL_TRACE` | — | `apps/skate2-pc/src/vehicle_render.rs` |
| `SKATE2_VEHICLE_SHADE_DIAGNOSTIC` | — | `apps/skate2-pc/src/gpu_renderer/dynamic_object.rs` |
| `SKATE2_WGPU_LOG` | `SKATE2_WGPU_LOG`: wgpu's warnings and errors on stderr, with `WGPU_VALIDATION=1` also the | `apps/skate2-pc/src/gpu_log.rs` |
| `SKATE2_WORLD_CASTER_CULL` | — | `apps/skate2-pc/src/gpu_renderer/map_draw.rs` |
| `SKATE2_Y_FOLLOWUP_FRAMES` | Optional original-owner continuation after the first completed OffBoard frame. Zero preserves the original boundary proof. | `crates/skate2-core/src/physics/player/runtime/physical_player_three_frame_corpus.rs` |

## Notes

(Hand-written notes on switches go here; the regeneration keeps them.)
