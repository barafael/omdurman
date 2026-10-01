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

This runs on the developer desktop (KDE Wayland): the game opens as an
XWayland window there, and `xdotool` drives it.

## Prerequisites

`xdotool` and ImageMagick (`magick`) on `PATH`, a desktop session with
XWayland (`DISPLAY` set). There is no software Vulkan driver here, so
`Xvfb` cannot host the game.

## Run (agent path)

```bash
D=.claude/skills/run-omdurman/driver.sh
$D launch Lobby          # cargo run (builds if needed), offline self-host, lobby open
$D shot lobby            # -> /tmp/omdurman-play/lobby.png, half size: double coords to click
```

Read the screenshot, then click by window pixels (full resolution). For the
3072x1704 window, a solo Campaign as the Anglo-Egyptians against the AI
Khalifa is:

```bash
$D click 1202 424        # Faction: Anglo-Egyptian
$D click 1470 715        # AI Commanders: Khalifa (Dervish)
$D click 1535 799        # Start Battle
$D wait 'phase=Movement active=AngloEgyptian' 900   # the AI Dervish sets up first
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
$D units Dervish         # "owner q r disrupted id identity" per unit
$D key e                 # End phase;  key Return dismisses the end-of-turn telegram
$D ff 3 AngloEgyptian    # press E on your phases, wait out the AI, until turn 3 movement
$D log 'fire resolved'   # every resolved attack: firers, roll, modifier, band, result
$D stop
```

Fire: select a unit (`hex`) or tile (`dbl`), click enemy hexes to allocate,
then click "Resolve N attacks" in the tray; the combat cards appear top
right. Melee: `dbl` your tile, `hex` the adjacent enemy, click "Resolve
Melee". Set-up and reinforcements: click a sidebar counter, then `hex` the
target; "Auto next" keeps selecting the next counter of the same group.

## Run (human path)

```bash
cargo run -p omdurman-app
```

Splash → Lobby → pick a faction and scenario → tick an AI commander for the
other side → Start Battle. With `OMDURMAN_OFFLINE=1` it self-hosts with no
signalling server.

## Gotchas

- **Bevy is linked dynamically**: `target/debug/omdurman` alone fails with
  `libbevy_dylib-*.so: cannot open shared object file`. Always go through
  `cargo run` (the driver does).
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
- **Both factions AI + Faction: Spectate** plays a whole scenario
  unattended (good for end-of-game rules such as victory levels).

## Troubleshooting

- Lobby lists nobody and Start Battle stays grey offline → fixed in
  `net_plugin::setup_offline` (it must call `refresh_sorted`); rebuild.
- "Waiting on Dervish" forever in Campaign set-up with an AI Dervish →
  fixed in `bot_player::ai_chooser` (set-up uses `player_to_act`); rebuild.
- `hex q,r not on screen` → the hex is outside the camera; `zoom` with a
  negative count, or `zoom` at a visible hex nearer the target.
- `shot` prints "no screenshot" → the app is not running (`$D launch`).
