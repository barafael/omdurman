use bevy::prelude::*;

/// Whether Ctrl is held (either side). The single place these key codes are
/// OR'd together, so input handlers don't each re-derive it.
pub fn ctrl_held(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
}
