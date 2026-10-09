#!/usr/bin/env bash
# Drive the gnome-boxes Debian VM out-of-band through libvirt (qemu:///session):
# screenshots via `virsh screenshot`, pointer via QMP `input-send-event` on the
# absolute vmmouse, keys via `virsh send-key`. Nothing here touches the host
# desktop, the Boxes window, or the pointer -- the user keeps working.
#
#   vm.sh shot NAME [WxH+X+Y]   -> $VM_PLAY/NAME.png (full guest resolution; optional crop)
#   vm.sh move X Y              guest pixels
#   vm.sh click X Y [right|middle]
#   vm.sh dbl X Y
#   vm.sh drag X1 Y1 X2 Y2
#   vm.sh wheel X Y N           N notches, negative = down
#   vm.sh key KEY...            linux key names without KEY_, pressed together: ctrl l / Return
#   vm.sh type TEXT             ASCII text (US layout in the guest)
#   vm.sh size                  guest screen size
set -euo pipefail
VM=${VM_NAME:-debian13-liv}
URI=${VM_URI:-qemu:///session}
PLAY=${VM_PLAY:-/tmp/omdurman-vm}
mkdir -p "$PLAY"
v() { virsh -q -c "$URI" "$@"; }
qmp() { v qemu-monitor-command "$VM" "$1" >/dev/null; }

size() {
  if [ ! -s "$PLAY/size" ]; then
    v screenshot "$VM" "$PLAY/_size.ppm" >/dev/null
    magick identify -format '%w %h\n' "$PLAY/_size.ppm" > "$PLAY/size"
  fi
  cat "$PLAY/size"
}

shot() {
  local name=$1 crop=${2:-}
  v screenshot "$VM" "$PLAY/$name.ppm" >/dev/null
  if [ -n "$crop" ]; then magick "$PLAY/$name.ppm" -crop "$crop" +repage "$PLAY/$name.png"
  else magick "$PLAY/$name.ppm" "$PLAY/$name.png"; fi
  rm -f "$PLAY/$name.ppm"
  echo "$PLAY/$name.png"
}

abs() { # x y -> QMP abs events (0..32767 across the screen)
  read -r w h < <(size)
  local ax=$(( $1 * 32767 / (w - 1) )) ay=$(( $2 * 32767 / (h - 1) ))
  printf '{"type":"abs","data":{"axis":"x","value":%d}},{"type":"abs","data":{"axis":"y","value":%d}}' "$ax" "$ay"
}
btn() { printf '{"type":"btn","data":{"down":%s,"button":"%s"}}' "$1" "$2"; }
send() { qmp "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[$1]}}"; }

move() { send "$(abs "$1" "$2")"; }
press() { local b=${3:-left}; send "$(abs "$1" "$2"),$(btn true "$b")"; sleep 0.06; send "$(btn false "$b")"; }
click() { move "$1" "$2"; sleep 0.08; press "$1" "$2" "${3:-left}"; }
dbl() { click "$1" "$2"; sleep 0.08; press "$1" "$2"; }
drag() {
  move "$1" "$2"; sleep 0.1; send "$(btn true left)"; sleep 0.1
  local mx=$(( ($1 + $3) / 2 )) my=$(( ($2 + $4) / 2 ))
  move "$mx" "$my"; sleep 0.1; move "$3" "$4"; sleep 0.15; send "$(btn false left)"
}
wheel() {
  move "$1" "$2"; sleep 0.08
  local n=$3 b=wheel-up; [ "$n" -lt 0 ] && { n=$(( -n )); b=wheel-down; }
  for ((i = 0; i < n; i++)); do send "$(btn true $b)"; send "$(btn false $b)"; sleep 0.05; done
}

keyname() { # friendly -> KEY_*
  case $1 in
    Return|enter) echo KEY_ENTER ;; Escape|esc) echo KEY_ESC ;; space) echo KEY_SPACE ;;
    ctrl) echo KEY_LEFTCTRL ;; alt) echo KEY_LEFTALT ;; shift) echo KEY_LEFTSHIFT ;; super) echo KEY_LEFTMETA ;;
    Tab) echo KEY_TAB ;; BackSpace) echo KEY_BACKSPACE ;; Delete) echo KEY_DELETE ;;
    Up|Down|Left|Right|Home|End|Minus|Equal|Dot|Comma|Slash|Semicolon|Apostrophe|Grave) echo "KEY_${1^^}" ;;
    F[0-9]|F1[0-2]|[a-zA-Z0-9]) echo "KEY_${1^^}" ;;
    KEY_*) echo "$1" ;;
    *) echo "KEY_${1^^}" ;;
  esac
}
key() { local ks=(); for k in "$@"; do ks+=("$(keyname "$k")"); done; v send-key "$VM" --holdtime 60 "${ks[@]}" >/dev/null; }

type_text() {
  local s=$1 c
  for ((i = 0; i < ${#s}; i++)); do
    c=${s:i:1}
    case $c in
      [a-z0-9]) key "$c" ;;
      [A-Z]) key shift "${c,,}" ;;
      ' ') key space ;; $'\n') key Return ;;
      '-') key Minus ;; '=') key Equal ;; '.') key Dot ;; ',') key Comma ;; '/') key Slash ;;
      ';') key Semicolon ;; "'") key Apostrophe ;; '`') key Grave ;; '[') key LEFTBRACE ;; ']') key RIGHTBRACE ;; '\') key BACKSLASH ;;
      '!') key shift 1 ;; '@') key shift 2 ;; '#') key shift 3 ;; '$') key shift 4 ;; '%') key shift 5 ;;
      '^') key shift 6 ;; '&') key shift 7 ;; '*') key shift 8 ;; '(') key shift 9 ;; ')') key shift 0 ;;
      '_') key shift Minus ;; '+') key shift Equal ;; ':') key shift Semicolon ;; '"') key shift Apostrophe ;;
      '<') key shift Comma ;; '>') key shift Dot ;; '?') key shift Slash ;; '~') key shift Grave ;;
      '{') key shift LEFTBRACE ;; '}') key shift RIGHTBRACE ;; '|') key shift BACKSLASH ;;
      *) echo "type: unmapped char '$c'" >&2; return 1 ;;
    esac
    sleep 0.03
  done
}

cmd=${1:-}; shift || true
case $cmd in
  shot) shot "$@" ;; move) move "$@" ;; click) click "$@" ;; dbl) dbl "$@" ;; drag) drag "$@" ;;
  wheel) wheel "$@" ;; key) key "$@" ;; type) type_text "$@" ;; size) size ;;
  *) sed -n '2,15p' "$0"; exit 1 ;;
esac
