//! Off-by-default screenshot capture for verifying UI/theme work.
//!
//! Entirely inert unless `OMDURMAN_SCREENSHOT` is set to an output path. When
//! set, the app captures the primary window once, some frames in (so the splash
//! and egui have rendered), writes a PNG, and exits. This is a development
//! verification aid -- the physics/game paths are untouched.
//!
//! ```text
//! OMDURMAN_SCREENSHOT=out.png OMDURMAN_SCREENSHOT_FRAMES=600 cargo run -p omdurman-app
//! ```
//! `OMDURMAN_SCREENSHOT_FRAMES` (optional) sets the capture frame; default 300.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

/// The output path and target frame, read once from the environment.
#[derive(Resource)]
struct CaptureConfig {
    path: String,
    at_frame: u32,
}

#[derive(Resource, Default)]
struct FrameCounter(u32);

pub struct DebugCapturePlugin;

impl Plugin for DebugCapturePlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var("OMDURMAN_SCREENSHOT") else {
            return; // Not requested -- add nothing.
        };
        let at_frame = std::env::var("OMDURMAN_SCREENSHOT_FRAMES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);
        info!(%path, at_frame, "debug capture armed");
        app.insert_resource(CaptureConfig { path, at_frame })
            .init_resource::<FrameCounter>()
            .add_systems(Update, capture_then_exit);
    }
}

fn capture_then_exit(
    mut commands: Commands,
    config: Res<CaptureConfig>,
    mut counter: ResMut<FrameCounter>,
    mut captured_at: Local<Option<u32>>,
    mut exit: MessageWriter<AppExit>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    // Counts frames: keep them coming (the app otherwise idles between
    // inputs, see `activity`).
    activity.keep_running();
    counter.0 += 1;
    if let Some(at) = *captured_at {
        // Hold for a margin of frames after the capture request: the screenshot
        // is read back on the render thread and delivered through a channel, so
        // exiting the same frame closes the channel before the image lands on
        // disk ("sending on a closed channel"). ~30 frames is ample.
        if counter.0 >= at + 30 {
            exit.write(AppExit::Success);
        }
        return;
    }
    if counter.0 >= config.at_frame {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(config.path.clone()));
        info!(frame = counter.0, path = %config.path, "capturing screenshot");
        *captured_at = Some(counter.0);
    }
}

/// Off-by-default hex probe for scripted play-testing: with
/// `OMDURMAN_HEX_PROBE=<path>` set, the app rewrites `<path>` twice a second
/// with one `q r x y` line per board hex -- the hex centre in window
/// (physical) pixels under the current camera -- so an external driver
/// (xdotool, ydotool) can click a hex by coordinate. Creating `<path>.shot` requests a
/// window screenshot, written to `<path>.png` (the request file is removed).
/// Writing `<path>.input` requests pointer input served by the app itself --
/// one step per line, one step per frame, in window (physical) pixels:
///
/// ```text
/// move X Y      the pointer to (X, Y)
/// down [1|2|3]  press the left (default), middle or right button
/// up [1|2|3]    release it
/// click [1|2|3] down, then up on the next frame
/// wheel DY      one scroll of DY lines (+ up / zoom in, - down)
/// wait N        N idle frames
/// ```
///
/// The file is removed once read. The steps go through the same messages
/// winit's would (`CursorMoved`, `MouseButtonInput`, `MouseWheel`, and the
/// `WindowEvent` stream egui reads), so egui widgets, the board picking and
/// the camera see them as real input -- for a desktop whose compositor
/// ignores pointer warps, and they can reach no other window. Inert when
/// unset.
pub struct HexProbePlugin;

#[derive(Resource)]
struct HexProbePath(String);

impl Plugin for HexProbePlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = std::env::var("OMDURMAN_HEX_PROBE") else {
            return;
        };
        info!(%path, "hex probe armed");
        app.insert_resource(HexProbePath(path))
            .add_systems(Last, (write_hex_probe, probe_screenshot, probe_input));
    }
}

/// The board resources the probe projects hex centres from.
#[derive(bevy::ecs::system::SystemParam)]
struct ProbeBoard<'w> {
    map: Option<Res<'w, omdurman_hexmap::GameMap>>,
    layout: Option<Res<'w, omdurman_hexmap::HexLayout>>,
    overlay: Option<Res<'w, omdurman_hexmap::HexOverlay>>,
    turn: Option<Res<'w, crate::TurnState>>,
}

fn write_hex_probe(
    path: Res<HexProbePath>,
    time: Res<Time>,
    mut last: Local<f64>,
    board: ProbeBoard,
    cams: Query<(&Camera, &GlobalTransform), With<crate::camera::RtsCamera>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let now = time.elapsed_secs_f64();
    if now - *last < 0.5 {
        return;
    }
    *last = now;
    let ProbeBoard {
        map,
        layout,
        overlay,
        turn,
    } = board;
    let (Some(map), Some(layout), Some(overlay), Ok((camera, cam_tf))) =
        (map, layout, overlay, cams.single())
    else {
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    // Physical pixels, as the screenshot and the input tools address them
    // (viewport coordinates are logical: off by the DPI scale otherwise).
    let scale = window.single().map_or(1.0, Window::scale_factor);
    let mut out = String::new();
    for coord in map.hexes.keys() {
        let world = omdurman_hexmap::hex_world_pos(*coord, origin, &overlay.params);
        if let Ok(screen) = camera.world_to_viewport(cam_tf, world) {
            out.push_str(&format!(
                "{} {} {:.0} {:.0}\n",
                coord.q,
                coord.r,
                screen.x * scale,
                screen.y * scale
            ));
        }
    }
    let _ = std::fs::write(&path.0, out);
    // The rules state beside it: `<path>.state` has the turn/phase (plus
    // `game_over result=...` once the game has ended) and one
    // `U owner q r disrupted id kind` line per unit. The default engine state
    // before any `StartGame` reads like a game in set-up, so that case says
    // `started=false`: an AI doing nothing then waits for a game, not hung.
    if let Some(gs) = game_state {
        let gs = &gs.0;
        let mut state = format!(
            "T turn={:?} phase={:?} active={:?}{}{}\n",
            gs.current_turn,
            gs.phase,
            gs.player_to_act().unwrap_or(gs.phase_player()),
            if gs.game_over {
                format!(" game_over result={:?}", gs.game_result)
            } else {
                String::new()
            },
            if turn.is_some_and(|turn| turn.game_started) {
                ""
            } else {
                " started=false"
            }
        );
        for u in &gs.units {
            state.push_str(&format!(
                "U {:?} {} {} {} {:?} {:?}\n",
                u.profile.identity.owner(),
                u.position.q,
                u.position.r,
                u.state.disrupted,
                u.id,
                u.profile.identity,
            ));
        }
        let _ = std::fs::write(format!("{}.state", path.0), state);
    }
}

/// Serve a `<probe>.shot` screenshot request (see [`HexProbePlugin`]).
/// How often the probe looks for a `<probe>.shot` request file.
const PROBE_SHOT_POLL_SECS: f64 = 0.25;
/// Frames kept running after a probe screenshot, so it is rendered, read back
/// and written while the app would otherwise idle.
const PROBE_SHOT_FRAMES: u8 = 10;

fn probe_screenshot(
    mut commands: Commands,
    path: Res<HexProbePath>,
    time: Res<Time>,
    mut last_poll: Local<f64>,
    mut capturing: Local<u8>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    if *capturing > 0 {
        *capturing -= 1;
        activity.keep_running();
    }
    // A syscall a frame for a dev aid is too much: poll a few times a second.
    let now = time.elapsed_secs_f64();
    if now - *last_poll < PROBE_SHOT_POLL_SECS {
        return;
    }
    *last_poll = now;
    let request = format!("{}.shot", path.0);
    if std::fs::remove_file(&request).is_ok() {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("{}.png", path.0)));
        *capturing = PROBE_SHOT_FRAMES;
        activity.keep_running();
    }
}

/// One step of a `<probe>.input` request (see [`HexProbePlugin`]).
enum PointerStep {
    /// To a window position, in physical pixels.
    Move(Vec2),
    Down(MouseButton),
    Up(MouseButton),
    Wheel(f32),
    Wait,
}

fn parse_pointer_steps(text: &str) -> Vec<PointerStep> {
    let button = |word: Option<&str>| match word {
        Some("2") => MouseButton::Middle,
        Some("3") => MouseButton::Right,
        _ => MouseButton::Left,
    };
    let mut steps = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        match words.next() {
            Some("move") => {
                let x = words.next().and_then(|w| w.parse::<f32>().ok());
                let y = words.next().and_then(|w| w.parse::<f32>().ok());
                match (x, y) {
                    (Some(x), Some(y)) => steps.push(PointerStep::Move(Vec2::new(x, y))),
                    _ => warn!(line, "hex probe: malformed move"),
                }
            }
            Some("down") => steps.push(PointerStep::Down(button(words.next()))),
            Some("up") => steps.push(PointerStep::Up(button(words.next()))),
            Some("click") => {
                let b = button(words.next());
                steps.push(PointerStep::Down(b));
                steps.push(PointerStep::Up(b));
            }
            Some("wheel") => match words.next().and_then(|w| w.parse::<f32>().ok()) {
                Some(dy) => steps.push(PointerStep::Wheel(dy)),
                None => warn!(line, "hex probe: malformed wheel"),
            },
            Some("wait") => {
                let n = words
                    .next()
                    .and_then(|w| w.parse::<usize>().ok())
                    .unwrap_or(1);
                steps.extend(std::iter::repeat_with(|| PointerStep::Wait).take(n));
            }
            Some(_) => warn!(line, "hex probe: unknown input step"),
            None => {}
        }
    }
    steps
}

/// Serve a `<probe>.input` pointer request (see [`HexProbePlugin`]): one
/// step per frame, through the messages winit's input would arrive by.
#[allow(clippy::too_many_arguments)]
fn probe_input(
    path: Res<HexProbePath>,
    time: Res<Time>,
    mut last_poll: Local<f64>,
    mut queue: Local<std::collections::VecDeque<PointerStep>>,
    mut windows: Query<(Entity, &mut Window), With<bevy::window::PrimaryWindow>>,
    mut window_events: MessageWriter<bevy::window::WindowEvent>,
    mut moved: MessageWriter<bevy::window::CursorMoved>,
    mut buttons: MessageWriter<bevy::input::mouse::MouseButtonInput>,
    mut wheel: MessageWriter<bevy::input::mouse::MouseWheel>,
    mut activity: ResMut<crate::activity::Activity>,
) {
    use bevy::input::ButtonState;
    use bevy::input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
    use bevy::window::{CursorMoved, WindowEvent};

    if queue.is_empty() {
        let now = time.elapsed_secs_f64();
        if now - *last_poll < PROBE_SHOT_POLL_SECS {
            return;
        }
        *last_poll = now;
        let request = format!("{}.input", path.0);
        let Ok(text) = std::fs::read_to_string(&request) else {
            return;
        };
        let _ = std::fs::remove_file(&request);
        queue.extend(parse_pointer_steps(&text));
        if queue.is_empty() {
            return;
        }
        info!(steps = queue.len(), "hex probe: pointer input requested");
    }
    let Ok((window_entity, mut window)) = windows.single_mut() else {
        queue.clear();
        return;
    };
    // The frames keep coming while steps are pending (one per frame), so
    // the app sees them at its normal pace instead of all in one frame.
    activity.keep_running();
    let Some(step) = queue.pop_front() else {
        return;
    };
    match step {
        PointerStep::Move(physical) => {
            window.set_physical_cursor_position(Some(physical.as_dvec2()));
            let position = physical / window.scale_factor();
            let event = CursorMoved {
                window: window_entity,
                position,
                delta: None,
            };
            moved.write(event.clone());
            window_events.write(WindowEvent::CursorMoved(event));
        }
        PointerStep::Down(button) | PointerStep::Up(button) => {
            let state = if matches!(step, PointerStep::Down(_)) {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            };
            let event = MouseButtonInput {
                button,
                state,
                window: window_entity,
            };
            buttons.write(event);
            window_events.write(WindowEvent::MouseButtonInput(event));
        }
        PointerStep::Wheel(dy) => {
            let event = MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: dy,
                window: window_entity,
                phase: bevy::input::touch::TouchPhase::Moved,
            };
            wheel.write(event);
            window_events.write(WindowEvent::MouseWheel(event));
        }
        PointerStep::Wait => {}
    }
}
