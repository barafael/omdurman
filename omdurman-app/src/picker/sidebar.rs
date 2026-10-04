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
            let path = super::sprite_asset_path(filename);
            let handle = asset_server.load(&path);
            section_sprites[idx].push(PickerUnit {
                section_name,
                col,
                row,
                handle,
                is_boat: false,
                visible: true,
                offered: true,
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
    // Grouped by what the units are, not by counter sheet: leaders,
    // gunboats, one group per brigade (brigade integrity, §5.54), mounted
    // units, guns -- and each Dervish tribe apart (tribes may not stack
    // together, §5.52). Within a group, sheet order.
    let mut groups: Vec<((u8, String), Vec<usize>)> = Vec::new();
    for (idx, unit) in picker.available.iter().enumerate() {
        if !unit.shown()
            || omdurman_rules::unit_profiles::section_owner(unit.section_name) != Some(faction)
        {
            continue;
        }
        let key = unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
            .and_then(omdurman_rules::unit_profiles::profile_for_unit)
            .map_or_else(
                || (9, unit.section_name.display_name().to_string()),
                |p| tray_group(&p.identity),
            );
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, members)) => members.push(idx),
            None => groups.push((key, vec![idx])),
        }
    }
    groups.sort_by(|(a, _), (b, _)| a.cmp(b));
    for ((_, label), members) in &groups {
        {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(13.0)
                        .color(crate::ui::palette::TEXT_SOFT),
                );
                ui.label(
                    egui::RichText::new(format!("({})", members.len()))
                        .size(11.0)
                        .color(crate::ui::palette::TEXT_FAINT),
                );
            });
            ui.add_space(2.0);

            ui.horizontal_wrapped(|ui| {
                for &j in members {
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

/// A counter's tray group: (display order, heading).
pub(crate) fn tray_group(identity: &omdurman_rules::UnitIdentity) -> (u8, String) {
    use omdurman_rules::UnitIdentity as U;
    match identity {
        U::AngloEgyptianLeader(_) | U::DervishLeader(_) => (0, "Leaders".into()),
        U::AngloEgyptianGunboat(_) | U::DervishGunboat(_) => (1, "Gunboats".into()),
        U::AngloEgyptianInfantry { .. } if identity.is_friendlies() => (3, "Friendlies".into()),
        U::AngloEgyptianInfantry { brigade, .. } => (2, format!("{brigade} Brigade")),
        U::DervishTribal { tribe } => (2, tribe.to_string()),
        U::AngloEgyptianCavalry => (4, "Cavalry".into()),
        U::AngloEgyptianCamelCorps => (4, "Camel Corps".into()),
        U::AngloEgyptianArtillery | U::DervishArtillery => (5, "Artillery".into()),
        U::AngloEgyptianMaxim => (5, "Maxims".into()),
        U::RoyalEngineers => (6, "Royal Engineers".into()),
        U::AngloEgyptianFort | U::DervishFort => (7, "Forts".into()),
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

/// The counter tray, drawn as a section of the command rail
/// (`overview::unit_overview_ui`): only the counters that may be placed now
/// -- the set-up force during set-up, this turn's arrivals during a Movement
/// phase -- grouped by counter-sheet section. Click or drag a counter, then
/// a hex. [`unit_picker_ui`] keeps the tray's flags and textures current.
pub(crate) fn draw_tray(
    ui: &mut egui::Ui,
    picker: &mut UnitPicker,
    state: &mut PickerState,
    rulebook: &crate::rulebook::Rulebook,
    stamp: &crate::ui_trace::Stamp,
) {
    use omdurman_types::Player;
    let mut clicked_idx: Option<usize> = None;
    let mut drag_idx: Option<usize> = None;
    let mut drag_cancelled = false;
    let sprite_size = 44.0;
    let cell_size = sprite_size + 4.0;
    let factions: Vec<Player> = [Player::Dervish, Player::AngloEgyptian]
        .into_iter()
        .filter(|&faction| {
            picker.available.iter().any(|u| {
                u.shown()
                    && omdurman_rules::unit_profiles::section_owner(u.section_name) == Some(faction)
            })
        })
        .collect();
    for &faction in &factions {
        // Both sides only in an unbound (solo test) session.
        if factions.len() > 1 {
            ui.label(
                egui::RichText::new(crate::ui::faction_name(faction))
                    .size(14.0)
                    .color(crate::ui::faction_color(faction)),
            );
        }
        render_faction_units(
            ui,
            PickerRead { picker, state },
            faction,
            cell_size,
            sprite_size,
            DragState {
                clicked_idx: &mut clicked_idx,
                drag_idx: &mut drag_idx,
                drag_cancelled: &mut drag_cancelled,
            },
            UnitAnnotations { rulebook },
        );
    }
    ui.add_space(2.0);
    ui.checkbox(
        &mut picker.auto_place_next,
        egui::RichText::new("Then pick the next of the group")
            .size(12.0)
            .color(crate::ui::palette::TEXT_MUTED),
    );

    let pick_label = |picker: &UnitPicker, idx: usize| -> String {
        picker
            .available
            .get(idx)
            .map(|u| format!("{} {},{}", u.section_name.display_name(), u.col, u.row))
            .unwrap_or_else(|| format!("tray#{idx}"))
    };
    if let Some(idx) = clicked_idx {
        match &*state {
            PickerState::Placing { unit_idx, .. } if *unit_idx == idx => {
                *state = PickerState::Idle;
            }
            _ => {
                crate::ui_trace::placement_pick(&pick_label(picker, idx), "click", stamp);
                *state = PickerState::Placing {
                    unit_idx: idx,
                    preview_hex: None,
                    preview_valid: false,
                    drag_drop: false,
                };
            }
        }
    }
    if drag_cancelled && let Some(next) = state_after_cancelled_drag(state) {
        *state = next;
    }
    if let Some(idx) = drag_idx {
        crate::ui_trace::placement_pick(&pick_label(picker, idx), "drag", stamp);
        *state = PickerState::Placing {
            unit_idx: idx,
            preview_hex: None,
            preview_valid: false,
            drag_drop: true,
        };
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
    mut offer_cache: Local<Option<(usize, Vec<bool>)>>,
) {
    let PickerAssetCtx {
        images,
        annotations,
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

    // Nothing to place outside Setup / Movement (or after the battle): skip
    // the per-counter filter passes below, which would otherwise run every
    // frame for a panel that is not shown.
    if game_state.as_deref().is_some_and(|gs| {
        gs.0.game_over
            || !matches!(
                gs.0.phase,
                omdurman_rules::Phase::Setup | omdurman_rules::Phase::Movement
            )
    }) {
        // Nothing is placeable: empty the tray's "now" flags (cheaply, once).
        if picker_ctx.picker.available.iter().any(|u| u.offered) {
            for unit in &mut picker_ctx.picker.bypass_change_detection().available {
                unit.offered = false;
            }
            *offer_cache = None;
        }
        return;
    }

    // The filter passes rewrite the tray's flags every frame; they go around
    // change detection, and the tray reads as changed only when a flag
    // actually flipped (so systems gated on `UnitPicker` changes, e.g.
    // `reconcile_unit_sprites`, stay idle).
    let flags = |picker: &UnitPicker| -> Vec<(bool, bool, bool, bool)> {
        picker
            .available
            .iter()
            .map(|u| {
                (
                    u.visible && u.offered,
                    u.is_boat,
                    u.annotations_loaded,
                    u.egui_texture.is_some(),
                )
            })
            .collect()
    };
    let before = flags(&picker_ctx.picker);
    {
        let picker = picker_ctx.picker.bypass_change_detection();
        // -- cache egui textures & look up is_boat from annotations --
        for unit in &mut picker.available {
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
                for unit in &mut picker.available {
                    if !allowed.contains(&unit.section_name) {
                        unit.visible = false;
                    }
                }
            }
            if matches!(state.0.scenario, Scenario::Historical) {
                // §9.211/§9.212: GORDON, the Friendlies, Isa Zachneih, the
                // gunboats and the forts sit this battle out.
                for unit in picker.available.iter_mut().filter(|u| u.visible) {
                    unit.visible =
                        unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                            .is_some_and(omdurman_rules::effects::historical_counter_in_play);
                }
            }
            if matches!(state.0.scenario, Scenario::Campaign) {
                // §9.113: "The GORDON unit is not used in this scenario."
                for unit in picker.available.iter_mut().filter(|u| u.visible) {
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
                for unit in &mut picker.available {
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
                for unit in picker.available.iter().filter(|u| u.visible) {
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
                for unit in &mut picker.available {
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
            for unit in &mut picker.available {
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
            for unit in &mut picker.available {
                let owner_is_local =
                    omdurman_rules::unit_profiles::section_owner(unit.section_name)
                        .is_some_and(|owner| owner == local);
                if !owner_is_local {
                    unit.visible = false;
                }
            }
        }

        // -- placeable now (recomputed, never sticky) --
        // Set-up offers the scenario's set-up force (§9.111/§9.211/§9.321;
        // in the Campaign the Anglo-Egyptians deploy nothing), narrowed in a
        // commanded game to one's own command scope plus the communal pool
        // (§1.1); a Movement phase offers what may enter this turn
        // (§9.112/§9.113). Recomputed when the engine state or the tray moved.
        if let Some(state) = game_state.as_deref() {
            let key = picker.available.len();
            if game_state.as_ref().is_some_and(|gs| gs.is_changed())
                || offer_cache.as_ref().is_none_or(|(len, _)| *len != key)
            {
                let gs = &state.0;
                let offered: Vec<bool> = picker
                    .available
                    .iter()
                    .map(|unit| {
                        if !unit.visible {
                            return false;
                        }
                        let Some(id) = unit_id_for_section_pos(
                            unit.section_name,
                            unit.col as u8,
                            unit.row as u8,
                        ) else {
                            return false;
                        };
                        match gs.phase {
                            omdurman_rules::Phase::Setup => {
                                gs.counter_in_play_at_setup(id)
                                    && (!peers.any_commands()
                                        || omdurman_rules::unit_profiles::profile_for_unit(id)
                                            .is_some_and(|p| peers.scope_allows(&p.identity)))
                            }
                            omdurman_rules::Phase::Movement => crate::reinforce::enterable(gs, id),
                            _ => false,
                        }
                    })
                    .collect();
                *offer_cache = Some((key, offered));
            }
            if let Some((_, offered)) = offer_cache.as_ref() {
                for (unit, &offered) in picker.available.iter_mut().zip(offered) {
                    unit.offered = offered;
                }
            }
        }
    }
    if flags(&picker_ctx.picker) != before {
        picker_ctx.picker.set_changed();
    }

    // A counter in hand that is no longer placeable (its quota filled, the
    // phase moved on) drops out of the hand.
    if let PickerState::Placing { unit_idx, .. } = &*picker_ctx.state
        && picker_ctx
            .picker
            .available
            .get(*unit_idx)
            .is_none_or(|u| !u.shown())
    {
        *picker_ctx.state = PickerState::Idle;
    }

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
    /// Every counter of the roster (`units.ron`) has its sprite: the cell the
    /// game buckets from the sprite files exists for its section, column and
    /// row. (Moved here from the retired asset editor.)
    #[test]
    fn every_unit_in_the_roster_has_a_sprite() {
        use strum::VariantArray;
        let roster = include_str!("../../../Boardgame - Remember_Gordon/tables/units.ron");
        let ids: Vec<omdurman_rules::UnitId> = roster
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix('"'))
            .filter_map(|l| l.split('"').next())
            .filter_map(|id| ron::from_str(id).ok())
            .collect();
        assert!(ids.len() > 200, "the full counter set, got {}", ids.len());
        let order = SectionName::VARIANTS;
        let missing: Vec<_> = ids
            .iter()
            .filter(|id| {
                let (section, col, row) = id.section_pos();
                !generated::SPRITE_PATHS.iter().any(|&(f, c, r)| {
                    c == u32::from(col)
                        && r == u32::from(row)
                        && bucket_section(order, f, c, r) == Some(section)
                })
            })
            .collect();
        assert!(missing.is_empty(), "no sprite for {missing:?}");
    }

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
