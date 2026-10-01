#!/bin/bash
# Drive the native omdurman game by real input (xdotool) and read it back
# through the app's own hex probe. Run from the repo root:
#   .claude/skills/run-omdurman/driver.sh <command> [args]
# State lives in $OMDURMAN_PLAY (default /tmp/omdurman-play): app.log, the
# probe files, and screenshots.
set -u
D=${OMDURMAN_PLAY:-/tmp/omdurman-play}
mkdir -p "$D"

win() { xdotool search --name '^omdurman$' 2>/dev/null | head -1; }
need_win() { W=$(win); [ -n "$W" ] || { echo "no omdurman window (run: launch)" >&2; exit 1; }; }
# Input goes to whatever window is on top: never send any while another
# window (the user's own work, a game...) is active.
need_input() {
  need_win
  local active; active=$(xdotool getactivewindow 2>/dev/null)
  if [ "$active" != "$W" ]; then
    echo "refusing input: active window is '$(xdotool getwindowname "$active" 2>/dev/null)', not omdurman" >&2
    exit 2
  fi
}
# Window pixel of hex (q,r) from the probe (window pixels == xdotool pixels).
hexpos() { awk -v q="$1" -v r="$2" '$1==q && $2==r {print $3, $4}' "$D/probe"; }

cmd=${1:-help}; shift || true
case "$cmd" in
launch)
  # launch [Lobby|Menu]  -- offline self-host, XWayland, probe armed.
  mode=${1:-Lobby}
  [ -n "$(win)" ] && { echo "already running"; exit 0; }
  rm -f "$D/probe" "$D/probe.state" "$D/probe.png"
  env -u WAYLAND_DISPLAY OMDURMAN_OFFLINE=1 OMDURMAN_START_MODE="$mode" \
    OMDURMAN_HEX_PROBE="$D/probe" OMDURMAN_PLAYER_SLOT=${OMDURMAN_PLAYER_SLOT:-7} \
    RUST_LOG=${RUST_LOG:-warn,omdurman=info} \
    setsid nohup cargo run -q -p omdurman-app >"$D/app.log" 2>&1 < /dev/null &
  for _ in $(seq 1 240); do [ -n "$(win)" ] && break; sleep 1; done
  need_win
  sleep 3
  echo "window $W: $(xdotool getwindowgeometry "$W" | grep Geometry)"
  ;;
shot)
  # shot <name> [WxH+X+Y]  -- in-app screenshot; whole window at half size,
  # or a full-resolution crop. Prints the path. Half-size: double coords to click.
  name=$1; crop=${2:-}
  rm -f "$D/probe.png"; touch "$D/probe.shot"
  for _ in $(seq 1 80); do [ -s "$D/probe.png" ] && break; sleep 0.25; done
  [ -s "$D/probe.png" ] || { echo "no screenshot (app not running?)" >&2; exit 1; }
  sleep 0.3
  if [ -n "$crop" ]; then magick "$D/probe.png" -crop "$crop" +repage "$D/$name.png"
  else magick "$D/probe.png" -resize 50% "$D/$name.png"; fi
  echo "$D/$name.png"
  ;;
click)
  # click <x> <y> [button]  -- window pixels (full resolution).
  need_input; xdotool mousemove --window "$W" "$1" "$2" sleep 0.15 click "${3:-1}"
  ;;
hex)
  # hex <q> <r> [button]  -- click a hex centre (selects / targets / places).
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  xdotool mousemove --window "$W" "$x" "$y" sleep 0.15 click "${3:-1}"
  ;;
dbl)
  # dbl <q> <r>  -- double-click: select the whole tile (combined fire/melee).
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  xdotool mousemove --window "$W" "$x" "$y" click --repeat 2 --delay 120 1
  ;;
hover)
  # hover <q> <r>  -- move the pointer over a hex (tooltips, LOS overlay).
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  xdotool mousemove --window "$W" "$x" "$y"
  ;;
zoom)
  # zoom <q> <r> <notches>  -- wheel-zoom the camera in (+) or out (-) at a hex.
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  n=$3; b=4; [ "$n" -lt 0 ] && { b=5; n=$((-n)); }
  xdotool mousemove --window "$W" "$x" "$y"
  for _ in $(seq 1 "$n"); do xdotool click $b; sleep 0.25; done
  sleep 1.5   # the probe refreshes twice a second
  ;;
key)
  # key <keysym>  -- e.g. e (End phase), Return (dismiss telegram), Escape.
  need_input; xdotool key --window "$W" "$1"
  ;;
state)
  # state  -- turn/phase line, then unit counts per side and disrupted.
  head -1 "$D/probe.state"
  awk '$1=="U"{n[$2]++; if($5=="true") d[$2]++} END{for(s in n) printf "%s: %d units, %d disrupted\n", s, n[s], d[s]+0}' "$D/probe.state"
  ;;
units)
  # units [grep-pattern]  -- "q,r disrupted id identity" per unit.
  grep '^U ' "$D/probe.state" | grep -E "${1:-.}" | awk '{$1=""; print}'
  ;;
wait)
  # wait <regex> [timeout_s]  -- until the turn/phase line matches.
  t=${2:-1800}; end=$(( $(date +%s) + t ))
  until head -1 "$D/probe.state" 2>/dev/null | grep -qE "$1"; do
    [ "$(date +%s)" -ge "$end" ] && { echo "timeout: $(head -1 "$D/probe.state")"; exit 1; }
    sleep 2
  done
  head -1 "$D/probe.state"
  ;;
ff)
  # ff <turn> <Dervish|AngloEgyptian>  -- fast-forward: End phase (E) on that
  # side's phases, wait out the AI's, until <turn> <side> Movement.
  need_input; target="turn=GameTurnIndex($1) phase=Movement active=$2"
  last=""; since=$(date +%s)
  while true; do
    s=$(head -1 "$D/probe.state")
    [ "$s" != "$last" ] && { echo "$(date +%T) $s"; last=$s; since=$(date +%s); }
    case "$s" in *"$target"*) exit 0;; esac
    if echo "$s" | grep -q "active=$2"; then
      [ $(( $(date +%s) - since )) -gt 25 ] && { echo "stuck: take a shot"; exit 1; }
      [ "$(xdotool getactivewindow 2>/dev/null)" = "$W" ] || { echo "refusing input: game window no longer active" >&2; exit 2; }
      xdotool key --window "$W" Return; sleep 0.5; xdotool key --window "$W" e; sleep 4
    else
      [ $(( $(date +%s) - since )) -gt 900 ] && { echo "AI stuck"; exit 1; }
      sleep 3
    fi
  done
  ;;
log)
  # log [pattern]  -- app log, ANSI stripped (e.g. log 'fire resolved').
  sed 's/\x1b\[[0-9;]*m//g' "$D/app.log" | grep -E "${1:-.}"
  ;;
stop)
  pgrep -x omdurman | xargs -r kill; sleep 1; pgrep -x omdurman >/dev/null && echo "still running" || echo stopped
  ;;
*)
  sed -n '2,8p' "$0"; grep -E '^  [a-z]+\)$' "$0" | tr -d ' )'
  ;;
esac
