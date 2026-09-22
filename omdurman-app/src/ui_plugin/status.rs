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
    room: Res<RoomId>,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: Peers,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let new = match state.get() {
        AppState::Splash => Cow::Borrowed(""),
        AppState::Lobby => Cow::Owned(format!(
            "Lobby -- choose your faction (share: ?room={})",
            room.as_str()
        )),
        // In game, the phase banner provides full turn/phase/sequence info.
        // The status line is a minimal complement showing server info.
        AppState::InGame => Cow::Owned(format!(
            "Room: {}  |  {}",
            room.as_str(),
            match game_state.as_deref() {
                Some(gs) => {
                    // The player who may act *now*: the turn owner except
                    // during defensive fire, where control passes to the
                    // non-moving side (§6.4/§6.7).
                    let acting = gs.0.phase_player();
                    let label = match acting {
                        omdurman_types::Player::AngloEgyptian => "Anglo-Egyptian",
                        omdurman_types::Player::Dervish => "Dervish",
                    };
                    if peers.may_act(acting) {
                        format!("You act now ({label})")
                    } else {
                        format!("Waiting on {label}")
                    }
                }
                None => "Setting up...".into(),
            },
        )),
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
