#!/bin/bash
# A headless display for one game instance: a nested KWin on a virtual
# output (rendered on the GPU through GBM, never shown) with its own
# Xwayland, which has DRI3 -- so RADV can present, unlike on Xvfb. It runs
# under its own D-Bus session and config dir: nothing reaches the desktop's
# KWin or ~/.config.
#   virt.sh start <dir> [WxH]  -- prints the Xwayland DISPLAY (also in <dir>/display)
#   virt.sh stop <dir>
# The driver calls this for OMDURMAN_VIRTUAL=1 (or =WxH); see SKILL.md.
set -u
dir=$2/virt; mkdir -p "$dir/cfg"
case $1 in
start)
  size=${3:-1920x1080}; w=${size%x*}; h=${size#*x}
  sock=omd-virt-$$
  # KWin routes Xwayland's XTest through EIS: let xdotool in without a prompt.
  [ -e "$dir/cfg/kwinrc" ] || printf '[Xwayland]\nXwaylandEisNoPromptApps=xdotool\n' > "$dir/cfg/kwinrc"
  # Our display is the X socket that appears next (KWin's process cannot be
  # inspected to ask it): one start at a time, or parallel launches share one.
  exec 9>"${XDG_RUNTIME_DIR:-/tmp}/omdurman-virt.lock"; flock 9
  before=$(ls /tmp/.X11-unix/)
  env -u DISPLAY -u WAYLAND_DISPLAY XDG_CONFIG_HOME="$dir/cfg" setsid nohup dbus-run-session -- \
    kwin_wayland --virtual --xwayland --socket "$sock" --width "$w" --height "$h" \
    >"$dir/kwin.log" 2>&1 </dev/null 9>&- &
  echo $! > "$dir/pid"
  new=
  for _ in $(seq 1 40); do
    new=$(comm -13 <(echo "$before") <(ls /tmp/.X11-unix/) | head -1); [ -n "$new" ] && break; sleep 0.25
  done
  [ -n "$new" ] || { echo "no Xwayland appeared; see $dir/kwin.log" >&2; kill -- "-$(cat "$dir/pid")"; exit 1; }
  sleep 1
  # The virtual output comes up at scale 1.5; at 1 the window gets every pixel.
  XDG_CONFIG_HOME="$dir/cfg" WAYLAND_DISPLAY=$sock kscreen-doctor output.Virtual-0.scale.1 >/dev/null 2>&1
  echo ":${new#X}" | tee "$2/display"
  ;;
stop)
  # setsid made the dbus-run-session the group leader: the group is KWin,
  # its Xwayland and its bus.
  pid=$(cat "$dir/pid" 2>/dev/null) && kill -- "-$pid" 2>/dev/null
  # Until its X socket is gone: a start right after would otherwise get the
  # same display number back and never see a new socket appear.
  if display=$(cat "$2/display" 2>/dev/null); then
    for _ in $(seq 1 40); do [ -e "/tmp/.X11-unix/X${display#:}" ] || break; sleep 0.25; done
  fi
  rm -f "$2/display" "$dir/pid"
  ;;
esac
