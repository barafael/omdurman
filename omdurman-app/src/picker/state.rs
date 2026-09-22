//! Selection state: what the player has selected, and read-helpers over it.

use super::*;

/// The selected unit's rules `UnitId` and hex, if it is engine-tracked.
///
/// Single-unit selections only: a movement group or a combat tile selection
/// is not a single action target, so it reports `None` — melee, retreat and
/// the action panel key off [`selected_unit_ids`] / [`selected_origin_hex`]
/// instead, which treat every selection shape as a set of actors.
pub fn selected_unit_id(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
) -> Option<(UnitId, HexCoord)> {
    let PickerState::Selected { source, .. } = state else {
        return None;
    };
    let (_, placed) = placed_units.get(*source).ok()?;
    Some((placed.unit_id?, placed.coord))
}

/// The rules `UnitId`s of every unit in the active selection — the single
/// selected counter, or all members of a movement group / combat tile — in
/// stable (entity-id) order, engine-tracked members only. Combat systems
/// (melee, retreat, advance-after-combat, the action panel) consume this so a
/// whole-tile selection acts with exactly the units the player selected.
pub fn selected_unit_ids(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
) -> Vec<UnitId> {
    let sources: &[Entity] = match state {
        PickerState::Selected { source, .. } => std::slice::from_ref(source),
        PickerState::SelectedStack(sel) => &sel.sources,
        PickerState::SelectedTile(sel) => &sel.sources,
        _ => &[],
    };
    let mut ids: Vec<UnitId> = sources
        .iter()
        .filter_map(|&e| placed_units.get(e).ok().and_then(|(_, p)| p.unit_id))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// The hex the active selection was made on, for any selection shape.
pub fn selected_origin_hex(state: &PickerState) -> Option<HexCoord> {
    match state {
        PickerState::Selected { start_coord, .. } => Some(*start_coord),
        PickerState::SelectedStack(sel) => Some(sel.start_coord),
        PickerState::SelectedTile(sel) => Some(sel.start_coord),
        _ => None,
    }
}

#[derive(Resource, Clone)]
pub struct UnitPicker {
    pub available: Vec<PickerUnit>,
    pub all: Vec<(SectionName, u32, u32, Handle<Image>, bool)>,
    /// When true (the default), placing a unit automatically selects the next
    /// available unit in the same section so the player can keep clicking to
    /// place multiples without returning to the picker panel.
    pub auto_place_next: bool,
}

impl Default for UnitPicker {
    fn default() -> Self {
        Self {
            available: Vec::new(),
            all: Vec::new(),
            auto_place_next: true,
        }
    }
}

impl UnitPicker {
    pub fn reset_available(&mut self) {
        self.available = self
            .all
            .iter()
            .map(|(sn, col, row, handle, is_boat)| PickerUnit {
                section_name: *sn,
                col: *col,
                row: *row,
                handle: handle.clone(),
                is_boat: *is_boat,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            })
            .collect();
    }
}

#[derive(Clone)]
pub struct PickerUnit {
    pub section_name: SectionName,
    pub col: u32,
    pub row: u32,
    pub handle: Handle<Image>,
    pub is_boat: bool,
    pub visible: bool,
    pub egui_texture: Option<egui::TextureHandle>,
    pub annotations_loaded: bool,
}

#[derive(Resource, Default, Clone)]
pub enum PickerState {
    #[default]
    Idle,
    Placing {
        unit_idx: usize,
        preview_hex: Option<HexCoord>,
        preview_valid: bool,
        drag_drop: bool,
    },
    /// A friendly unit has been selected.  Actions:
    /// * Left-click on reachable hex -> extend path annotation
    /// * Confirm (Enter / button) -> commit full path
    /// * Right-click -> deselect
    Selected {
        source: Entity,
        start_coord: HexCoord,
        remaining_mp: i16,
        /// Once a leg has entered an enemy ZOC (§5.43), the unit must stop and
        /// may not extend the path further this turn. The player can still
        /// commit the path built so far.
        forced_stop: bool,
    },
    /// A double-click in the movement phase selected *every* unit in a hex as
    /// one group. The whole group follows a single plotted path; units with
    /// less remaining movement drop off (stop) as soon as the next leg would
    /// exceed their budget, and on commit each unit is moved along the longest
    /// prefix of the path it can afford (§stack-move). Once the move is
    /// committed, the units are independent again.
    SelectedStack(StackSelection),
    /// A double-click in a *combat* phase (fire sub-phase or Melee), anywhere
    /// on a hex, selected the whole tile as one acting group — the unified
    /// combat selection. In a fire sub-phase the group is every firing unit
    /// of the hex (§6.14 combines them; §6.15 lets a smaller set fire
    /// instead — the engine's `build_fire_attack_from` takes any firer
    /// list); in Melee it is every melee-capable unit (the engine's
    /// `build_melee_attack` always gathers the co-stacked attackers, §7).
    /// A single click selects one counter instead. There is no plotted path:
    /// the group is the actor set for target rings, direction arrows,
    /// previews, and allocation. Clicking the group's own hex again
    /// dismisses it; right-click cancels.
    SelectedTile(TileSelection),
}

/// The group of units selected by a movement-phase double-click on their hex.
///
/// `sources` and the two movement vectors are parallel and kept in stable
/// (entity-id) order. While a leg is being plotted, only units whose remaining
/// movement covers the leg's cost are charged; a unit that can't afford the
/// next leg keeps its remaining budget (it has "dropped" and will stop at the
/// last affordable hex, recomputed from its budget at commit).
#[derive(Clone, PartialEq)]
pub struct StackSelection {
    /// Every selected unit, sharing one hex.
    pub sources: Vec<Entity>,
    /// The group's planned position: the start hex, advancing with each
    /// plotted leg.
    pub start_coord: HexCoord,
    /// Per-unit remaining movement this turn, parallel to `sources`.
    pub remaining_mp: Vec<i16>,
    /// Each unit's remaining movement when the stack was selected -- needed to
    /// refund a popped leg exactly on undo.
    pub initial_mp: Vec<i16>,
    /// Sticky once any plotted leg enters an enemy ZOC (§5.43): the group may
    /// not extend the path further this turn.
    pub forced_stop: bool,
}

/// The group selected by a combat-phase double-click on a hex — the unified
/// fire/melee selection: every unit of the hex that can act in the current
/// phase (fire: non-disrupted with a fire factor, §6.14; melee:
/// melee-capable, §7). No movement budgets — the group is the actor set for
/// target rings, direction arrows, previews, and allocation.
#[derive(Clone, PartialEq)]
pub struct TileSelection {
    pub sources: Vec<Entity>,
    pub start_coord: HexCoord,
}

/// Owned snapshot of the picker state driving a click or hotkey. Copied out
/// of `PickerState` before the match so the arms can re-borrow it mutably
/// (to hand `&mut` back into a [`SelectedClick`] / [`SelectedStackClick`]
/// or to build a replacement state) without aliasing the match scrutinee.
pub(crate) enum ActiveSelection {
    Idle,
    Placing {
        unit_idx: usize,
        drag_drop: bool,
    },
    Single {
        source: Entity,
        start_coord: HexCoord,
        remaining_mp: i16,
        forced_stop: bool,
    },
    Stack(StackSelection),
    Tile(TileSelection),
}

impl ActiveSelection {
    pub(crate) fn snapshot(state: &PickerState) -> ActiveSelection {
        match state {
            PickerState::Idle => ActiveSelection::Idle,
            PickerState::Placing {
                unit_idx,
                drag_drop,
                ..
            } => ActiveSelection::Placing {
                unit_idx: *unit_idx,
                drag_drop: *drag_drop,
            },
            PickerState::Selected {
                source,
                start_coord,
                remaining_mp,
                forced_stop,
            } => ActiveSelection::Single {
                source: *source,
                start_coord: *start_coord,
                remaining_mp: *remaining_mp,
                forced_stop: *forced_stop,
            },
            PickerState::SelectedStack(sel) => ActiveSelection::Stack(sel.clone()),
            PickerState::SelectedTile(sel) => ActiveSelection::Tile(sel.clone()),
        }
    }
}

/// Accumulated multi-leg movement path while a unit is selected.
///
/// Each leg is a `(from, to)` pair; the first leg's `from` is the unit's
/// original position.  Legs are accumulated locally but *not* committed
/// until the player confirms.  On confirm the full path is sent as a
/// single [`GameEvent::MoveUnit`] and the path is cleared.
///
/// The turn-path shadow (translucent mesh) is rendered from this resource
/// while it is populated, and persists after confirmation via
/// [`UnitPaths`].
#[derive(Resource, Default, Clone)]
pub struct MovementPath {
    pub legs: Vec<(HexCoord, HexCoord)>,
    pub cost_so_far: i16,
}

impl MovementPath {
    /// The final hex of the path (the unit's planned destination), or
    /// `None` if no legs have been added yet.
    pub fn current_end(&self) -> Option<HexCoord> {
        self.legs.last().map(|(_, to)| *to)
    }

    /// Clear the path (called on confirm, deselect, or new selection).
    pub fn reset(&mut self) {
        self.legs.clear();
        self.cost_so_far = 0;
    }
}

// -- Components -----------------------------------------------------------------

/// Marker present on the currently-selected unit entity. Allows ECS queries
/// like `Query<&PlacedUnit, With<Selected>>` without touching `PickerState`.
#[derive(Component)]
pub struct Selected;

/// Read-only bundle shared by the overview sidebar and the hover tooltip: the
/// picker state, the in-progress movement path, the hovered hex, the engine
/// state, and the placed-unit query -- everything both UIs read to describe
/// the board. Bundled so their signatures stay under clippy's argument limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PickerReadState<'w, 's> {
    pub picker_state: Res<'w, PickerState>,
    pub movement_path: Res<'w, MovementPath>,
    pub hovered: Res<'w, crate::HoveredHex>,
    pub game_state: Option<Res<'w, crate::GameStateResource>>,
    pub placed_units: Query<'w, 's, (Entity, &'static PlacedUnit)>,
}
