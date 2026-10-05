//! The left-rail status pane: turn/phase text and the hovered-hex label.
use super::*;

#[derive(Component)]
pub(crate) struct StatusPane;

#[derive(Component)]
pub(crate) struct StatusText;

#[derive(Component)]
pub(crate) struct HexCoordLabel;

#[derive(Component)]
pub(crate) struct HexCoordPane;

pub(crate) fn update_status_text(
    state: Res<State<AppState>>,
    mode: Res<State<crate::AppMode>>,
    room: Res<RoomId>,
    mut query: Query<&mut Text, With<StatusText>>,
    mut computed: Local<bool>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    // The line only changes when one of its inputs does; an idle frame
    // re-formats nothing (the write below is compare-guarded anyway, but the
    // `format!` itself is what costs).
    let inputs_changed = state.is_changed() || mode.is_changed() || room.is_changed();
    if *computed && !inputs_changed {
        return;
    }
    *computed = true;
    let new = match state.get() {
        // The title screen shows no game yet: no room / turn line under it.
        _ if *mode.get() == crate::AppMode::Menu => Cow::Borrowed(""),
        AppState::Splash => Cow::Borrowed(""),
        AppState::Lobby => Cow::Owned(format!(
            "Lobby -- choose your faction (share: #room={})",
            room.as_str()
        )),
        // In game the top bar carries the turn, who acts and the room.
        AppState::InGame => Cow::Borrowed(""),
        AppState::Spectating => Cow::Borrowed("Reviewing game -- use the timeline"),
    };
    if text.as_str() != new.as_ref() {
        *text = Text::new(new.into_owned());
    }
}

pub(crate) fn update_hex_coord_display(
    hovered: Res<HoveredHex>,
    mut query: Query<&mut Text, With<HexCoordLabel>>,
) {
    // `HoveredHex` is written with `set_if_neq`, so it reads as changed only
    // when the hover moved: the label is re-formatted then, not every frame.
    if !hovered.is_changed() {
        return;
    }
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let new = match hovered.0 {
        Some(coord) => format!("({}, {})", coord.q, coord.r),
        None => String::new(),
    };
    if text.as_str() != new {
        *text = Text::new(new);
    }
}

/// Keep the bevy_ui bottom panes beside the egui chrome, which draws over
/// them: the status line starts right of the left rail (it used to sit under
/// the sidebar with only its tail visible) and the hovered-hex label left of
/// the charts sheet / peek tab. Runs in `Last`: the ledger
/// ([`ScreenLayout`](crate::ScreenLayout)) is reset in `First` and refilled by
/// the egui pass, so earlier in the frame it reads the empty default.
pub(crate) fn inset_bottom_panes(
    layout: Res<crate::ScreenLayout>,
    mut status: Query<&mut Node, (With<StatusPane>, Without<HexCoordPane>)>,
    mut coord: Query<&mut Node, (With<HexCoordPane>, Without<StatusPane>)>,
) {
    const MARGIN: f32 = 14.0;
    if let Ok(mut node) = status.single_mut() {
        let left = Val::Px(layout.left_inset + MARGIN);
        if node.left != left {
            node.left = left;
        }
    }
    if let Ok(mut node) = coord.single_mut() {
        let right = Val::Px(layout.right_inset + MARGIN);
        if node.right != right {
            node.right = right;
        }
    }
}
