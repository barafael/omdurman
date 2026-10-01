//! The board click router: the *one* place that reads raw mouse buttons for
//! board interaction and decides which interaction mode owns a click.
//!
//! Before this module, six systems (picker, fire allocation, melee, advance
//! after combat, retreat, optional-rule river placement) each read
//! `ButtonInput<MouseButton>` themselves, and whether a click was "consumed"
//! depended on `.before(handle_picker_clicks)` ordering, shared `PickerState`
//! writes, and the engine happening to reject the click in all but one of
//! them. Now:
//!
//! 1. [`route_board_clicks`] (pointer-gated: `MapPointerInputSet`, so a click
//!    over egui never reaches the board) turns the frame's left press/release
//!    plus the board-plane hit into a hex, gathers the UI facts into a
//!    [`ClickCtx`], and asks the pure [`click_mode`] which [`ClickMode`] owns
//!    each edge.
//! 2. It emits exactly one mode message per edge — [`PickerClick`],
//!    [`FireClick`], [`MeleeClick`], [`AdvanceClick`], [`RetreatClick`] or
//!    [`RiverPlacementClick`] — which the former handlers now read instead of
//!    the mouse. Right-click is the existing [`PickerCommand::Cancel`].
//!
//! The handlers live in [`BoardClickHandlerSet`], ordered after the router,
//! so no handler-vs-handler ordering is needed to arbitrate a click.
//!
//! There is no separate touch-tap path today: board picking reads only
//! `PointerId::Mouse` (see `picking::update_pointer_ground_hit`), and touch
//! drives only the camera gestures. A future tap source should feed this
//! router (one more edge source) rather than a handler.

use bevy::ecs::message::{Message, MessageWriter};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use omdurman_hexmap::{HexLayout, hit_to_hex};
use omdurman_types::{HexCoord, Player};

use crate::GameStateResource;
use crate::hotkeys::PickerCommand;
use crate::picker::{PickerState, PlacedUnit, selected_unit_ids};
use crate::render::HexOverlay;
use crate::ui_phase_state::{PhaseKind, UiPhaseState};

/// Which edge of a left click a routing decision is for. Selection happens
/// on the press; placement, path plotting and every combat action on the
/// release.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClickEdge {
    #[default]
    Press,
    Release,
}

/// The interaction mode that owns one click edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickMode {
    /// Nobody acts (not this seat's turn, game over, a placement press).
    None,
    /// The unit picker's place / select / plot state machine.
    Picker,
    /// §6.41 fire allocation against the clicked hex.
    Fire,
    /// §7 melee declaration against the clicked hex.
    Melee,
    /// §6.82/§7.6 advance after combat into the clicked (vacated) hex.
    Advance,
    /// §7.5 retreat before melee to the clicked hex.
    Retreat,
    /// §10.11/§10.21 river mine / chain placement on the clicked Nile hex.
    RiverPlacement,
}

/// Everything [`click_mode`] decides on, gathered by [`route_board_clicks`]
/// from the UI state. Plain data so the decision is unit-testable.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClickCtx {
    pub edge: ClickEdge,
    /// The mirrored §4 machine, derived from the live engine state
    /// (`NoGame` when there is none).
    pub phase: UiPhaseState,
    /// `Peers::may_act(phase_player)`: the seat whose phase this is.
    pub may_act_now: bool,
    /// `Peers::may_act(active.opponent())`: the melee defender's seat.
    pub may_act_defender: bool,
    /// `Peers::may_act(Dervish)`: river mines / chain are Dervish-only.
    pub may_act_dervish: bool,
    /// A counter is in hand (`PickerState::Placing`).
    pub placing: bool,
    /// The optional-rule panel armed a mine or chain placement.
    pub river_placement_armed: bool,
    /// The fire allocations were already committed this sub-phase.
    pub fire_committed: bool,
    /// A declared melee awaits resolution (the §7.5 retreat window).
    pub pending_melee: bool,
    /// The selection holds a unit threatened by the pending infantry melee.
    pub selection_can_retreat: bool,
    /// The clicked hex holds a unit that may retreat before the pending
    /// melee (the defender selects it by clicking the threatened hex).
    pub hex_has_retreat_candidate: bool,
    /// A selected unit may advance after combat into the clicked hex.
    pub selection_can_advance_here: bool,
}

/// Decide which mode owns a click edge. The priority order is the whole
/// arbitration — read top to bottom:
///
/// 1. No engine state (headless tools): the picker, ungated.
/// 2. Game over: nothing.
/// 3. Setup: an armed river mine/chain placement owns the click for the
///    Dervish seat (acting on the release; the press is swallowed so the
///    picker neither focuses nor drops a counter under it). Otherwise the
///    picker. The click is not turn-gated here: the engine
///    (`GameState::require_setup_turn`) enforces the sequential set-up order.
/// 4. A counter in hand: the picker owns both edges in every phase
///    (reinforcement / FoK entry placement is not turn-gated).
/// 5. The §7.5 retreat window: on a Melee-phase release with a melee pending,
///    the defender's seat clicking the threatened hex selects the unit that
///    may retreat, and with it selected clicks a destination to retreat.
///    This sits above the turn gate because the defender is not the phase
///    player (so the picker would refuse the selection).
/// 6. Not this seat's phase: nothing.
/// 7. Movement: the picker (select on press, plot on release).
/// 8. Combat phases, press: the picker (single select / double-click tile).
/// 9. Combat phases, release — the combat target click:
///    * Offensive fire: advance after combat when a selected unit may enter
///      the (vacated) hex, else fire allocation (unless committed).
///    * Defensive fire: fire allocation (unless committed); §6.7 forbids
///      advancing after defensive fire.
///    * Melee: advance after combat into a vacated hex, else a melee
///      declaration when none is pending.
///
///    Advance targets vacated hexes and fire / melee target occupied ones,
///    so checking advance first only settles the empty-hex case (where the
///    old parallel handlers both ran and fire merely refused).
pub fn click_mode(ctx: &ClickCtx) -> ClickMode {
    let release = ctx.edge == ClickEdge::Release;
    let phase = match ctx.phase {
        UiPhaseState::NoGame => return ClickMode::Picker,
        UiPhaseState::GameOver => return ClickMode::None,
        UiPhaseState::Setup => {
            return if ctx.river_placement_armed && ctx.may_act_dervish {
                if release {
                    ClickMode::RiverPlacement
                } else {
                    ClickMode::None
                }
            } else {
                ClickMode::Picker
            };
        }
        UiPhaseState::Turn { phase, .. } => phase,
    };
    if ctx.placing {
        return ClickMode::Picker;
    }
    if release
        && phase == PhaseKind::Melee
        && ctx.pending_melee
        && ctx.may_act_defender
        && (ctx.selection_can_retreat || ctx.hex_has_retreat_candidate)
    {
        return ClickMode::Retreat;
    }
    if !ctx.may_act_now {
        return ClickMode::None;
    }
    if phase == PhaseKind::Movement || !release {
        return ClickMode::Picker;
    }
    match phase {
        PhaseKind::Movement => ClickMode::Picker,
        PhaseKind::OffensiveFire(_) if ctx.selection_can_advance_here => ClickMode::Advance,
        PhaseKind::OffensiveFire(_) | PhaseKind::DefensiveFire(_) if !ctx.fire_committed => {
            ClickMode::Fire
        }
        PhaseKind::OffensiveFire(_) | PhaseKind::DefensiveFire(_) => ClickMode::None,
        PhaseKind::Melee if ctx.selection_can_advance_here => ClickMode::Advance,
        PhaseKind::Melee if !ctx.pending_melee => ClickMode::Melee,
        PhaseKind::Melee => ClickMode::None,
    }
}

/// A board click routed to the unit picker. Carries both edges because a
/// click can press and release within one frame, which the picker's state
/// machine handles as one step.
#[derive(Message, Clone, Copy, Debug)]
pub struct PickerClick {
    pub hex: HexCoord,
    /// World-space board-plane hit (picks the counter within a fanned stack).
    pub hit: Vec3,
    pub pressed: bool,
    pub released: bool,
}

/// A fire-allocation target click (§6.41).
#[derive(Message, Clone, Copy, Debug)]
pub struct FireClick(pub HexCoord);

/// A melee-declaration target click (§7).
#[derive(Message, Clone, Copy, Debug)]
pub struct MeleeClick(pub HexCoord);

/// An advance-after-combat destination click (§6.82/§7.6).
#[derive(Message, Clone, Copy, Debug)]
pub struct AdvanceClick(pub HexCoord);

/// A retreat-before-melee destination click (§7.5).
#[derive(Message, Clone, Copy, Debug)]
pub struct RetreatClick(pub HexCoord);

/// A river mine / chain placement click (§10.11/§10.21).
#[derive(Message, Clone, Copy, Debug)]
pub struct RiverPlacementClick(pub HexCoord);

/// Every board-click consumer. Ordered after [`route_board_clicks`], so each
/// sees this frame's click; nothing inside needs ordering against a sibling.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoardClickHandlerSet;

/// Raw pointer input: the mouse buttons and the board-plane hit.
#[derive(SystemParam)]
pub struct BoardPointer<'w> {
    buttons: Res<'w, ButtonInput<MouseButton>>,
    ground: Res<'w, crate::picking::PointerGroundHit>,
    layout: Res<'w, HexLayout>,
    overlay: Res<'w, HexOverlay>,
}

/// The UI state the mode decision reads.
#[derive(SystemParam)]
pub struct ClickUiState<'w, 's> {
    game_state: Option<Res<'w, GameStateResource>>,
    picker: Res<'w, PickerState>,
    placed_units: Query<'w, 's, (Entity, &'static PlacedUnit)>,
    peers: crate::peers::Peers<'w, 's>,
    fire: Option<Res<'w, crate::fire_allocation::FireAllocationState>>,
    river: Option<Res<'w, crate::ui_plugin::OptionalRulePlacement>>,
}

impl ClickUiState<'_, '_> {
    fn ctx(&self, hex: HexCoord) -> ClickCtx {
        let placing = matches!(*self.picker, PickerState::Placing { .. });
        let river_placement_armed = self
            .river
            .as_ref()
            .is_some_and(|r| r.placing_mine || r.placing_chain);
        let fire_committed = self.fire.as_ref().is_some_and(|f| f.committed);
        let may_act_dervish = self.peers.may_act(Player::Dervish);
        let Some(gs) = self.game_state.as_deref() else {
            return ClickCtx {
                placing,
                river_placement_armed,
                fire_committed,
                may_act_dervish,
                ..Default::default()
            };
        };
        let gs = &gs.0;
        let phase = UiPhaseState::derive(gs);
        let combat = matches!(
            phase,
            UiPhaseState::Turn {
                phase: PhaseKind::OffensiveFire(_) | PhaseKind::Melee,
                ..
            }
        );
        let melee = matches!(
            phase,
            UiPhaseState::Turn {
                phase: PhaseKind::Melee,
                ..
            }
        );
        ClickCtx {
            edge: ClickEdge::Press,
            phase,
            may_act_now: self.peers.may_act_now(gs),
            may_act_defender: self.peers.may_act(gs.active_player.opponent()),
            may_act_dervish,
            placing,
            river_placement_armed,
            fire_committed,
            pending_melee: gs.pending_melee.is_some(),
            selection_can_retreat: melee
                && crate::retreat::selected_threatened_unit(&self.picker, &self.placed_units, gs)
                    .is_some(),
            hex_has_retreat_candidate: melee
                && crate::retreat::retreat_candidate_at(gs, hex).is_some(),
            selection_can_advance_here: combat
                && selected_unit_ids(&self.picker, &self.placed_units)
                    .into_iter()
                    .any(|id| gs.can_advance_after_combat(id, hex).is_ok()),
        }
    }
}

/// One writer per mode message.
#[derive(SystemParam)]
pub struct BoardClickWriters<'w> {
    cancel: MessageWriter<'w, PickerCommand>,
    picker: MessageWriter<'w, PickerClick>,
    fire: MessageWriter<'w, FireClick>,
    melee: MessageWriter<'w, MeleeClick>,
    advance: MessageWriter<'w, AdvanceClick>,
    retreat: MessageWriter<'w, RetreatClick>,
    river: MessageWriter<'w, RiverPlacementClick>,
}

/// The board click router (see the module docs). Runs in `GameSet` and the
/// pointer-gated `MapPointerInputSet`.
pub fn route_board_clicks(pointer: BoardPointer, ui: ClickUiState, mut out: BoardClickWriters) {
    // Right-click on the board means Cancel (same as Esc). The Cancel
    // handler itself is not pointer-gated, so the actions-panel button
    // (over UI) reaches it too.
    if pointer.buttons.just_pressed(MouseButton::Right) {
        out.cancel.write(PickerCommand::Cancel);
    }

    let pressed = pointer.buttons.just_pressed(MouseButton::Left);
    let released = pointer.buttons.just_released(MouseButton::Left);
    if !pressed && !released {
        return;
    }
    // `None` over UI, off the board, or unseen by any camera.
    let Some(hit) = **pointer.ground else {
        return;
    };
    let origin = pointer.layout.adjusted_origin(&pointer.overlay.params);
    let hex = hit_to_hex(hit, origin, &pointer.overlay.params);

    let game_state = ui.game_state.as_deref();
    // UI trace: every board press, with what stood under the cursor, so
    // click-through and rejected clicks are observable whichever mode (if
    // any) takes the click.
    if pressed {
        let units: Vec<String> = ui
            .placed_units
            .iter()
            .filter(|(_, u)| u.coord == hex)
            .map(|(_, u)| crate::ui_trace::placed_label(u, game_state))
            .collect();
        crate::ui_trace::board_click(
            crate::ui_trace::HexLabel::of(hex),
            &units,
            &crate::ui_trace::Stamp::of(game_state),
        );
    }

    let ctx = ui.ctx(hex);
    let mode_of = |edge| click_mode(&ClickCtx { edge, ..ctx });
    let press_mode = pressed.then(|| mode_of(ClickEdge::Press));
    let release_mode = released.then(|| mode_of(ClickEdge::Release));

    let picker_press = press_mode == Some(ClickMode::Picker);
    let picker_release = release_mode == Some(ClickMode::Picker);
    if picker_press || picker_release {
        out.picker.write(PickerClick {
            hex,
            hit,
            pressed: picker_press,
            released: picker_release,
        });
    }
    for mode in [press_mode, release_mode].into_iter().flatten() {
        match mode {
            ClickMode::None | ClickMode::Picker => {}
            ClickMode::Fire => {
                out.fire.write(FireClick(hex));
            }
            ClickMode::Melee => {
                out.melee.write(MeleeClick(hex));
            }
            ClickMode::Advance => {
                out.advance.write(AdvanceClick(hex));
            }
            ClickMode::Retreat => {
                out.retreat.write(RetreatClick(hex));
            }
            ClickMode::RiverPlacement => {
                out.river.write(RiverPlacementClick(hex));
            }
        }
    }
}

/// Registers the click messages, the router, and the handler-set ordering.
pub struct BoardClickPlugin;

impl Plugin for BoardClickPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PickerCommand>()
            .add_message::<PickerClick>()
            .add_message::<FireClick>()
            .add_message::<MeleeClick>()
            .add_message::<AdvanceClick>()
            .add_message::<RetreatClick>()
            .add_message::<RiverPlacementClick>()
            .configure_sets(Update, BoardClickHandlerSet.after(route_board_clicks))
            .add_systems(
                Update,
                route_board_clicks
                    .in_set(crate::GameSet)
                    .in_set(crate::ui_plugin::MapPointerInputSet)
                    .before(crate::picker::cancel_placement),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_phase_state::FireSubKind;

    fn turn(phase: PhaseKind) -> UiPhaseState {
        UiPhaseState::Turn {
            active: Player::AngloEgyptian,
            night: false,
            phase,
        }
    }

    /// A seat that may act in the current phase, nothing else set.
    fn my_turn(phase: UiPhaseState, edge: ClickEdge) -> ClickCtx {
        ClickCtx {
            edge,
            phase,
            may_act_now: true,
            ..Default::default()
        }
    }

    const OFF_FIRE: PhaseKind = PhaseKind::OffensiveFire(FireSubKind::Direct);
    const DEF_FIRE: PhaseKind = PhaseKind::DefensiveFire(FireSubKind::Direct);

    #[test]
    fn setup_placement_goes_to_the_picker_on_both_edges() {
        for edge in [ClickEdge::Press, ClickEdge::Release] {
            let ctx = ClickCtx {
                edge,
                phase: UiPhaseState::Setup,
                placing: true,
                ..Default::default()
            };
            assert_eq!(click_mode(&ctx), ClickMode::Picker, "{edge:?}");
        }
    }

    #[test]
    fn movement_selection_and_plotting_go_to_the_picker() {
        for edge in [ClickEdge::Press, ClickEdge::Release] {
            assert_eq!(
                click_mode(&my_turn(turn(PhaseKind::Movement), edge)),
                ClickMode::Picker
            );
        }
    }

    #[test]
    fn fire_phase_press_selects_and_release_allocates() {
        for phase in [OFF_FIRE, DEF_FIRE] {
            assert_eq!(
                click_mode(&my_turn(turn(phase), ClickEdge::Press)),
                ClickMode::Picker
            );
            assert_eq!(
                click_mode(&my_turn(turn(phase), ClickEdge::Release)),
                ClickMode::Fire
            );
            let committed = ClickCtx {
                fire_committed: true,
                ..my_turn(turn(phase), ClickEdge::Release)
            };
            assert_eq!(click_mode(&committed), ClickMode::None);
        }
    }

    #[test]
    fn melee_release_declares_when_nothing_is_pending() {
        assert_eq!(
            click_mode(&my_turn(turn(PhaseKind::Melee), ClickEdge::Release)),
            ClickMode::Melee
        );
        let pending = ClickCtx {
            pending_melee: true,
            ..my_turn(turn(PhaseKind::Melee), ClickEdge::Release)
        };
        assert_eq!(click_mode(&pending), ClickMode::None, "one melee at a time");
    }

    #[test]
    fn retreat_window_belongs_to_the_defender() {
        // Bound game: the defender is not the phase player.
        let ctx = ClickCtx {
            edge: ClickEdge::Release,
            phase: turn(PhaseKind::Melee),
            may_act_defender: true,
            pending_melee: true,
            selection_can_retreat: true,
            ..Default::default()
        };
        assert_eq!(click_mode(&ctx), ClickMode::Retreat);
        // Without a threatened unit selected there is nothing to retreat.
        let no_sel = ClickCtx {
            selection_can_retreat: false,
            ..ctx
        };
        assert_eq!(click_mode(&no_sel), ClickMode::None);
        // Clicking the threatened hex selects the retreating unit — the
        // defender cannot select through the picker (not the phase player).
        let select = ClickCtx {
            selection_can_retreat: false,
            hex_has_retreat_candidate: true,
            ..ctx
        };
        assert_eq!(click_mode(&select), ClickMode::Retreat);
        // No pending melee: no retreat window.
        let closed = ClickCtx {
            pending_melee: false,
            ..ctx
        };
        assert_eq!(click_mode(&closed), ClickMode::None);
    }

    #[test]
    fn advance_window_takes_vacated_hex_clicks() {
        for phase in [OFF_FIRE, PhaseKind::Melee] {
            let ctx = ClickCtx {
                selection_can_advance_here: true,
                ..my_turn(turn(phase), ClickEdge::Release)
            };
            assert_eq!(click_mode(&ctx), ClickMode::Advance, "{phase:?}");
        }
        // §6.7: never after defensive fire -- the click stays a fire click.
        let def = ClickCtx {
            selection_can_advance_here: true,
            ..my_turn(turn(DEF_FIRE), ClickEdge::Release)
        };
        assert_eq!(click_mode(&def), ClickMode::Fire);
    }

    #[test]
    fn optional_rule_placement_is_dervish_only() {
        let armed = ClickCtx {
            edge: ClickEdge::Release,
            phase: UiPhaseState::Setup,
            river_placement_armed: true,
            may_act_dervish: true,
            ..Default::default()
        };
        assert_eq!(click_mode(&armed), ClickMode::RiverPlacement);
        // The press is swallowed so the picker doesn't act under it.
        let press = ClickCtx {
            edge: ClickEdge::Press,
            ..armed
        };
        assert_eq!(click_mode(&press), ClickMode::None);
        // Not the Dervish seat: an ordinary setup click.
        let ae = ClickCtx {
            may_act_dervish: false,
            ..armed
        };
        assert_eq!(click_mode(&ae), ClickMode::Picker);
    }

    #[test]
    fn not_my_turn_does_nothing() {
        for phase in [PhaseKind::Movement, OFF_FIRE, DEF_FIRE, PhaseKind::Melee] {
            for edge in [ClickEdge::Press, ClickEdge::Release] {
                let ctx = ClickCtx {
                    edge,
                    phase: turn(phase),
                    selection_can_advance_here: true,
                    ..Default::default()
                };
                assert_eq!(click_mode(&ctx), ClickMode::None, "{phase:?} {edge:?}");
            }
        }
    }

    #[test]
    fn a_counter_in_hand_is_placed_in_any_phase() {
        let ctx = ClickCtx {
            edge: ClickEdge::Release,
            phase: turn(PhaseKind::Melee),
            placing: true,
            ..Default::default()
        };
        assert_eq!(click_mode(&ctx), ClickMode::Picker);
    }

    #[test]
    fn game_over_ignores_clicks() {
        let ctx = my_turn(UiPhaseState::GameOver, ClickEdge::Release);
        assert_eq!(click_mode(&ctx), ClickMode::None);
    }

    // -- the router system, headless ------------------------------------------

    fn router_app(gs: omdurman_rules::effects::GameState) -> App {
        let mut app = App::new();
        app.add_message::<PickerCommand>()
            .add_message::<PickerClick>()
            .add_message::<FireClick>()
            .add_message::<MeleeClick>()
            .add_message::<AdvanceClick>()
            .add_message::<RetreatClick>()
            .add_message::<RiverPlacementClick>()
            .insert_resource(ButtonInput::<MouseButton>::default())
            .insert_resource(crate::picking::PointerGroundHit(Some(Vec3::ZERO)))
            .insert_resource(HexOverlay::default())
            .insert_resource(omdurman_board_ui::board_store::default_layout())
            .insert_resource(PickerState::default())
            .insert_resource(crate::peers::LocalPeer::default())
            .insert_resource(crate::seats::Seats::default())
            .insert_resource(crate::seats::SeatPresence::default())
            .insert_resource(crate::seats::LocalPlayerKey(omdurman_net::PlayerKey(1)))
            .insert_resource(crate::fire_allocation::FireAllocationState::default())
            .insert_resource(crate::ui_plugin::OptionalRulePlacement::default())
            .insert_resource(GameStateResource(gs))
            .add_systems(Update, route_board_clicks);
        app
    }

    fn count<M: Message>(app: &App) -> usize {
        app.world()
            .resource::<bevy::ecs::message::Messages<M>>()
            .len()
    }

    #[test]
    fn router_sends_a_fire_phase_release_only_to_fire() {
        let mut gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        gs.phase = omdurman_rules::Phase::OffensiveFire(omdurman_rules::FireSubPhase::DirectFire);
        let mut app = router_app(gs);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(count::<PickerClick>(&app), 1, "the press selects");
        assert_eq!(count::<FireClick>(&app), 0);

        let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        buttons.clear();
        buttons.release(MouseButton::Left);
        app.update();
        assert_eq!(count::<FireClick>(&app), 1, "the release allocates");
        assert_eq!(count::<AdvanceClick>(&app), 0);
    }

    #[test]
    fn router_turns_right_click_into_cancel() {
        let gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        let mut app = router_app(gs);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.update();
        assert_eq!(count::<PickerCommand>(&app), 1);
        assert_eq!(count::<PickerClick>(&app), 0);
    }
}
