//! Keyboard shortcuts: one gate for "is the keyboard free?" and one message
//! for the board-command keys, so a key press and its on-screen button share
//! a single implementation.
//!
//! * [`EguiKeyboardFocus`] is a per-frame snapshot (written in `First`, like
//!   `EguiPointerOverUi`) of whether egui owns the keyboard — a text field
//!   (rulebook search, lobby room id, player name) has focus. Every hotkey
//!   system runs under [`keyboard_free`], so typing "c" or pressing Backspace
//!   in a text field never toggles the chart sheet or undoes a movement leg.
//! * [`PickerCommand`] is what the board-command keys (Enter / Backspace /
//!   Del / Esc) and right-click *mean*. The key reader
//!   ([`picker_hotkeys`]), the board click router
//!   (`board_click::route_board_clicks`, right-click) and the actions-panel
//!   buttons all emit it; the
//!   picker's handlers (`confirm_movement_path`, `undo_movement_leg`,
//!   `delete_selected_unit`, `cancel_placement`) consume it.

use bevy::ecs::message::{Message, MessageWriter};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Per-frame snapshot: does egui want keyboard input (a focused widget, e.g.
/// a text field being typed into)? Refreshed in `First` from the last
/// completed egui pass; read by the [`keyboard_free`] run condition.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct EguiKeyboardFocus(pub bool);

/// Refresh [`EguiKeyboardFocus`] for this frame (runs in `First`).
pub fn sync_egui_keyboard_focus(mut contexts: EguiContexts, mut focus: ResMut<EguiKeyboardFocus>) {
    focus.0 = contexts
        .ctx_mut()
        .ok()
        .is_some_and(|ctx| ctx.egui_wants_keyboard_input());
}

/// Run condition for every hotkey system: the keyboard is not being typed
/// into an egui widget. (A missing snapshot — headless tests, no egui —
/// counts as free.)
pub fn keyboard_free(focus: Option<Res<EguiKeyboardFocus>>) -> bool {
    !focus.is_some_and(|f| f.0)
}

/// A board command, from a key, a right-click, or an actions-panel button.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerCommand {
    /// Commit the plotted movement path (Enter).
    ConfirmMove,
    /// Pop the last leg of the plotted path (Backspace).
    UndoStep,
    /// Return the selected, deployed unit to the tray during Setup (Del).
    ReturnToTray,
    /// Cancel: drop the path, the selection, a pending placement, and close
    /// the fire-allocation tray (Esc / right-click).
    Cancel,
    /// Narrow a stack / tile selection to one of its members (a click on its
    /// name in the "Selected units" panel) -- the way to reach a counter
    /// buried in a stack, e.g. the one battery of three that has not fired.
    SelectMember(Entity),
}

impl PickerCommand {
    /// The key a command is bound to, for button labels and the help line.
    pub fn key_label(self) -> &'static str {
        match self {
            Self::ConfirmMove => "Enter",
            Self::UndoStep => "Backspace",
            Self::ReturnToTray => "Del",
            Self::Cancel => "Esc",
            Self::SelectMember(_) => "select stack member",
        }
    }
}

/// The key → command mapping for the picker keys. `chart_sheet_open`
/// yields Esc to the chart sheet (it closes first; a second Esc cancels).
pub fn command_for_keys(keys: &ButtonInput<KeyCode>, chart_sheet_open: bool) -> Vec<PickerCommand> {
    let mut out = Vec::new();
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        out.push(PickerCommand::ConfirmMove);
    }
    if keys.just_pressed(KeyCode::Backspace) {
        out.push(PickerCommand::UndoStep);
    }
    if keys.just_pressed(KeyCode::Delete) {
        out.push(PickerCommand::ReturnToTray);
    }
    if keys.just_pressed(KeyCode::Escape) && !chart_sheet_open {
        out.push(PickerCommand::Cancel);
    }
    out
}

/// Translate the picker keys into [`PickerCommand`]s. Gated by
/// [`keyboard_free`] at registration.
pub fn picker_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    sheet: Option<Res<crate::charts::ChartSheet>>,
    mut writer: MessageWriter<PickerCommand>,
) {
    let chart_open = sheet.is_some_and(|s| s.is_open());
    for cmd in command_for_keys(&keys, chart_open) {
        writer.write(cmd);
    }
}

/// The shortcut reference shown by the toolbar's "Keys" popover.
pub const KEY_HELP: &[(&str, &str)] = &[
    ("M", "Menu"),
    ("C", "Charts & rulebook sheet"),
    ("V", "Event viewer"),
    ("E", "End the current phase"),
    ("R", "Open the hovered hex's rule in the rulebook"),
    (
        "Enter",
        "Confirm the plotted move / resolve staged fire / resolve the melee",
    ),
    ("Backspace", "Undo the last movement step"),
    ("Del", "Return the selected unit to the tray (setup)"),
    (
        "Esc",
        "Close charts, else cancel path / selection / fire tray",
    ),
    ("Right-click", "Cancel (same as Esc)"),
    ("Double-click", "Select the whole stack / combat tile"),
    ("Right-drag / arrows", "Pan the map"),
    ("Wheel", "Zoom (Ctrl+wheel / PgUp / PgDn tilt)"),
    ("Home", "Fit the whole board in view"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn pressed(keys: &[KeyCode]) -> ButtonInput<KeyCode> {
        let mut input = ButtonInput::<KeyCode>::default();
        for &k in keys {
            input.press(k);
        }
        input
    }

    #[test]
    fn keys_map_to_commands() {
        assert_eq!(
            command_for_keys(&pressed(&[KeyCode::Enter]), false),
            vec![PickerCommand::ConfirmMove]
        );
        assert_eq!(
            command_for_keys(&pressed(&[KeyCode::Backspace]), false),
            vec![PickerCommand::UndoStep]
        );
        assert_eq!(
            command_for_keys(&pressed(&[KeyCode::Delete]), false),
            vec![PickerCommand::ReturnToTray]
        );
        assert_eq!(
            command_for_keys(&pressed(&[KeyCode::Escape]), false),
            vec![PickerCommand::Cancel]
        );
    }

    #[test]
    fn escape_goes_to_the_open_chart_sheet_first() {
        assert!(command_for_keys(&pressed(&[KeyCode::Escape]), true).is_empty());
    }

    #[test]
    fn keyboard_free_follows_the_focus_snapshot() {
        let mut world = World::new();
        world.insert_resource(EguiKeyboardFocus(false));
        let free = world.run_system_cached(keyboard_free).unwrap();
        assert!(free);
        world.insert_resource(EguiKeyboardFocus(true));
        let free = world.run_system_cached(keyboard_free).unwrap();
        assert!(!free, "typing into egui must gate the hotkeys");
    }

    #[test]
    fn cancel_command_closes_the_fire_tray_and_deselects() {
        let mut app = App::new();
        app.add_message::<PickerCommand>()
            .insert_resource(crate::picker::PickerState::default())
            .insert_resource(crate::picker::MovementPath::default())
            .insert_resource(crate::fire_allocation::FireAllocationState {
                panel_open: true,
                ..Default::default()
            })
            .add_systems(Update, crate::picker::cancel_placement);
        app.world_mut().write_message(PickerCommand::Cancel);
        app.update();
        let alloc = app
            .world()
            .resource::<crate::fire_allocation::FireAllocationState>();
        assert!(!alloc.panel_open, "Esc / right-click closes the tray");
        assert!(matches!(
            *app.world().resource::<crate::picker::PickerState>(),
            crate::picker::PickerState::Idle
        ));
    }

    #[test]
    fn hotkeys_are_silent_while_typing() {
        let mut app = App::new();
        app.add_message::<PickerCommand>()
            .insert_resource(EguiKeyboardFocus(true))
            .insert_resource(pressed(&[KeyCode::Enter, KeyCode::Escape]))
            .add_systems(Update, picker_hotkeys.run_if(keyboard_free));
        app.update();
        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<PickerCommand>>();
        assert!(msgs.is_empty());

        app.insert_resource(EguiKeyboardFocus(false));
        app.update();
        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<PickerCommand>>();
        assert_eq!(msgs.len(), 2);
    }
}
