#!/bin/bash
# Drive the native omdurman game by real input (xdotool) and read it back
# through the app's own hex probe. Run from the repo root:
#   .claude/skills/run-omdurman/driver.sh <command> [args]
# State lives in $OMDURMAN_PLAY (default /tmp/omdurman-play): app.log, the
# probe files, screenshots, and the instance's window id. Two instances run
# side by side with distinct OMDURMAN_PLAY dirs and OMDURMAN_PLAYER_SLOTs,
# online (OMDURMAN_ROOM=<room>) instead of offline.
set -u
D=${OMDURMAN_PLAY:-/tmp/omdurman-play}
mkdir -p "$D"

# This instance's window: the one `launch` recorded, while it still exists.
win() {
  local w; w=$(cat "$D/win" 2>/dev/null) || return 0
  [ "$(xdotool getwindowname "$w" 2>/dev/null)" = omdurman ] && echo "$w"
}
all_wins() { xdotool search --name '^omdurman$' 2>/dev/null | sort; }
need_win() { W=$(win); [ -n "$W" ] || { echo "no omdurman window (run: launch)" >&2; exit 1; }; }
# Input goes to whatever window is on top: never send any while another
# window (the user's own work, a game...) is active.
need_input() {
  need_win
  local active; active=$(xdotool getactivewindow 2>/dev/null)
  # Side-by-side instances: take the focus over from a sibling game window
  # (never from anything else).
  if [ "$active" != "$W" ] && [ "$(xdotool getwindowname "$active" 2>/dev/null)" = omdurman ]; then
    xdotool windowactivate --sync "$W" 2>/dev/null; sleep 0.2
    active=$(xdotool getactivewindow 2>/dev/null)
  fi
  if [ "$active" != "$W" ]; then
    echo "refusing input: active window is '$(xdotool getwindowname "$active" 2>/dev/null)', not omdurman" >&2
    exit 2
  fi
}
# Pointer: xdotool warps it where the compositor allows (KDE). COSMIC's (and
# GNOME's) XWayland ignores warps, so fall back to ydotool (uinput, needs
# ydotoold on $YDOTOOL_SOCKET): relative steps through pointer acceleration,
# corrected in a closed loop against the X pointer position.
export YDOTOOL_SOCKET=${YDOTOOL_SOCKET:-/run/user/$(id -u)/.ydotool_socket}
mouse() { xdotool getmouselocation | sed 's/x:\([0-9-]*\) y:\([0-9-]*\).*/\1 \2/'; }
# moveto <x> <y>  -- window pixels; exits 3 if the pointer cannot get there.
moveto() {
  local ox oy tx ty x y dx dy i
  eval "$(xdotool getwindowgeometry --shell "$W" | grep -E '^[XY]=')"; ox=$X; oy=$Y
  tx=$((ox + $1)); ty=$((oy + $2))
  # KWin honours warps but reads them in its own (scaled) units: the pointer
  # lands at request x scale. Warp, measure that scale (cached per state
  # dir), and re-warp at target / scale; within a pixel is on target.
  near() { local a=$(( $1 - $3 )) b=$(( $2 - $4 )); [ "${a#-}" -le 1 ] && [ "${b#-}" -le 1 ]; }
  local sx sy
  read -r sx sy 2>/dev/null < "$D/warp_scale" || { sx=1; sy=1; }
  xdotool mousemove "$(awk -v t=$tx -v s=$sx 'BEGIN{printf "%d", t/s+0.5}')" \
    "$(awk -v t=$ty -v s=$sy 'BEGIN{printf "%d", t/s+0.5}')"; sleep 0.05
  read -r x y < <(mouse); near "$x" "$y" "$tx" "$ty" && return 0
  if [ "$tx" -gt 100 ] && [ "$ty" -gt 100 ]; then
    xdotool mousemove "$tx" "$ty"; sleep 0.05; read -r x y < <(mouse)
    if [ "$x $y" != "$tx $ty" ] && [ "$x" -gt 0 ] && [ "$y" -gt 0 ]; then
      sx=$(awk -v a=$x -v b=$tx 'BEGIN{print a/b}'); sy=$(awk -v a=$y -v b=$ty 'BEGIN{print a/b}')
      echo "$sx $sy" > "$D/warp_scale"
      xdotool mousemove "$(awk -v t=$tx -v s=$sx 'BEGIN{printf "%d", t/s+0.5}')" \
        "$(awk -v t=$ty -v s=$sy 'BEGIN{printf "%d", t/s+0.5}')"; sleep 0.05
      read -r x y < <(mouse)
    fi
    near "$x" "$y" "$tx" "$ty" && return 0
  fi
  [ -S "$YDOTOOL_SOCKET" ] || { echo "pointer warp ignored by the compositor; start ydotoold (see SKILL.md)" >&2; exit 3; }
  for i in $(seq 1 80); do
    read -r x y < <(mouse)
    dx=$((tx - x)); dy=$((ty - y))
    [ "$dx" = 0 ] && [ "$dy" = 0 ] && return 0
    # A third of the error (acceleration amplifies a step ~2-3x), at least
    # 1; then wait for the X pointer position to catch up -- it lags the
    # uinput motion by a frame or more, and steering on a stale reading
    # overshoots.
    step() { local e=$1; [ "${1#-}" -gt 2 ] && e=$((e / 3)); echo "$e"; }
    ydotool mousemove -x "$(step $dx)" -y "$(step $dy)" >/dev/null
    local j nx ny
    for j in 1 2 3 4 5 6 7 8 9 10; do
      sleep 0.03; read -r nx ny < <(mouse)
      [ "$nx $ny" != "$x $y" ] && break
    done
  done
  # The X pointer position only updates over X windows: a native Wayland
  # window (a terminal, a notification) covering the target freezes it.
  echo "pointer stuck at $x,$y, wanted $tx,$ty: is a non-game window covering that part of the game?" >&2; exit 3
}
# click with button 1|2|3 (left|middle|right) at the current pointer.
press() {
  if [ -S "$YDOTOOL_SOCKET" ]; then
    case ${1:-1} in 1) ydotool click 0xC0;; 2) ydotool click 0xC2;; 3) ydotool click 0xC1;; esac >/dev/null
  else xdotool click "${1:-1}"; fi
}
# wheel <+n|-n>  -- n notches up (zoom in) or down.
wheel() {
  local n=$1 b=4 i; [ "$n" -lt 0 ] && { b=5; n=$((-n)); }
  for i in $(seq 1 "$n"); do
    if [ -S "$YDOTOOL_SOCKET" ]; then ydotool mousemove -w -x 0 -y $([ $b = 4 ] && echo 1 || echo -1) >/dev/null
    else xdotool click $b; fi
    sleep 0.25
  done
}

# Window pixel of hex (q,r) from the probe (window pixels == xdotool pixels).
hexpos() { awk -v q="$1" -v r="$2" '$1==q && $2==r {print $3, $4}' "$D/probe"; }

cmd=${1:-help}; shift || true
case "$cmd" in
launch)
  # launch [Lobby|Menu]  -- offline self-host, XWayland, probe armed.
  mode=${1:-Lobby}
  [ -n "$(win)" ] && { echo "already running"; exit 0; }
  rm -f "$D/probe" "$D/probe.state" "$D/probe.png" "$D/win"
  before=$(all_wins)
  # Offline self-host, unless OMDURMAN_ROOM names an online room.
  net=(OMDURMAN_OFFLINE=1); room=()
  [ -n "${OMDURMAN_ROOM:-}" ] && { net=(); room=(-- "$OMDURMAN_ROOM"); }
  env -u WAYLAND_DISPLAY "${net[@]}" OMDURMAN_START_MODE="$mode" \
    OMDURMAN_HEX_PROBE="$D/probe" OMDURMAN_PLAYER_SLOT=${OMDURMAN_PLAYER_SLOT:-7} \
    RUST_LOG=${RUST_LOG:-warn,omdurman=info,rodio=off,cpal=off} \
    setsid -w nohup cargo run -q -p omdurman-app "${room[@]}" >"$D/app.log" 2>&1 < /dev/null &
  pid=$!
  # A cold build takes many minutes: wait as long as cargo (then the game)
  # is alive, not a fixed time.
  until new=$(comm -13 <(echo "$before") <(all_wins) | head -1); [ -n "$new" ]; do
    kill -0 "$pid" 2>/dev/null || { echo "cargo run exited; see $D/app.log" >&2; tail -5 "$D/app.log" >&2; exit 1; }
    sleep 1
  done
  echo "$new" > "$D/win"
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
  need_input; moveto "$1" "$2"; sleep 0.15; press "${3:-1}"
  ;;
hex)
  # hex <q> <r> [button]  -- click a hex centre (selects / targets / places).
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  moveto "$x" "$y"; sleep 0.15; press "${3:-1}"
  ;;
dbl)
  # dbl <q> <r>  -- double-click: select the whole tile (combined fire/melee).
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  moveto "$x" "$y"; sleep 0.15; press 1; sleep 0.12; press 1
  ;;
drag)
  # drag <x1> <y1> <q|x2> <r|y2> [px]  -- press at window pixel (x1,y1), move,
  # release over hex (q,r), or over window pixel (x2,y2) with "px".
  need_input
  if [ "${5:-}" = px ]; then x=$3; y=$4; else read -r x y < <(hexpos "$3" "$4"); fi
  [ -n "${x:-}" ] || { echo "hex $3,$4 not on screen" >&2; exit 1; }
  moveto "$1" "$2"; sleep 0.15
  if [ -S "$YDOTOOL_SOCKET" ]; then ydotool click 0x40 >/dev/null; else xdotool mousedown 1; fi
  sleep 0.2; moveto $(( ($1 + x) / 2 )) $(( ($2 + y) / 2 )); sleep 0.1; moveto "$x" "$y"; sleep 0.2
  if [ -S "$YDOTOOL_SOCKET" ]; then ydotool click 0x80 >/dev/null; else xdotool mouseup 1; fi
  ;;
hover)
  # hover <q> <r>  -- move the pointer over a hex (tooltips, LOS overlay).
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  moveto "$x" "$y"
  ;;
zoom)
  # zoom <q> <r> <notches>  -- wheel-zoom the camera in (+) or out (-) at a hex.
  need_input; read -r x y < <(hexpos "$1" "$2")
  [ -n "${x:-}" ] || { echo "hex $1,$2 not on screen" >&2; exit 1; }
  moveto "$x" "$y"; wheel "$3"
  sleep 1.5   # the probe refreshes twice a second
  ;;
pan)
  # pan <Up|Down|Left|Right> [seconds]  -- hold an arrow key: the camera
  # pans (0.4 s at the default zoom moves ~700 px).
  need_input; xdotool keydown --window "$W" "$1"; sleep "${2:-0.2}"
  xdotool keyup --window "$W" "$1"; sleep 1
  ;;
scroll)
  # scroll <x> <y> <notches>  -- mouse wheel at a window pixel (the sidebar,
  # a panel): positive scrolls up, negative down.
  need_input; moveto "$1" "$2"; wheel "$3"; sleep 0.5
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
    case "$s" in *"$target"*) exit 0;; *game_over*) echo "game over"; exit 0;; esac
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
  # This instance only: the process owning its window.
  W=$(win); [ -n "$W" ] || { echo stopped; exit 0; }
  pid=$(xdotool getwindowpid "$W" 2>/dev/null)
  [ -n "$pid" ] && kill "$pid"; sleep 1
  [ -n "$(win)" ] && echo "still running" || echo stopped
  ;;
*)
  sed -n '2,8p' "$0"; grep -E '^  [a-z]+\)$' "$0" | tr -d ' )'
  ;;
esac
