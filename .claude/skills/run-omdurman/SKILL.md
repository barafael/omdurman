---
name: run-omdurman
description: Run, launch, play, screenshot or click-test the native omdurman Bevy game (Remember Gordon!) — start a solo game against the AI, drive it by real mouse/keyboard input, read the rules state back, fast-forward turns. Use for rules play-throughs, UI checks and reproducing in-game bugs.
---

# Run omdurman (native, driven by clicks)

Drive the real game window with `xdotool` and read it back through the
app's own **hex probe** (`OMDURMAN_HEX_PROBE`): every hex's window-pixel
position, the rules state (turn, phase, every unit), and in-app
screenshots on request. The driver is
`.claude/skills/run-omdurman/driver.sh` (paths are relative to the repo
root). Everything lands in `/tmp/omdurman-play/` (override with
`OMDURMAN_PLAY`).

This runs on a Wayland desktop: the game opens as an XWayland window, and
`xdotool` drives it. KDE honours `xdotool`'s pointer warps; COSMIC (and
GNOME) ignore them, so there the driver moves the pointer with `ydotool`
instead (see Prerequisites) -- automatically, whenever a warp does not land.

## Prerequisites

`xdotool` and ImageMagick (`magick`) on `PATH`, a desktop session with
XWayland (`DISPLAY` set). There is no software Vulkan driver here, so
`Xvfb` cannot host the game.

On a compositor that ignores pointer warps (COSMIC, GNOME), also `ydotool`
(1.x, kernel uinput: the user needs write access to `/dev/uinput`) with its
daemon running as the user, on the socket the driver expects:

```bash
export YDOTOOL_SOCKET=/run/user/$(id -u)/.ydotool_socket
setsid nohup ydotoold --socket-path=$YDOTOOL_SOCKET --socket-own=$(id -u):$(id -g) >/dev/null 2>&1 &
```

`ydotool` moves are relative and go through pointer acceleration, so the
driver steers in a closed loop against `xdotool getmouselocation` until the
pointer sits on the exact pixel (a `click` takes ~0.6 s).

## Run (agent path)

```bash
D=.claude/skills/run-omdurman/driver.sh
$D launch Lobby          # cargo run (builds if needed -- a cold build takes many minutes;
                         # launch waits for it), offline self-host, lobby open
$D shot lobby            # -> /tmp/omdurman-play/lobby.png, half size: double coords to click
```

Read the screenshot, then click by window pixels (full resolution: double
the half-size shot's coordinates). The window size differs per machine
(`launch` prints it), so lobby pixels are only valid for the size they were
read at. For a 3072x1704 window, a solo Campaign as the Anglo-Egyptians
against the AI Khalifa is (for a 2208x1403 window: 672 570, 1004 1006,
1104 1132; any other size: read them off a `shot`):

```bash
$D click 1202 424        # Faction: Anglo-Egyptian
$D click 1470 715        # AI Commanders: Khalifa (Dervish)
$D click 1535 799        # Start Battle
$D wait 'phase=Setup active=AngloEgyptian' 900      # the AI Dervish sets up first
$D click 32 128          # Ready (rail): the A-E deploy nothing in the Campaign,
                         # but set-up waits for it -- the game sits idle until then
$D wait 'phase=Movement active=AngloEgyptian' 60
$D state                 # turn/phase line + units per side
```

Lobby positions shift with the scenario (the Optional Rules box only shows
for the Campaign) and when an AI row is added: take a `shot` (or a crop,
`$D shot name WxH+X+Y`) before each lobby click.

In game, address the board by hex, never by pixel:

```bash
$D zoom 33 13 4          # wheel-zoom 4 notches in at hex (33,13); negative zooms out
$D hex 33 13             # click: select a counter / target / place
$D dbl 33 13             # double-click: select the whole tile (combined fire, melee)
$D hover 33 13           # tooltips, LOS overlay origin
$D drag 74 268 24 0      # press on a sidebar counter (window px), release on hex (24,0)
$D drag 74 268 160 700 px  # ... or release on a window pixel
$D scroll 150 900 -15    # mouse wheel at a window pixel (the sidebar): - is down
$D units Dervish         # "owner q r disrupted id identity" per unit
$D key e                 # End phase;  key Return dismisses the end-of-turn telegram
$D ff 3 AngloEgyptian    # press E on your phases, wait out the AI, until turn 3 movement
                         # (stops early on game over: the probe line gains `game_over result=...`)
$D log 'fire resolved'   # every resolved attack: firers, roll, modifier, band, result
$D pan Up 0.3           # hold an arrow key: pan the camera
$D stop
```

Fire: select a unit (`hex`) or tile (`dbl`), click enemy hexes to allocate,
then click "Resolve N attacks" in the tray; the combat cards appear top
right. Melee: `dbl` your tile, `hex` the adjacent enemy, click "Resolve
Melee". Set-up and reinforcements: click a sidebar counter, then `hex` the
target; "Auto next" keeps selecting the next counter of the same group --
so a whole group goes down from one sidebar click plus one `hex` per
counter, in the sidebar's row-major order. A drag released off the board
leaves its counter selected: a click on that counter then deselects it.

## Run (human path)

```bash
cargo run -p omdurman-app
```

Splash → Lobby → pick a faction and scenario → tick an AI commander for the
other side → Start Battle. With `OMDURMAN_OFFLINE=1` it self-hosts with no
signalling server.

## Gotchas

- **Two games side by side** (to play both sides of an online game): give
  each its own state dir and player slot, and the same room:
  `OMDURMAN_PLAY=/tmp/omdurman-a OMDURMAN_PLAYER_SLOT=7 OMDURMAN_ROOM=omd-test-xyz $D launch`
  and `OMDURMAN_PLAY=/tmp/omdurman-b OMDURMAN_PLAYER_SLOT=8 OMDURMAN_ROOM=omd-test-xyz $D launch`
  (online, via the baked-in matchbox server; without `OMDURMAN_ROOM` the
  game self-hosts offline). Each driver call then addresses the window its
  `launch` recorded (`$OMDURMAN_PLAY/win`); `stop` stops only that one.
- **The pointer only reads back over X windows**: `moveto` steers by
  `xdotool getmouselocation`, which freezes (reporting the root window)
  wherever a native Wayland window covers the game. A click there fails
  with "pointer stuck ... is a non-game window covering that part of the
  game?" -- keep the game window uncovered.
- **The driver refuses input unless the game is the active window**
  (exit 2, "refusing input: active window is ..."): on this desktop the
  game opens on a monitor the user may be using -- xdotool input lands in
  whatever window is on top. Ask the user to bring the game window to the
  front (or to stop what they are doing) instead of working around it.
  Under Wayland the check only sees X windows: a focused native window
  (the terminal) is invisible to it, so it is a guard against other X
  windows only.
- **`key` can miss** when the game lacks keyboard focus (e.g. after the user
  clicked elsewhere): check the probe's phase line after a `key`, and use
  the on-screen button (End phase is in the Game control panel) instead.
- **Bevy is linked dynamically**: `target/debug/omdurman` alone fails with
  `libbevy_dylib-*.so: cannot open shared object file`. Always go through
  `cargo run` (the driver does).
- **Probe pixels are physical window pixels** (the probe scales Bevy's
  logical viewport coordinates by the window's scale factor), the same
  space as `shot` and the input tools. On a HiDPI setup (scale 1.5 here)
  a probe from before that fix is off by the scale.
- **The audio backend can flood the log**: with a broken ALSA stream rodio
  logs `alsa::poll() returned POLLERR` thousands of times a second (7 GB in
  ten minutes filled the `/tmp` tmpfs). The driver's default `RUST_LOG`
  turns `rodio` and `cpal` off; keep that if you override `RUST_LOG`.
- **Unset `WAYLAND_DISPLAY`** (the driver does) or winit opens a native
  Wayland window that `xdotool` cannot touch.
- **Screenshots from outside are stale under XWayland**; use the probe's
  in-app screenshot (`shot`), which the driver requests by touching
  `probe.shot`.
- **The sidebar reflows** when a counter leaves it: to place several from
  one grid, click them from the end of the group backwards, or re-shoot
  between clicks. Auto-next stops at the end of a group.
- **Set-up order ≠ turn order**: in the Campaign the Dervish deploy first,
  the Anglo-Egyptians move first. `state`'s `active=` is the side to act now
  (`GameState::player_to_act`): the deployer in set-up, the non-moving side
  in defensive fire.
- **End-of-turn "Field Telegram"** is modal: keys and clicks do nothing
  until `key Return` (or its Continue button). `ff` presses Return first.
- **The probe refreshes twice a second**: after `zoom` or a camera move,
  wait before `hex`/`dbl` (`zoom` already sleeps 1.5 s).
- **AI turns are slow** (one hex of movement per event, ~2 events/s): a
  Campaign game turn takes ~3 min, Fall of Khartoum ~1 min.
- **zsh is the login shell**: unquoted `$var` does not word-split; loops
  over `"q r"` pairs belong in `bash`. Never `pkill -f` a pattern that
  also appears in your own command line — `stop` uses `pgrep -x omdurman`.
- **Both factions AI + Faction: Spectate** plays a whole scenario (good
  for end-of-game rules such as victory levels) -- but the host's AI waits
  for each end-of-turn telegram to be read, so keep pressing Return:
  `until grep -q game_over /tmp/omdurman-play/probe.state; do $D key Return; sleep 8; done`.
  The probe's state line ends in `game_over result=...` once it is over.

## Troubleshooting

- Lobby lists nobody and Start Battle stays grey offline → fixed in
  `net_plugin::setup_offline` (it must call `refresh_sorted`); rebuild.
- "Waiting on Dervish" forever in Campaign set-up with an AI Dervish →
  fixed in `bot_player::ai_chooser` (set-up uses `player_to_act`); rebuild.
- `hex q,r not on screen` → the hex is outside the camera; `zoom` with a
  negative count, or `zoom` at a visible hex nearer the target.
- `shot` prints "no screenshot" → the app is not running (`$D launch`).
