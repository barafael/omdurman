//! The left picker sidebar: sprite assets, the unit grid, and tooltips.

use super::*;

/// Bucket a sprite-sheet cell `(filename, col, row)` into its section.
///
/// Matches the *complete* cell name (`Hadendowa_Forts_0_0`) rather than a
/// prefix: a prefix match would swallow the `Hadendowa_Forts` block into
/// `Hadendowa` (both names start with `Hadendowa_`), silently dropping the
/// fort counters from the picker and with them the auto-setup North Fort
/// placement (§9.344).
fn bucket_section(
    order: &[SectionName],
    filename: &str,
    col: u32,
    row: u32,
) -> Option<SectionName> {
    order
        .iter()
        .find(|s| format!("{}_{}_{}", s, col, row) == filename)
        .copied()
}

pub fn spawn_picker_assets(mut picker: ResMut<UnitPicker>, asset_server: Res<AssetServer>) {
    let order = section_order();

    let mut section_sprites: Vec<Vec<PickerUnit>> = order.iter().map(|_| Vec::new()).collect();

    for &(filename, col, row) in generated::SPRITE_PATHS {
        if let Some(section_name) = bucket_section(order, filename, col, row) {
            let idx = order.iter().position(|s| *s == section_name).unwrap();
            let path = format!("sprites/{}.webp", filename);
            let handle = asset_server.load(&path);
            section_sprites[idx].push(PickerUnit {
                section_name,
                col,
                row,
                handle,
                is_boat: false,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            });
        }
    }

    for sprites in section_sprites {
        for sprite in sprites {
            picker.all.push((
                sprite.section_name,
                sprite.col,
                sprite.row,
                sprite.handle.clone(),
                sprite.is_boat,
            ));
            picker.available.push(sprite);
        }
    }
}

// -- Left sidebar: list available units -----------------------------------------

fn load_egui_texture(
    ctx: &egui::Context,
    image: &Image,
    label: &str,
) -> Option<egui::TextureHandle> {
    let w = image.width() as usize;
    let h = image.height() as usize;
    if w == 0 || h == 0 {
        return None;
    }
    let data = image.data.as_ref()?;
    if data.len() < w * h * 4 {
        return None;
    }
    let pixels: Vec<egui::Color32> = data
        .chunks(4)
        .take(w * h)
        // Data-driven: the sprite image's own RGBA pixels.
        .map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
        .collect();
    if pixels.len() != w * h {
        return None;
    }
    let color_image = egui::ColorImage {
        size: [w, h],
        pixels,
        source_size: egui::vec2(w as f32, h as f32),
    };
    Some(ctx.load_texture(label, color_image, egui::TextureOptions::LINEAR))
}

/// Render the visible picker units belonging to `faction`, grouped by section
/// with a label per section and a wrapped grid of sprite cells. Records a click
/// or drag-start into `clicked_idx` / `drag_idx` (an index into
/// `picker.available`). Shared by both faction categories in the picker.
///
/// When `annotations` is `Some`, each sprite cell gains a hover tooltip
/// showing the counter's resolved profile -- identity (e.g. "1B 1st Btn"),
/// fire/melee/movement factors, weapon class, and the rulebook paragraph for
/// its section. The tooltip is informational; clicking still picks the unit.
/// Bundle of `&UnitPicker` + `&PickerState` so [`render_faction_units`] stays
/// under clippy's argument limit. Plain struct (the consumer is not a system).
struct PickerRead<'a> {
    picker: &'a UnitPicker,
    state: &'a PickerState,
}

/// Bundle of the `clicked_idx` + `drag_idx` + `drag_cancelled`
/// out-parameters so [`render_faction_units`] stays under clippy's argument
/// limit.
struct DragState<'a> {
    clicked_idx: &'a mut Option<usize>,
    drag_idx: &'a mut Option<usize>,
    /// A counter's drag ended over the UI (the sidebar, a card) instead of
    /// the board: no drop happened.
    drag_cancelled: &'a mut bool,
}

/// The picker state after a sidebar drag ends over the UI rather than the
/// board: the board click router never sees that release, so a drag-held
/// counter would otherwise stay "in hand" and the next click on it would
/// toggle it off instead of picking it. Drop it: the following click picks
/// the counter afresh. `None` leaves the state alone (no drag in hand).
fn state_after_cancelled_drag(state: &PickerState) -> Option<PickerState> {
    matches!(
        state,
        PickerState::Placing {
            drag_drop: true,
            ..
        }
    )
    .then_some(PickerState::Idle)
}

/// Bundle of the optional annotations + the rulebook reference so
/// [`render_faction_units`] stays under clippy's argument limit.
struct UnitAnnotations<'a> {
    rulebook: &'a crate::rulebook::Rulebook,
}

/// Bundle of the image assets + sprite annotations + rulebook reference
/// consumed by [`unit_picker_ui`], so the system stays under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PickerAssetCtx<'w> {
    pub images: Res<'w, Assets<Image>>,
    pub annotations: Option<Res<'w, SpriteAnnotationsResource>>,
    pub rulebook: Res<'w, crate::rulebook::Rulebook>,
}

fn render_faction_units(
    ui: &mut egui::Ui,
    picker: PickerRead,
    faction: omdurman_types::Player,
    cell_size: f32,
    sprite_size: f32,
    drag: DragState,
    ctx: UnitAnnotations,
) {
    let PickerRead { picker, state } = picker;
    let DragState {
        clicked_idx,
        drag_idx,
        drag_cancelled,
    } = drag;
    let UnitAnnotations { rulebook } = ctx;
    let mut current_section = None::<SectionName>;
    for idx in 0..picker.available.len() {
        if !picker.available[idx].visible {
            continue;
        }
        let section_name = picker.available[idx].section_name;
        if omdurman_rules::unit_profiles::section_owner(section_name) != Some(faction) {
            continue;
        }
        if Some(section_name) != current_section {
            current_section = Some(section_name);
            // Count how many counters in this section remain unplaced, so the
            // player can track deployment progress per block (e.g. "32×
            // Mulazmin") rather than only watching the tray empty.
            let remaining = picker
                .available
                .iter()
                .filter(|u| u.visible && u.section_name == section_name)
                .count();
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(section_name.display_name())
                        .size(13.0)
                        .color(crate::ui::palette::TEXT_SOFT),
                );
                ui.label(
                    egui::RichText::new(format!("({remaining})"))
                        .size(11.0)
                        .color(crate::ui::palette::TEXT_FAINT),
                );
            });
            ui.add_space(2.0);

            ui.horizontal_wrapped(|ui| {
                for j in idx..picker.available.len() {
                    if Some(picker.available[j].section_name) != current_section {
                        break;
                    }
                    if !picker.available[j].visible {
                        continue;
                    }
                    let is_selected =
                        matches!(&*state, PickerState::Placing { unit_idx, .. } if *unit_idx == j);
                    let unit = &picker.available[j];

                    let (rect, response) = ui.allocate_exact_size(
                        egui::Vec2::new(cell_size, cell_size),
                        egui::Sense::click_and_drag(),
                    );

                    let bg = if is_selected {
                        crate::ui::palette::theme::SELECTION_BG
                    } else if response.hovered() {
                        crate::ui::palette::theme::WIDGET_HOVER
                    } else {
                        crate::ui::palette::NEUTRAL_FILL
                    };
                    let painter = ui.painter();
                    painter.rect_filled(rect, 3.0, bg);

                    if let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id()) {
                        let img_rect = egui::Rect::from_center_size(
                            rect.center(),
                            egui::Vec2::new(sprite_size, sprite_size),
                        );
                        painter.image(
                            tex_id,
                            img_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    } else {
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}x{}", unit.col, unit.row),
                            egui::FontId::proportional(10.0),
                            crate::ui::palette::TEXT_FAINT,
                        );
                    }

                    if response.clicked() {
                        *clicked_idx = Some(j);
                    }
                    if response.drag_started() {
                        *drag_idx = Some(j);
                    }
                    if response.drag_stopped() && ui.ctx().is_pointer_over_egui() {
                        *drag_cancelled = true;
                    }

                    // Hover tooltip: the counter's resolved profile, sourced
                    // from the compiled annotations data + the rules engine's
                    // section classifier. Plain text (egui tooltips are
                    // non-interactive by default), with the rulebook citation
                    // rendered as a titled reference via `Rulebook::title_of`
                    // -- the player sees "§2.32 Anglo-Egyptian weapon types"
                    // rather than a bare section number.
                    let unit_id =
                        unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8);
                    let profile = unit_id.and_then(omdurman_rules::unit_profiles::profile_for_unit);
                    response.on_hover_ui(|ui| {
                        draw_picker_tooltip(
                            ui,
                            unit.section_name,
                            unit.col,
                            unit.row,
                            unit_id,
                            profile.as_ref(),
                            rulebook,
                        );
                    });
                }
            });
        }
    }
}

/// Render the hover tooltip for one picker sprite. Plain text + a titled §
/// reference; not interactive (egui's `on_hover_ui` tooltip closes on cursor
/// exit, so deep-links would be fiddly -- the rulebook tab is one click away
/// via the chart sheet for players who want to read more).
fn draw_picker_tooltip(
    ui: &mut egui::Ui,
    section_name: SectionName,
    col: u32,
    row: u32,
    unit_id: Option<UnitId>,
    profile: Option<&omdurman_rules::UnitProfile>,
    rulebook: &crate::rulebook::Rulebook,
) {
    ui.set_max_width(240.0);
    // Identity header.
    let identity_str = if let Some(p) = profile {
        p.identity.short_label()
    } else {
        format!("{} ({}x{})", section_name.display_name(), col, row)
    };
    ui.label(
        egui::RichText::new(identity_str)
            .color(crate::ui::palette::INK)
            .strong()
            .size(13.0),
    );

    // Factors + weapon + movement.
    if let Some(p) = profile {
        let factors = format!(
            "fire {}  ·  melee {}  ·  {}",
            p.fire.map(|f| f.value().to_string()).unwrap_or("—".into()),
            p.melee.map(|m| m.value().to_string()).unwrap_or("—".into()),
            movement_short(&p.movement),
        );
        ui.colored_label(crate::ui::palette::FAINT_INK, factors);
        ui.colored_label(
            crate::ui::palette::FAINT_INK,
            format!("weapon: {}", p.weapon),
        );
        // Printed counter text (e.g. "1B", "Khalifa") and the second-fire
        // flag -- facts the rules profile doesn't carry but the player can
        // see on the counter itself.
        let text = unit_id.map(UnitId::text).unwrap_or("");
        if !text.is_empty() {
            ui.colored_label(crate::ui::palette::FAINT_INK, format!("“{text}”"));
        }
        if unit_id.is_some_and(|id| id.kind().is_some_and(|k| k.fires_twice())) {
            crate::rulebook::refs_label(
                ui,
                "fires twice per phase (§6.42)",
                crate::ui::palette::FAINT_INK,
                12.0,
            );
        }
    } else {
        ui.colored_label(
            crate::ui::palette::FAINT_INK,
            "no profile resolved for this counter",
        );
    }

    // Rulebook citation for the section, annotated with its title.
    let paragraph = section_paragraph(section_name);
    let title = rulebook.title_of(paragraph);
    let citation = if let Some(t) = title {
        format!("§{paragraph} {t}")
    } else {
        format!("§{paragraph}")
    };
    ui.add_space(2.0);
    crate::rulebook::refs_label(ui, &citation, crate::ui::palette::FAINT_INK, 12.0);
}

fn movement_short(m: &omdurman_rules::UnitMovement) -> String {
    match m {
        omdurman_rules::UnitMovement::Land(a) => format!("move {}", a.value()),
        omdurman_rules::UnitMovement::Gunboat(g) => {
            format!("gunboat {}↑/{}↓", g.upstream.value(), g.downstream.value())
        }
        omdurman_rules::UnitMovement::Immobile => "immobile".into(),
    }
}

/// The rulebook section that documents a sprite-sheet section. Used by the
/// picker tooltip to deep-link the player to the right paragraph for the
/// counter they're hovering.
fn section_paragraph(section_name: SectionName) -> &'static str {
    use SectionName::*;
    match section_name {
        // Dervish leaders and tribes (§2.31).
        KhalifaAbdullah | Sherif | AliWadHelu | SheikElDin | Yakub | OsmanDigna => "2.31",
        Taiasha | Hadendowa | Baggara | Jehadia | Mulazmin | Kehena | Degheim | Danagla
        | JaalinI | JaalinII => "2.31",
        HadendowaForts => "2.31",
        // Anglo-Egyptian units (§2.32).
        BritishArmy | EgyptianArmy | Kitchener | BritishBoats => "2.32",
        MulazminI | MulazminII => "2.32",
        // FALL OF KHARTOUM's Forts Makran and Buri: the fort rules.
        BritishForts => "6.54",
    }
}

#[allow(clippy::too_many_arguments)]
pub fn unit_picker_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    mut picker_ctx: PickerContext,
    peers: crate::peers::Peers,
    assets: PickerAssetCtx,
    game_state: Option<Res<crate::GameStateResource>>,
    mut was_game_started: Local<bool>,
    mut layout: ResMut<crate::ScreenLayout>,
) {
    let PickerAssetCtx {
        images,
        annotations,
        rulebook,
    } = assets;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_play() {
        return;
    }
    // Spectators have no units to place -- hide the picker entirely so they
    // can't enter a placement (the click handler also rejects it defensively).
    if peers.is_spectator() {
        return;
    }

    // -- cache egui textures & look up is_boat from annotations --
    for unit in &mut picker_ctx.picker.available {
        if unit.egui_texture.is_none()
            && let Some(image) = images.get(&unit.handle)
        {
            let label = format!("picker_{}_{}_{}", unit.section_name, unit.col, unit.row);
            unit.egui_texture = load_egui_texture(ctx, image, &label);
        }
        if !unit.annotations_loaded {
            if (!unit.is_boat || unit.visible)
                && let Some(ref ann) = annotations
            {
                let entry = ann
                    .0
                    .get(&unit.section_name)
                    .and_then(|m| m.get(&(unit.col, unit.row)));
                if let Some(a) = entry {
                    if a.is_boat() {
                        unit.is_boat = true;
                    }
                    if !a.is_unit() {
                        unit.visible = false;
                    }
                }
            }
            // Fallback to compiled sprite data when no annotation entry exists
            // for this position. Hide non-placeable cells -- turn counters,
            // section labels, §6.63 wall-breach markers, bare colour counters
            // -- so they never appear in the picker (and especially not during
            // setup). A cell is placeable iff it resolves to a unit profile;
            // Marker / Breech / BareCounter cells all resolve to `None`.
            if unit.visible {
                let placeable =
                    unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                        .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                        .is_some();
                if !placeable {
                    unit.visible = false;
                }
            }
            unit.annotations_loaded = true;
        }
    }

    // -- scenario-based visibility filter --
    // Hide units whose section is not part of the active scenario's order of
    // battle, and hide named gunboats in FoK (§9.321 — only old gunboats).
    if let Some(state) = game_state.as_deref() {
        if let Some(allowed) = state.0.scenario.sections_for_picker() {
            for unit in &mut picker_ctx.picker.available {
                if !allowed.contains(&unit.section_name) {
                    unit.visible = false;
                }
            }
        }
        if matches!(state.0.scenario, Scenario::Historical) {
            // §9.211/§9.212: GORDON, the Friendlies, Isa Zachneih, the
            // gunboats and the forts sit this battle out.
            for unit in picker_ctx.picker.available.iter_mut().filter(|u| u.visible) {
                unit.visible =
                    unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                        .is_some_and(omdurman_rules::effects::historical_counter_in_play);
            }
        }
        if matches!(state.0.scenario, Scenario::Campaign) {
            // §9.113: "The GORDON unit is not used in this scenario."
            for unit in picker_ctx.picker.available.iter_mut().filter(|u| u.visible) {
                unit.visible =
                    unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                        .is_some_and(omdurman_rules::effects::campaign_counter_in_play);
            }
        }
        if matches!(state.0.scenario, Scenario::FallOfKhartoum) {
            use omdurman_rules::effects::fok_cap_group;
            // §9.321/§9.322: the FoK order of battle is exactly the set of
            // identities covered by `fok_cap_group`. Hide every picker counter
            // whose identity is *not* in that table -- cavalry, engineers,
            // Maxims, Dervish leaders, Dervish gunboats, named gunboats, Isa
            // Zachneih, etc. This subsumes the old named-gunboat filter. The
            // Ali_Wad_Helu block's Kehena/Degheim "Deghelim" counters resolve
            // to those tribes (see `unit_profiles::ali_wad_helu`), so they
            // stay in the order of battle while the block's leader is hidden.
            for unit in &mut picker_ctx.picker.available {
                if !unit.visible {
                    continue;
                }
                let in_oob =
                    unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                        .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                        .is_some_and(|p| fok_cap_group(&p.identity).is_some());
                if !in_oob {
                    unit.visible = false;
                }
            }
            // Hide excess counters once the OOB per-group cap is reached
            // (§9.321/§9.322). E.g. only 2 Hadendowa and 2 old gunboats
            // exist in FoK even though the sheets carry more counters.
            // (group, cap, placed_count, kept_count)
            let mut groups: Vec<(FokCapGroup, usize, usize, usize)> = Vec::new();
            // Seed every visible counter's group at placed = 0 so caps apply
            // even before anything is deployed.
            for unit in picker_ctx.picker.available.iter().filter(|u| u.visible) {
                let Some((g, c)) =
                    unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                        .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                        .and_then(|p| fok_cap_group(&p.identity))
                else {
                    continue;
                };
                if !groups.iter().any(|(eg, _, _, _)| *eg == g) {
                    groups.push((g, c, 0, 0));
                }
            }
            if let Some(gs) = game_state.as_deref() {
                // Eliminated counters used their slot too: a kill must not
                // reopen the group for a fresh counter.
                let identities = gs.0.units.iter().map(|u| u.profile.identity).chain(
                    gs.0.eliminated.iter().filter_map(|&id| {
                        omdurman_rules::unit_profiles::profile_for_unit(id).map(|p| p.identity)
                    }),
                );
                for identity in identities {
                    if let Some((g, _)) = fok_cap_group(&identity)
                        && let Some(entry) = groups.iter_mut().find(|(eg, _, _, _)| *eg == g)
                    {
                        entry.2 += 1;
                    }
                }
            }
            // Iterate once more to hide counters once (placed + kept >= cap).
            for unit in &mut picker_ctx.picker.available {
                if !unit.visible {
                    continue;
                }
                let group =
                    unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                        .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                        .and_then(|p| fok_cap_group(&p.identity).map(|(g, _)| g));
                let Some(group) = group else { continue };
                let entry = groups.iter_mut().find(|(eg, _, _, _)| *eg == group);
                let Some(entry) = entry else { continue };
                let (_g, cap, placed, kept) = *entry;
                if placed + kept >= cap {
                    unit.visible = false;
                } else {
                    entry.3 += 1;
                }
            }
        }
    }

    // -- eliminated counters --
    // A destroyed unit never returns to play; the engine refuses it
    // (`RuleError::UnitEliminated`), so keep it out of the tray.
    if let Some(state) = game_state.as_deref() {
        for unit in &mut picker_ctx.picker.available {
            if unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                .is_some_and(|id| state.0.eliminated.contains(&id))
            {
                unit.visible = false;
            }
        }
    }

    // -- faction filter (bound multiplayer) --
    // In a bound game each side deploys (and brings on reinforcements) only
    // its own counters: hide units whose owner isn't the local player
    // (§9.2/§9.3). This keeps the wrong side's counters out of sight; the
    // engine's placement checks backstop it. Unbound sessions (no faction
    // binding, `local` is `None`) stay permissive so solo testing can drive
    // both sides.
    if let (Some(local), Some(_)) = (peers.local(), game_state.as_deref()) {
        for unit in &mut picker_ctx.picker.available {
            let owner_is_local = omdurman_rules::unit_profiles::section_owner(unit.section_name)
                .is_some_and(|owner| owner == local);
            if !owner_is_local {
                unit.visible = false;
            }
        }
    }

    // -- command-scope filter (§1.1 multi-player commands, setup only) --
    // In a commanded game hide counters another member's command claims; one's
    // own scope plus the communal pool (no scope claims it) stay visible and
    // placable. `scope_allows` is the same predicate the pickup gates use.
    // Sessions without command assignments are unaffected.
    if let Some(state) = game_state.as_deref()
        && matches!(state.0.phase, omdurman_rules::Phase::Setup)
        && peers.any_commands()
    {
        for unit in &mut picker_ctx.picker.available {
            if !unit.visible {
                continue;
            }
            let Some(identity) = omdurman_rules::unit_profiles::identity_for_counter(
                unit.section_name,
                unit.col,
                unit.row,
            ) else {
                continue;
            };
            if !peers.scope_allows(&identity) {
                unit.visible = false;
            }
        }
    }

    // -- Hide outside placing phases, or when the tray is empty --
    // The picker window exists to *place* counters: deployment during Setup
    // (§9.2/§9.3/§10) and reinforcements entering during Movement
    // (§9.112/§9.113). During fire/melee nothing can be placed, so the whole
    // left-rail panel collapses and the board gets the full width. The empty
    // check also hides a spent tray instead of leaving an "all units placed"
    // stub. Visibility filters (scenario OOB, faction, command scope) have all
    // run above, so `.visible` is authoritative.
    let placing_phase = game_state.as_deref().is_none_or(|gs| {
        matches!(
            gs.0.phase,
            omdurman_rules::Phase::Setup | omdurman_rules::Phase::Movement
        )
    });
    if !placing_phase || !picker_ctx.picker.available.iter().any(|u| u.visible) {
        return;
    }

    crate::layout::left_rail_panel(
        ctx,
        &mut layout,
        "picker_panel",
        "unit_picker_panel",
        216.0,
        |ui| {
            // -- sidebar --
            egui::Panel::left("unit_picker_panel")
                .resizable(true)
                .default_size(200.0)
                .size_range(140.0..=320.0)
                .frame(crate::ui::frames::rail())
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));
                    ui.label(
                        egui::RichText::new("Unit Picker")
                            .size(16.0)
                            .color(crate::ui::palette::TEXT_STRONG),
                    );
                    ui.separator();
                    ui.add_space(4.0);

                    // Auto-place-next toggle: when enabled, placing a unit
                    // automatically selects the next one in the same section.
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Auto next")
                                .size(12.0)
                                .color(crate::ui::palette::TEXT_MUTED),
                        );
                        ui.checkbox(&mut picker_ctx.picker.auto_place_next, "");
                    });
                    ui.add_space(2.0);

                    let mut clicked_idx: Option<usize> = None;
                    let mut drag_idx: Option<usize> = None;
                    let mut drag_cancelled = false;
                    let sprite_size = 44.0;
                    let margin = 2.0;
                    let cell_size = sprite_size + margin * 2.0;

                    // clear selection if the picked unit is now invisible
                    if let PickerState::Placing { unit_idx, .. } = &*picker_ctx.state
                        && picker_ctx
                            .picker
                            .available
                            .get(*unit_idx)
                            .is_some_and(|u| !u.visible)
                    {
                        *picker_ctx.state = PickerState::Idle;
                    }

                    // Once a game starts, default-open the local player's faction and
                    // collapse the other. This is a local view choice -- afterwards the
                    // user may fold/unfold either heading freely, and nothing is sent
                    // over the network.
                    let local_faction = peers.local();
                    let game_started = peers.any_assigned();

                    ui.style_mut().spacing.scroll.floating = false;
                    egui::ScrollArea::vertical()
                        .id_salt("unit_picker_scroll")
                        .show(ui, |ui| {
                            use omdurman_types::Player;
                            // On the transition into a started game, force each category
                            // open/closed once: the local faction open, the foreign one
                            // collapsed. `default_open` alone wouldn't do this, because
                            // egui persists the header's open state from before the game
                            // (when both were open), so we set it explicitly on the edge.
                            let just_started = game_started && !*was_game_started;
                            *was_game_started = game_started;

                            for faction in [Player::Dervish, Player::AngloEgyptian] {
                                let heading = crate::ui::faction_name(faction);
                                // Skip a category with no visible units.
                                let any_visible = picker_ctx.picker.available.iter().any(|u| {
                                    u.visible
                                        && omdurman_rules::unit_profiles::section_owner(
                                            u.section_name,
                                        ) == Some(faction)
                                });
                                if !any_visible {
                                    continue;
                                }

                                let header_id = ui.make_persistent_id(("picker_faction", heading));
                                let mut header =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                header_id,
                                true,
                            );
                                // Force open/closed at the game-start edge.
                                if just_started {
                                    header.set_open(local_faction == Some(faction));
                                }
                                header
                                    .show_header(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(heading)
                                                .size(14.0)
                                                .color(crate::ui::palette::TEXT),
                                        );
                                    })
                                    .body(|ui| {
                                        render_faction_units(
                                            ui,
                                            PickerRead {
                                                picker: &picker_ctx.picker,
                                                state: &picker_ctx.state,
                                            },
                                            faction,
                                            cell_size,
                                            sprite_size,
                                            DragState {
                                                clicked_idx: &mut clicked_idx,
                                                drag_idx: &mut drag_idx,
                                                drag_cancelled: &mut drag_cancelled,
                                            },
                                            UnitAnnotations {
                                                rulebook: &rulebook,
                                            },
                                        );
                                    });
                            }
                        });

                    let pick_label = |idx: usize| -> String {
                        picker_ctx
                            .picker
                            .available
                            .get(idx)
                            .map(|u| {
                                format!("{} {},{}", u.section_name.display_name(), u.col, u.row)
                            })
                            .unwrap_or_else(|| format!("tray#{idx}"))
                    };
                    let pick_stamp = crate::ui_trace::Stamp::of(game_state.as_deref());
                    if let Some(idx) = clicked_idx {
                        match &*picker_ctx.state {
                            PickerState::Placing { unit_idx, .. } if *unit_idx == idx => {
                                *picker_ctx.state = PickerState::Idle;
                            }
                            _ => {
                                crate::ui_trace::placement_pick(
                                    &pick_label(idx),
                                    "click",
                                    &pick_stamp,
                                );
                                *picker_ctx.state = PickerState::Placing {
                                    unit_idx: idx,
                                    preview_hex: None,
                                    preview_valid: false,
                                    drag_drop: false,
                                };
                            }
                        }
                    }
                    if drag_cancelled
                        && let Some(next) = state_after_cancelled_drag(&picker_ctx.state)
                    {
                        *picker_ctx.state = next;
                    }
                    if let Some(idx) = drag_idx {
                        crate::ui_trace::placement_pick(&pick_label(idx), "drag", &pick_stamp);
                        *picker_ctx.state = PickerState::Placing {
                            unit_idx: idx,
                            preview_hex: None,
                            preview_valid: false,
                            drag_drop: true,
                        };
                    }
                })
                .response
                .rect
        },
    );

    // -- ghost sprite at cursor when placing --
    if let PickerState::Placing { unit_idx, .. } = &*picker_ctx.state
        && let Some(unit) = picker_ctx.picker.available.get(*unit_idx)
        && let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id())
        && let Some(pos) = ctx.pointer_latest_pos()
    {
        let ghost_size = 48.0;
        let ghost_rect = egui::Rect::from_center_size(pos, egui::Vec2::new(ghost_size, ghost_size));
        ctx.debug_painter().image(
            tex_id,
            ghost_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            crate::ui::palette::GHOST_TINT,
        );
    }
}

// -- Placement preview: green/red hex highlight ---------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sprite file must bucket into exactly one section. The `Hadendowa`
    /// and `Hadendowa_Forts` blocks both start with the same prefix; a naive
    /// `starts_with` match would swallow the fort counters into `Hadendowa`,
    /// which silently drops the auto-setup North Fort placement. (Sections off
    /// the cut sheet, like FALL OF KHARTOUM's `British_Forts`, count too.)
    #[test]
    fn sprite_files_bucket_into_exact_sections() {
        use strum::VariantArray;
        let order = SectionName::VARIANTS;
        for &(filename, col, row) in generated::SPRITE_PATHS {
            let section = bucket_section(order, filename, col, row);
            assert!(
                section.is_some(),
                "sprite {filename} must bucket into exactly one section, got None"
            );
        }
    }

    #[test]
    fn fort_sprites_belong_to_hadendowa_forts() {
        let order = section_order();
        assert_eq!(
            bucket_section(order, "Hadendowa_Forts_0_0", 0, 0),
            Some(SectionName::HadendowaForts)
        );
        assert_eq!(
            bucket_section(order, "Hadendowa_0_0", 0, 0),
            Some(SectionName::Hadendowa)
        );
    }

    #[test]
    fn a_drag_dropped_on_the_ui_leaves_nothing_in_hand() {
        let dragged = PickerState::Placing {
            unit_idx: 3,
            preview_hex: None,
            preview_valid: false,
            drag_drop: true,
        };
        assert!(matches!(
            state_after_cancelled_drag(&dragged),
            Some(PickerState::Idle)
        ));
        // A click-picked counter (no drag) stays in hand.
        let clicked = PickerState::Placing {
            unit_idx: 3,
            preview_hex: None,
            preview_valid: false,
            drag_drop: false,
        };
        assert!(state_after_cancelled_drag(&clicked).is_none());
        assert!(state_after_cancelled_drag(&PickerState::Idle).is_none());
    }
}
