//! Board animation: every visual on the board that plays out over time
//! instead of landing in one frame, in one place and under one setting
//! ([`MotionSettings`]).
//!
//! A state change reaches the engine in one step; a few of them are worth a
//! moment on the board, where they happened -- no more than that, and quietly:
//!
//! * counters glide along the route they took and turn over when disrupted
//!   ([`counters`]);
//! * a thin ring marks where the opponent (or the AI) moved, entered or
//!   attacked, and a small pointer at the board's edge leads to it while it
//!   is out of view ([`announce`], [`pointers`]);
//! * a faint trace runs from the firers to the target (with a hop to where a
//!   howitzer shell scattered), a mark between melee opponents, and an
//!   eliminated counter fades where it stood ([`transient`]); the review
//!   timeline shows the same marks for the event it stands on ([`review`]);
//! * a refused click flashes its hex, and a committed move keeps faint dots
//!   along its route until its echo arrives ([`orders`]);
//! * the combat card under the pointer rings its hexes ([`pointers`]);
//! * a short note says when it becomes your move ([`banner`]).
//!
//! All of it is presentation only: the engine state is final before anything
//! starts, and nothing here gates input, submissions or the apply path. The
//! marks are raised from *live* changes only -- [`LiveApplied`] (filled on the
//! sequenced echo) and the engine's observations (discarded by a rebuild) --
//! so a late-join install or a timeline jump raises none; a jump also raises
//! [`SnapSprites`], which lands every counter at once and drops what was still
//! on screen. Everything is finite and asks for frames only while it plays
//! (see [`crate::activity`]).

use std::collections::HashMap;

use bevy::ecs::message::Message;
use bevy::prelude::*;
use bevy_egui::{EguiPrimaryContextPass, egui};
use omdurman_rules::UnitId;
use omdurman_rules::effects::GameState;
use omdurman_types::HexCoord;

pub mod announce;
pub mod banner;
pub mod counters;
pub mod orders;
pub mod pointers;
pub mod review;
pub mod transient;

pub use counters::{CounterTurn, MovementAnimation};
pub use pointers::CardFocus;
pub use transient::BoardBusy;

// -- Settings -----------------------------------------------------------------

/// How much the board animates.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MotionLevel {
    /// Counters glide and turn; marks fade in and out.
    #[default]
    Full,
    /// Nothing moves: counters land at once, marks only fade.
    Reduced,
    /// No marks at all: the board changes in one frame.
    Off,
}

impl MotionLevel {
    pub const ALL: [MotionLevel; 3] = [MotionLevel::Full, MotionLevel::Reduced, MotionLevel::Off];

    pub fn label(self) -> &'static str {
        match self {
            MotionLevel::Full => "Full",
            MotionLevel::Reduced => "Reduced",
            MotionLevel::Off => "Off",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            MotionLevel::Full => "Counters glide and turn over; marks fade in and out",
            MotionLevel::Reduced => "Nothing moves: counters land at once, marks only fade",
            MotionLevel::Off => "No marks: the board changes at once",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|level| level.label().eq_ignore_ascii_case(name.trim()))
    }
}

/// The local player's animation preferences (not networked: purely how this
/// window draws the shared state).
#[derive(Resource, Clone, Copy, Debug)]
pub struct MotionSettings {
    pub level: MotionLevel,
    /// Pan the camera to an opponent's action that happens out of view.
    pub follow_opponent: bool,
}

impl Default for MotionSettings {
    fn default() -> Self {
        Self {
            level: initial_level(),
            follow_opponent: false,
        }
    }
}

impl MotionSettings {
    /// Whether marks are drawn at all.
    pub fn effects(&self) -> bool {
        self.level != MotionLevel::Off
    }

    /// Whether things may *move* (glide, turn, grow, pan), rather than only
    /// appear and fade.
    pub fn motion(&self) -> bool {
        self.level == MotionLevel::Full
    }
}

/// `OMDURMAN_MOTION=full|reduced|off` (headless runs, measuring), else the
/// browser's reduced-motion preference on the web, else full.
fn initial_level() -> MotionLevel {
    if let Some(level) = std::env::var("OMDURMAN_MOTION")
        .ok()
        .and_then(|v| MotionLevel::from_name(&v))
    {
        return level;
    }
    if prefers_reduced_motion() {
        MotionLevel::Reduced
    } else {
        MotionLevel::Full
    }
}

#[cfg(target_arch = "wasm32")]
fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|query| query.matches())
}

#[cfg(not(target_arch = "wasm32"))]
fn prefers_reduced_motion() -> bool {
    false
}

/// The toolbar's Motion menu: the level, and the follow camera.
pub fn motion_menu(ui: &mut egui::Ui, settings: &mut MotionSettings) {
    ui.set_min_width(220.0);
    for level in MotionLevel::ALL {
        ui.radio_value(&mut settings.level, level, level.label())
            .on_hover_text(level.hint());
    }
    ui.separator();
    ui.add_enabled(
        settings.motion(),
        egui::Checkbox::new(&mut settings.follow_opponent, "Follow the opponent"),
    )
    .on_hover_text("Pan to what the opponent does out of view")
    .on_disabled_hover_text("Needs full motion");
}

// -- Live changes ---------------------------------------------------------------

/// The engine state *jumped* to a new position instead of playing out to it
/// (a history install, a timeline jump): the next sprite reconcile lands every
/// counter at once instead of gliding, turning or fading it, and the marks
/// still on screen are dropped. Inserted by the rebuilding system, removed by
/// [`crate::picker::reconcile_unit_sprites`].
#[derive(Resource, Default)]
pub struct SnapSprites;

/// What one live (sequenced-echo) event did to the counters: who moved where,
/// who entered. Filled by the receive path, never by a rebuild, so replays and
/// installs raise no marks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveChange {
    /// Units now on another hex: `(unit, new hex)`.
    pub moved: Vec<(UnitId, HexCoord)>,
    /// Units new on the board: `(unit, hex)`.
    pub placed: Vec<(UnitId, HexCoord)>,
    /// The change happened during set-up (deployment is no news).
    pub setup: bool,
}

impl LiveChange {
    /// Positions of every unit, to diff against after an apply.
    pub fn snapshot(gs: &GameState) -> HashMap<UnitId, HexCoord> {
        gs.units.iter().map(|u| (u.id, u.position)).collect()
    }

    /// The change from `before` (a [`snapshot`](Self::snapshot)) to `after`.
    pub fn between(before: &HashMap<UnitId, HexCoord>, after: &GameState, setup: bool) -> Self {
        let mut change = LiveChange {
            setup,
            ..Default::default()
        };
        for unit in &after.units {
            match before.get(&unit.id) {
                Some(&was) if was != unit.position => change.moved.push((unit.id, unit.position)),
                Some(_) => {}
                None => change.placed.push((unit.id, unit.position)),
            }
        }
        change
    }

    pub fn is_empty(&self) -> bool {
        self.moved.is_empty() && self.placed.is_empty()
    }
}

/// Live changes awaiting [`announce::announce_live_changes`].
#[derive(Resource, Default)]
pub struct LiveApplied(pub Vec<LiveChange>);

// -- Requests -------------------------------------------------------------------

/// Ask for a mark on the board. Written by whoever knows something happened;
/// [`transient::spawn_fx`] owns the assets and the board geometry.
#[derive(Message, Clone, Debug)]
pub enum FxRequest {
    /// Draw the eye to a hex: one thin ring, widening as it fades.
    Attention { hex: HexCoord },
    /// A shot from one hex at another (`scatter`: a howitzer shell's hop
    /// from the aimed hex to where it landed, drawn finer).
    Shot {
        from: HexCoord,
        to: HexCoord,
        scatter: bool,
    },
    /// Melee: a mark between the two hexes, pointing at the defenders.
    Clash {
        attacker: HexCoord,
        defender: HexCoord,
    },
    /// A hex an order could not use.
    Refused { hex: HexCoord },
    /// A counter that left the board fades where it stood.
    Ghost {
        transform: Transform,
        mesh: Handle<Mesh>,
        texture: Option<Handle<Image>>,
        tint: Color,
    },
}

// -- Plugin ---------------------------------------------------------------------

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        let on_board = in_state(crate::AppMode::Game);
        app.init_resource::<MotionSettings>()
            .init_resource::<LiveApplied>()
            .init_resource::<pointers::Sightings>()
            .init_resource::<CardFocus>()
            .add_message::<FxRequest>()
            .add_systems(Startup, transient::spawn_fx_assets)
            .add_systems(
                Update,
                (
                    transient::clear_effects_on_snap
                        .after(crate::net_socket::handle_reconnect)
                        .after(crate::timeline::scrub_rebuild)
                        .before(crate::picker::reconcile_unit_sprites),
                    announce::announce_live_changes
                        .after(crate::net_socket::handle_socket)
                        .before(transient::spawn_fx),
                    announce::announce_combat
                        .after(crate::events::drain_observations)
                        .before(transient::spawn_fx),
                    review::review_marks
                        .run_if(in_state(crate::AppState::Spectating))
                        .after(crate::timeline::scrub_rebuild)
                        .after(transient::clear_effects_on_snap)
                        .before(transient::spawn_fx),
                    orders::flash_refused_legs
                        .in_set(crate::GameSet)
                        .after(crate::picker::handle_picker_clicks)
                        .before(transient::spawn_fx),
                    orders::pending_order_trail.in_set(crate::GameSet),
                    pointers::card_focus_rings.run_if(on_board.clone()),
                    transient::spawn_fx
                        .after(crate::picker::reconcile_unit_sprites)
                        .run_if(on_board.clone()),
                    transient::animate_transients.after(transient::spawn_fx),
                ),
            )
            .add_systems(
                Update,
                (
                    // The counters follow the engine state in every board
                    // view; the reconcile starts their glides and turns.
                    counters::animate_unit_movement.after(crate::picker::reconcile_unit_sprites),
                    counters::animate_counter_turns.after(crate::picker::reconcile_unit_sprites),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    // After the rail and the charts sheet: both draw on the
                    // board those leave free (`ScreenLayout` insets).
                    pointers::offscreen_pointers_ui
                        .after(crate::ui_plugin::LeftRailSet)
                        .after(crate::charts::chart_sheet_ui)
                        .run_if(crate::map_view_active),
                    banner::your_move_banner_ui
                        .after(crate::ui_plugin::mode_toolbar_ui)
                        .after(crate::ui_plugin::LeftRailSet)
                        .after(crate::charts::chart_sheet_ui)
                        .run_if(in_state(crate::AppState::InGame).and_then(on_board)),
                ),
            )
            .add_systems(
                OnExit(crate::AppMode::Game),
                transient::clear_effects_on_exit,
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_live_change_lists_moves_and_entries_only() {
        use omdurman_rules::unit_profiles::profile_for_unit;
        use omdurman_rules::{UnitPlacement, UnitState};
        let mut gs = GameState::new(omdurman_types::Scenario::Campaign);
        let unit = |id, q| UnitPlacement {
            id,
            position: HexCoord::new(q, 0),
            profile: profile_for_unit(id).unwrap(),
            state: UnitState::default(),
        };
        gs.units.push(unit(UnitId::Taiasha_0_0, 1));
        gs.units.push(unit(UnitId::BritishBoats_3_0, 5));
        let before = LiveChange::snapshot(&gs);
        gs.units[0].position = HexCoord::new(2, 0);
        gs.units.push(unit(UnitId::AliWadHelu_0_0, 7));
        let change = LiveChange::between(&before, &gs, false);
        assert_eq!(
            change.moved,
            vec![(UnitId::Taiasha_0_0, HexCoord::new(2, 0))]
        );
        assert_eq!(
            change.placed,
            vec![(UnitId::AliWadHelu_0_0, HexCoord::new(7, 0))]
        );
    }

    #[test]
    fn the_motion_level_parses_its_labels() {
        assert_eq!(MotionLevel::from_name("off"), Some(MotionLevel::Off));
        assert_eq!(
            MotionLevel::from_name(" Reduced "),
            Some(MotionLevel::Reduced)
        );
        assert_eq!(MotionLevel::from_name("FULL"), Some(MotionLevel::Full));
        assert_eq!(MotionLevel::from_name("wobbly"), None);
    }
}
