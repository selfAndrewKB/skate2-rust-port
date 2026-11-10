# The co-op bench and milestone M0

The bench is one dedicated server and N game instances ("seats") on one machine, each seat with
its own scratch save root, nickname and log, driven without a human through the port's
diagnostic switches. The script is `tools/coop/bench.sh`; its header lists every knob.

## Running it

```
./tools/coop/bench.sh prime 2      # once: a scratch profile per seat with the first mission done
SEAT_ONLINE_1=create:9 SEAT_ONLINE_2=join:9 ./tools/coop/bench.sh start 2
./tools/coop/bench.sh status       # pids and the last line of each log
./tools/coop/bench.sh stop
```

The bench folder (`../bench` beside the repository, never a save root a person plays from) holds
`server/server.log`, `seat-N/seat.log`, `seat-N/prime.log`, `seat-N/SKATER.P` and the seats'
captures `seat-N/capture_<nnn>.png` (BMP data under a .png name, as the port's `--live-capture`
writes them; `sips -s format png` converts one on macOS).

Seat windows open inactive and off-screen (`SKATE2_DIAGNOSTIC_BACKGROUND_WINDOW`), so the
desktop stays the person's; `BENCH_VISIBLE=1` shows them. Priming needs the career demo
(`--live-controls-demo career`) with `SKATE2_DIAGNOSTIC_MOVIE_SKIP`, a profile name, the
first mission completed by `SKATE2_DIAGNOSTIC_COMPLETE_CHALLENGES` and a quit by
`SKATE2_DIAGNOSTIC_CAREER_QUIT_END`; the autosave writes `SKATER.P` at the save root.

## What the campaign lobby is (M0)

A new network challenge type, `challenge::CAMPAIGN` (9), with one key
(`protocol::campaign::KEY`) and `PlayNowMode::Campaign`, so the server's matchmaking makes or
finds exactly one campaign lobby per key and never starts a challenge in it (the campaign has no
round; the server logs "a campaign takes no challenge start" if asked). Protocol version 115.

On the client, `SKATE2_DIAGNOSTIC_ONLINE=create:9` / `join:9` signs in from the main menu and
joins that lobby. `start_online_game` routes a campaign into `start_online_campaign`: Continue
Career when the profile has a career, else the New Career intro, with the online game attached.
`start_career` then marks the game entered (`online_campaign_entered`), which is what makes the
port send and show replay-frame puppets, and builds the characters already held from the lobby.
No gameplay is synchronized yet; this is the bench that every later milestone runs on.

## What M0 proved (2026-11-10, macOS, build from this branch)

Server log:

```
game 1 created: type 9 (campaign) challenge 636f6f702d63616d
Seat 1 joined game 1 in slot 0
Seat 2 joined game 1 in slot 1
2 players online, 1 games, in 6.0 KB/s (45 datagrams/s), out 6.0 KB/s (45 datagrams/s)
```

Each seat (`SKATE2_TRACE_ONLINE=1 SKATE2_TRACE_CAREER=1`):

```
online: diagnostic create:9: main menu gate passed
online: joined game 1 in slot 0: GameAttributes { challenge_type: 9, max_players: 4, ... }, LobbySkate
online: campaign lobby: the career starts as Continue Career
career: SignPost loaded (1 completed, 44 available)
career: continue at [1491.8145, 109.711006, -743.30505] (saved orientation true)
online: campaign lobby: entered with the career
online: sending our character (32566 bytes)
online: <other seat> shows
```

Captures show each seat in its own career with the minimap and the lobby's player list
("Challenge Lobby: Seat 1, Seat 2") and the other skater's puppet beside the local skater
(`m0-seat1.png` here is seat 1's view after seat 2 pushed off).

## What the bench taught (fixed on this branch)

- **No main menu under a completion bound.** The client starts the native front end only
  without `--live-completions`, and `--live-capture` demanded one, so a seat that captured never
  reached the menu where the online diagnostic signs in. Periodic captures
  (`SKATE2_DIAGNOSTIC_CAPTURE_EVERY`) now need no bound and write no completion capture; the
  diagnostic traces why it waits ("no main menu", "main menu not settled", "gate passed").
- **Captures of an occluded window.** macOS withholds the swapchain of an off-screen window
  ("Occluded"); a capture then renders to an offscreen texture and presents nothing.
- **Inactivity kicks on a streaming hitch.** Continuing a career unloads and reloads dynamic
  hulls on the game thread, a stall of about 160 ticks on both seats, over the server's 2 s
  silence limit, and the client's periodic `Busy` cannot be sent during a synchronous stall.
  A campaign member now has 15 s (`CAMPAIGN_INACTIVITY`), logged with the kick as before.
- **Same spot, same look.** Two primed saves continue at the same point, so the puppet stands
  inside the local skater; `SEAT_EXTRA_2="SKATE2_DIAGNOSTIC_CAREER_PAD=600-640"` pushes seat 2
  once so they come apart.
- The audio "buffer underrun" lines on every seat are the two instances sharing one output
  device and mean nothing for the bench.

## Seen and not yet explained

- The puppet flickers ("no longer shows" then "shows" on the next line) now and then on both
  seats, each time near a `controlled host backlog` line: a tick backlog starves the remote
  sample window. Harmless on the bench; it will matter for interpolation under M1.
- Some offscreen captures come out with the lower part black and the shading off ("original
  material shading pending" lines nearby): the capture caught a frame mid-reload of the
  hull's materials. The next capture is whole again.

## Not yet done

Nothing synchronizes: the world, the challenges, the saves and the cutscenes are each seat's
own. Next is M1, the ownership spike (`design.md` §5).
