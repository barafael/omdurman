use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use bevy_matchbox::prelude::*;
use egui::text::{LayoutJob, TextFormat};
use ron::ser::PrettyConfig;

use crate::editor::EditorToolState;
use omdurman_net::{Ephemeral, NetMsg, NetState};

#[derive(Resource, Default)]
pub struct EventViewerState {
    pub selected: Option<usize>,
    cached_idx: Option<usize>,
    cached_detail: String,
}

pub fn event_viewer_ui(
    mut contexts: EguiContexts,
    mode: EditorToolState,
    mut state: ResMut<EventViewerState>,
    recorder: Option<Res<crate::game_record::GameRecorder>>,
    net: Res<NetState>,
    socket: Option<ResMut<MatchboxSocket>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_event_viewer() {
        return;
    }

    let Some(rec) = recorder else { return };
    let Some(ref record) = rec.record else { return };

    let prev_selected = state.selected;

    let bg = egui::Color32::from_rgb(20, 16, 12);
    let dim = egui::Color32::from_gray(140);
    let sel_bg = egui::Color32::from_rgb(90, 60, 30);
    let row_h = 22.0;

    // Fill only the space left *below* the docked top bar (`available_rect`
    // already excludes the mode/tab panel), so the viewer never paints over the
    // tab bar. Using `content_rect` here would start at y=0 and swallow it.
    let content = ctx.content_rect();

    egui::Area::new(egui::Id::new("event_viewer_backdrop"))
        // Anchor the Area's origin at the content top-left (below the top bar),
        // and lay children out with absolute rects from the same `content.min`.
        // The Area origin and the child layout must share an origin: when they
        // didn't (Area at window (0,0), children at `content.min`), egui's
        // interaction bounds were offset from the painted rows and the whole
        // viewer stopped responding to clicks and hovers.
        .fixed_pos(content.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.set_min_size(content.size());
            ui.painter().rect_filled(content, 0.0, bg);

            let left_w = content.width() * 0.32;
            let gap = 4.0;

            // -- left panel (scrollable event list) --
            let left = egui::Rect::from_min_size(content.min, egui::vec2(left_w, content.height()));
            ui.scope_builder(egui::UiBuilder::new().max_rect(left), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_gray(22))
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        let mut clicked = None;
                        egui::ScrollArea::vertical()
                            .id_salt("event_list")
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                for (idx, event) in record.events.iter().enumerate() {
                                    let is_sel = state.selected == Some(idx);
                                    let row = egui::Rect::from_min_size(
                                        ui.cursor().left_top(),
                                        egui::vec2(ui.available_width(), row_h),
                                    );
                                    let resp = ui.allocate_rect(row, egui::Sense::click());
                                    if resp.clicked() {
                                        clicked = Some(idx);
                                    }
                                    if is_sel {
                                        ui.painter().rect_filled(row, 2.0, sel_bg);
                                    }
                                    let text = format!(
                                        "#{:04}  {}  {}",
                                        event.seq,
                                        event.utc.format("%H:%M:%S"),
                                        payload_label(&event.payload),
                                    );
                                    ui.painter().text(
                                        egui::pos2(row.min.x + 6.0, row.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        text,
                                        egui::FontId::monospace(12.0),
                                        if is_sel { egui::Color32::WHITE } else { dim },
                                    );
                                }
                            });
                        if let Some(idx) = clicked {
                            state.selected = Some(idx);
                        }
                    });
            });

            // -- right panel (event detail with RON syntax highlighting) --
            let right = egui::Rect::from_min_size(
                egui::pos2(content.min.x + left_w + gap, content.min.y),
                egui::vec2(content.width() - left_w - gap, content.height()),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(right), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_gray(22))
                    .inner_margin(egui::Margin::symmetric(10, 10))
                    .show(ui, |ui| {
                        if let Some(idx) = state.selected {
                            if let Some(event) = record.events.get(idx) {
                                ui.style_mut().override_font_id =
                                    Some(egui::FontId::monospace(12.0));

                                // header: event metadata
                                ui.colored_label(
                                    egui::Color32::from_gray(100),
                                    format!(
                                        "#{:04}  {}  sender={}  {}",
                                        event.seq,
                                        event.utc.format("%H:%M:%S.%3f"),
                                        event.sender_idx.map_or("?".to_string(), |s| s.to_string()),
                                        payload_label(&event.payload),
                                    ),
                                );
                                ui.add_space(8.0);

                                // cache the RON serialization (expensive for LoadAnnotations)
                                if state.cached_idx != Some(idx) {
                                    state.cached_idx = Some(idx);
                                    state.cached_detail = ron::ser::to_string_pretty(
                                        &event.payload,
                                        PrettyConfig::default(),
                                    )
                                    .unwrap_or_else(|_| format!("{:#?}", event.payload));
                                }
                                let job = highlight_ron(&state.cached_detail);

                                egui::ScrollArea::vertical()
                                    .id_salt("event_detail")
                                    .auto_shrink(false)
                                    .show(ui, |ui| {
                                        ui.label(job);
                                    });
                            }
                        } else {
                            ui.style_mut().override_font_id =
                                Some(egui::FontId::proportional(14.0));
                            ui.colored_label(dim, "Select an event from the list");
                        }
                    });
            });
        });

    // broadcast selection changes to other players
    if state.selected != prev_selected
        && let Some(mut socket) = socket
    {
        let idx = state.selected.map(|i| i as i32).unwrap_or(-1);
        omdurman_net::broadcast_unreliable(
            &mut socket,
            &net.peers,
            &NetMsg::Ephemeral(Ephemeral::EventViewerSelect(idx)),
        );
    }
}

fn highlight_ron(source: &str) -> LayoutJob {
    let string_col = egui::Color32::from_rgb(206, 145, 120);
    let number_col = egui::Color32::from_rgb(220, 190, 120);
    let keyword_col = egui::Color32::from_rgb(224, 130, 60);
    let field_col = egui::Color32::from_rgb(235, 200, 140);
    let variant_col = egui::Color32::from_rgb(210, 120, 90);
    let punct_col = egui::Color32::from_gray(128);
    let default_col = egui::Color32::from_gray(180);

    let mut job = LayoutJob::default();
    let s = source.as_bytes();
    let n = s.len();
    let mut i = 0;

    let mut push = |range: std::ops::Range<usize>, color| {
        job.append(
            &source[range],
            0.0,
            TextFormat {
                font_id: egui::FontId::monospace(12.0),
                color,
                ..Default::default()
            },
        );
    };

    while i < n {
        // string literals
        if s[i] == b'"' {
            let start = i;
            i += 1;
            while i < n && s[i] != b'"' {
                if s[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if i < n {
                i += 1;
            }
            push(start..i, string_col);
            continue;
        }

        // line comments
        if i + 1 < n && s[i] == b'/' && s[i + 1] == b'/' {
            let start = i;
            while i < n && s[i] != b'\n' {
                i += 1;
            }
            push(start..i, egui::Color32::from_rgb(150, 130, 90));
            continue;
        }

        // numbers (integer, float, negative, scientific)
        if s[i].is_ascii_digit()
            || (s[i] == b'-' && i + 1 < n && s[i + 1].is_ascii_digit())
            || (s[i] == b'+' && i + 1 < n && s[i + 1].is_ascii_digit())
        {
            let start = i;
            if s[i] == b'-' || s[i] == b'+' {
                i += 1;
            }
            while i < n
                && (s[i].is_ascii_digit()
                    || s[i] == b'.'
                    || s[i] == b'e'
                    || s[i] == b'E'
                    || s[i] == b'+'
                    || s[i] == b'-')
            {
                i += 1;
            }
            push(start..i, number_col);
            continue;
        }

        // identifiers and keywords
        if s[i].is_ascii_alphabetic() || s[i] == b'_' {
            let start = i;
            while i < n && (s[i].is_ascii_alphanumeric() || s[i] == b'_') {
                i += 1;
            }
            let word = &source[start..i];
            match word {
                "true" | "false" | "Some" | "None" | "Ok" | "Err" => push(start..i, keyword_col),
                _ if i < n && s[i] == b':' => push(start..i, field_col),
                _ if word.as_bytes()[0].is_ascii_uppercase() => push(start..i, variant_col),
                _ => push(start..i, default_col),
            }
            continue;
        }

        // punctuation
        if matches!(
            s[i],
            b'(' | b')' | b'{' | b'}' | b'[' | b']' | b',' | b':' | b';' | b'.'
        ) {
            push(i..i + 1, punct_col);
            i += 1;
            continue;
        }

        // everything else (whitespace, operators)
        let start = i;
        i += 1;
        push(start..i, default_col);
    }

    job
}

fn payload_label(payload: &omdurman_net::GameEvent) -> &'static str {
    payload.into()
}
