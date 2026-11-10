#!/bin/sh
# The co-op bench: one dedicated server and N game instances ("seats") on one machine, each
# seat with its own scratch save root, nickname and log, driven without a human through the
# port's diagnostic switches (docs/online/implementation.md, "Diagnostics").
#
#   tools/coop/bench.sh prime [N]    give N seats a scratch profile with the first mission done (Continue Career)
#   tools/coop/bench.sh start [N]    start the server and N seats (default 2)
#   tools/coop/bench.sh stop         stop everything the last start began
#   tools/coop/bench.sh logs         tail every log
#   tools/coop/bench.sh status       pids and the last line of each log
#
# Environment (all optional):
#   SKATE2_GAME_ROOT  the extracted game (default: ../Skate2 beside this repository)
#   GAME              the game binary (default: dist/macos/skate2-pc)
#   BENCH             the bench folder (default: ../bench beside this repository; never a save root a person plays from)
#   SERVER            host:port the server binds and the seats sign in to (default 127.0.0.1:32082)
#   SEAT_ONLINE_1..N  each seat's SKATE2_DIAGNOSTIC_ONLINE (default: seat 1 "create:8", others "join:8": an online free skate)
#   SEAT_EXTRA        extra environment for every seat, e.g. "SKATE2_TRACE_CAREER=1"
#   SEAT_EXTRA_1..N   the same for one seat. The switches a campaign seat takes (the career's
#                     own SKATE2_DIAGNOSTIC_CAREER_* run only under the career demo, not here):
#                       SKATE2_DIAGNOSTIC_COOP_TELEPORT=<tick>:<x>,<y>,<z>  the skater put there
#                         (786,-58,882 is the waterfront spawn, a street with pedestrians; the
#                         career's continue point has none)
#                       SKATE2_DIAGNOSTIC_COOP_SHOVE=<tick>  the nearest puppet pedestrian
#                         knocked down as a hit would, the contact sent to its owner (M1)
#   Every seat traces online, career and coop decisions (SKATE2_TRACE_*=1): never blind.
#   BENCH_VISIBLE     set to show the seats' windows; by default they open inactive and off-screen
#                     (SKATE2_DIAGNOSTIC_BACKGROUND_WINDOW), so the desktop stays yours
#   WINDOW_SIZE       each seat's window (default 960x540)
#   CAPTURE_EVERY     seconds between a seat's PNG captures, written by the game itself as
#                     seat-N/capture_<n>.png (default 30; CAPTURE_FROM 60, CAPTURE_COUNT 40).
#                     The seats run unbounded in the native frontend (a completion bound would
#                     skip the main menu, where the online diagnostic signs in); the fork lets
#                     --live-capture stand on CAPTURE_EVERY alone.
#
# A dev build (no release-secret.txt) only talks to a server run from the same executable, so
# the server is `$GAME --server`, never a separately built skate2-server.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
GAME=${GAME:-$here/dist/macos/skate2-pc}
BENCH=${BENCH:-$here/../bench}
SERVER=${SERVER:-127.0.0.1:32082}
SKATE2_GAME_ROOT=${SKATE2_GAME_ROOT:-$here/../Skate2}
export SKATE2_GAME_ROOT
command=${1:-start}
seats=${2:-2}

seat_dirs() { ls -d "$BENCH"/seat-* 2>/dev/null || true; }

# The window and capture environment every seat runs with.
window_env() {
    printf 'SKATE2_DIAGNOSTIC_WINDOW_SIZE=%s' "${WINDOW_SIZE:-960x540}"
    [ -n "${BENCH_VISIBLE:-}" ] || printf ' SKATE2_DIAGNOSTIC_BACKGROUND_WINDOW=1'
    printf ' SKATE2_DIAGNOSTIC_CAPTURE_EVERY=%s SKATE2_DIAGNOSTIC_CAPTURE_FROM=%s SKATE2_DIAGNOSTIC_CAPTURE_COUNT=%s' \
        "${CAPTURE_EVERY:-30}" "${CAPTURE_FROM:-60}" "${CAPTURE_COUNT:-40}"
}

# A seat's scratch profile: the career demo names the first profile, makes it through
# Create-a-Skater, answers "New To Skate?" and completes the first mission by diagnostic, whose autosave writes SKATER.P
# with the profile no longer new (SKATER.P sits at the save root). The seat then starts a campaign as Continue Career.
prime() {
    [ -x "$GAME" ] || { echo "no game binary at $GAME (build it first)"; exit 1; }
    i=1
    while [ "$i" -le "$seats" ]; do
        dir="$BENCH/seat-$i"
        mkdir -p "$dir"
        if [ -f "$dir/SKATER.P" ]; then
            echo "seat $i: already has SKATER.P"
        else
            echo "seat $i: priming a profile (log $dir/prime.log)"
            (cd "$dir" && env SKATE2_SAVE_ROOT="$dir" SKATE2_TRACE_CAREER=1 SKATE2_DIAGNOSTIC_CAREER_YES=1 \
                SKATE2_DIAGNOSTIC_PROFILE_NAME="Seat $i" SKATE2_DIAGNOSTIC_MOVIE_SKIP=120 SKATE2_DIAGNOSTIC_COMPLETE_CHALLENGES="${PRIME_TICK:-1200}:m_1a_01_goto_slappys" \
                SKATE2_DIAGNOSTIC_CAREER_QUIT_END="${PRIME_END:-6000}" $(window_env) \
                "$GAME" --live-skating --live-controls-demo career --live-completions 600000 --live-capture "$dir/prime.png" > prime.log 2>&1
             [ -f "$dir/SKATER.P" ] && echo "seat $i: primed" || echo "seat $i: no save was written (read $dir/prime.log)") &
        fi
        i=$((i + 1))
    done
    wait
}

start() {
    [ -x "$GAME" ] || { echo "no game binary at $GAME (build it first)"; exit 1; }
    [ -f "$SKATE2_GAME_ROOT/default.xex" ] || { echo "no game at $SKATE2_GAME_ROOT"; exit 1; }
    mkdir -p "$BENCH/server"
    echo "server: $SERVER (log $BENCH/server/server.log)"
    (cd "$BENCH/server" && exec "$GAME" --server "$SERVER" > server.log 2>&1) &
    echo $! > "$BENCH/server/pid"
    sleep 1
    i=1
    while [ "$i" -le "$seats" ]; do
        dir="$BENCH/seat-$i"
        mkdir -p "$dir"
        # The save root is the seat's folder: config.json, online-config.json, saves/ live here,
        # never in the player's own folder.
        printf '{\n  "nickname": "Seat %s",\n  "server": "%s",\n  "show_lobby_chat": true,\n  "show_ghost_players": true\n}\n' "$i" "$SERVER" > "$dir/online-config.json"
        eval "online=\${SEAT_ONLINE_$i:-}"
        if [ -z "$online" ]; then
            if [ "$i" -eq 1 ]; then online="create:8"; else online="join:8"; fi
        fi
        eval "extra=\"\${SEAT_EXTRA_$i:-}\""
        echo "seat $i: SKATE2_DIAGNOSTIC_ONLINE=$online ${SEAT_EXTRA:-} $extra (log $dir/seat.log)"
        # The seat's whole environment is kept beside its log (seat.env), so a switch that did
        # not take is never a guess.
        (cd "$dir" && env SKATE2_SAVE_ROOT="$dir" SKATE2_TRACE_ONLINE=1 SKATE2_TRACE_CAREER=1 SKATE2_TRACE_COOP=1 SKATE2_DIAGNOSTIC_ONLINE="$online" \
            SKATE2_DIAGNOSTIC_PROFILE_NAME="Seat $i" $(window_env) ${SEAT_EXTRA:-} $extra \
            sh -c 'env | grep "^SKATE2_" | sort > seat.env; exec "$0" "$@"' "$GAME" --live-skating --live-capture "$dir/capture.png" > seat.log 2>&1) &
        echo $! > "$dir/pid"
        # The creator must hold the lobby before a joiner searches.
        [ "$i" -eq 1 ] && sleep 8
        i=$((i + 1))
    done
}

stop() {
    for d in "$BENCH/server" $(seat_dirs); do
        [ -f "$d/pid" ] || continue
        pid=$(cat "$d/pid")
        kill "$pid" 2>/dev/null && echo "stopped $(basename "$d") ($pid)" || true
        rm -f "$d/pid"
    done
}

status() {
    for d in "$BENCH/server" $(seat_dirs); do
        [ -d "$d" ] || continue
        pid=$(cat "$d/pid" 2>/dev/null || echo -)
        alive=dead; [ "$pid" != - ] && kill -0 "$pid" 2>/dev/null && alive=alive
        log=$(ls "$d"/*.log 2>/dev/null | head -1)
        printf '%-8s pid %-6s %-5s %s\n' "$(basename "$d")" "$pid" "$alive" "$( [ -n "$log" ] && tail -1 "$log" )"
    done
}

case "$command" in
    prime) prime ;;
    start) start ;;
    stop) stop ;;
    status) status ;;
    logs) tail -n 30 "$BENCH"/server/server.log "$BENCH"/seat-*/seat.log ;;
    *) echo "usage: bench.sh prime [N] | start [N] | stop | status | logs"; exit 1 ;;
esac
