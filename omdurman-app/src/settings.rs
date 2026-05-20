use crate::{PendingEdits, ReconnectRoom};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_net::{NetMsg, RoomId};
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

#[derive(Resource, Default)]
pub struct SettingsOverlay {
    pub visible: bool,
}

#[derive(Resource)]
pub struct LocalPlayerSettings {
    pub name: String,
    pub show_other_cursors: bool,
    color: egui::Color32,
    dirty: bool,
}

impl Default for LocalPlayerSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            show_other_cursors: true,
            color: egui::Color32::from_rgb(180, 200, 255),
            dirty: false,
        }
    }
}

impl LocalPlayerSettings {
    pub fn color_u8(&self) -> (u8, u8, u8) {
        (self.color.r(), self.color.g(), self.color.b())
    }
    pub fn color(&self) -> egui::Color32 {
        self.color
    }
    pub fn set_color(&mut self, c: egui::Color32) {
        self.color = c;
        self.dirty = true;
    }
    pub fn set_name(&mut self, n: String) {
        self.name = n;
        self.dirty = true;
    }
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }
}

#[derive(Resource, Default)]
pub struct PlayerInfoMap {
    pub peers: HashMap<bevy_matchbox::prelude::PeerId, PeerPlayerInfo>,
}

pub struct PeerPlayerInfo {
    pub name: String,
    pub color: egui::Color32,
}

pub fn settings_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut overlay: ResMut<SettingsOverlay>,
    mut local: ResMut<LocalPlayerSettings>,
    room: Res<RoomId>,
    mut pending: ResMut<PendingEdits>,
    #[cfg(target_arch = "wasm32")] recorder: Option<Res<crate::game_record::GameRecorder>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // ── hamburger button (top-right, always visible) ──
    egui::Area::new(egui::Id::new("hamburger_btn"))
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            let size = egui::vec2(40.0, 40.0);
            let resp = ui
                .add(egui::Button::new("").min_size(size).frame(false))
                .on_hover_text("Settings");
            let rect = resp.rect;
            if resp.clicked() {
                overlay.visible = !overlay.visible;
            }

            // draw three centered horizontal lines
            let painter = ui.painter();
            let w = 18.0;
            let h = 2.0;
            let gap = 5.0;
            let cx = rect.center().x;
            let cy = rect.center().y;
            let color = egui::Color32::WHITE;
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(cx, cy - gap), egui::vec2(w, h)),
                1.0,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(cx, cy), egui::vec2(w, h)),
                1.0,
                color,
            );
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(cx, cy + gap), egui::vec2(w, h)),
                1.0,
                color,
            );
        });

    if !overlay.visible {
        return;
    }

    let panel_w = 380.0;

    egui::Area::new(egui::Id::new("settings_backdrop"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(0.0, 52.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let screen = ui.max_rect();
            // no full-backdrop, just the panel

            let panel = egui::Rect::from_min_size(
                egui::pos2(screen.right() - panel_w, screen.top()),
                egui::vec2(panel_w, screen.height()),
            );

            let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(panel).layout(
                egui::Layout::top_down(egui::Align::LEFT).with_cross_align(egui::Align::LEFT),
            ));

            egui::Frame::new()
                .fill(egui::Color32::from_gray(30))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(16, 12))
                .show(&mut inner, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(16.0));

                    // ── header ──
                    ui.heading(egui::RichText::new("Settings").color(egui::Color32::WHITE));
                    ui.add_space(12.0);

                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));

                    // ── session ──
                    row_label(ui, "Session");
                    ui.horizontal(|ui| {
                        let mut session = room.0.clone();
                        ui.add_sized(
                            egui::vec2(180.0, 22.0),
                            egui::TextEdit::singleline(&mut session),
                        );
                        let host = ui.button("Host").clicked();
                        let join = ui.button("Join").clicked();
                        if host || join {
                            let id = if session.is_empty() {
                                room.0.clone()
                            } else {
                                session.clone()
                            };
                            commands.insert_resource(ReconnectRoom(id));
                            overlay.visible = false;
                        }
                    });
                    ui.add_space(8.0);

                    // ── name ──
                    row_label(ui, "Name");
                    let name_changed = ui
                        .add_sized(
                            egui::vec2(240.0, 22.0),
                            egui::TextEdit::singleline(&mut local.name),
                        )
                        .changed();
                    if name_changed {
                        let n = local.name.clone();
                        local.set_name(n);
                    }
                    ui.add_space(8.0);

                    // ── color ──
                    row_label(ui, "Color");
                    let mut c = local.color();
                    egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut c,
                        egui::color_picker::Alpha::Opaque,
                    );
                    if c != local.color() {
                        local.set_color(c);
                    }
                    ui.add_space(8.0);

                    // ── cursor checkbox ──
                    ui.checkbox(&mut local.show_other_cursors, "Show other players' cursors");
                    ui.add_space(12.0);

                    // ── download game record ──
                    #[cfg(target_arch = "wasm32")]
                    if let Some(ref rec) = recorder
                        && rec.record.is_some()
                    {
                        use ron::ser::PrettyConfig;
                        if ui.button("Download game record").clicked()
                            && let Some(ref record) = rec.record
                            && let Ok(ron_str) =
                                ron::ser::to_string_pretty(record, PrettyConfig::default())
                        {
                            download_ron_file(&ron_str);
                        }
                        ui.add_space(8.0);
                    }

                    // ── sync if dirty ──
                    if local.take_dirty() {
                        let (r, g, b) = local.color_u8();
                        pending.items.push(NetMsg::PlayerInfo {
                            name: local.name.clone(),
                            color_r: r,
                            color_g: g,
                            color_b: b,
                        });
                    }
                });
        });
}

fn row_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(egui::Color32::GRAY));
    ui.add_space(2.0);
}

/// Trigger a browser file download with the given content.
/// On native this is a no-op (the file is already saved to disk).
#[cfg(target_arch = "wasm32")]
fn download_ron_file(content: &str) {
    let arr = js_sys::Array::new();
    arr.push(&JsValue::from_str(content));
    let blob = web_sys::Blob::new_with_str_sequence(arr.as_ref()).ok();
    let Some(blob) = blob else { return };
    let url = web_sys::Url::create_object_url_with_blob(&blob).ok();
    let Some(url) = url else { return };
    let document = web_sys::window().unwrap().document().unwrap();
    let anchor: Option<web_sys::HtmlAnchorElement> = document
        .create_element("a")
        .ok()
        .and_then(|e| e.dyn_into::<web_sys::HtmlAnchorElement>().ok());
    let Some(a) = anchor else {
        web_sys::Url::revoke_object_url(&url).ok();
        return;
    };
    a.set_href(&url);
    a.set_download("game_record.ron");
    a.click();
    web_sys::Url::revoke_object_url(&url).ok();
}
