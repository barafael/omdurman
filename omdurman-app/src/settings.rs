use bevy::prelude::*;
use bevy_egui::egui;
use omdurman_net::new_player_petname;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

// -- Settings resources -----------------------------------------------------

/// Set by the lobby when the user clicks Host or Join.
/// The system `handle_reconnect` picks this up, disconnects from
/// the current room, and opens a new socket with the new room ID.
#[derive(Resource)]
pub struct ReconnectRoom(pub String);

fn generate_name() -> String {
    new_player_petname()
}

/// Vendored Dawnbringer32 warm colors (originally from bevy-color-palettes).
mod dawnbringer32 {
    use bevy::color::Srgba;

    pub const RUST_BROWN: Srgba = Srgba::rgb(0.396, 0.224, 0.192);
    pub const COPPER_TAN: Srgba = Srgba::rgb(0.561, 0.337, 0.231);
    pub const PUMPKIN_ORANGE: Srgba = Srgba::rgb(0.875, 0.443, 0.149);
    pub const SANDY_GOLD: Srgba = Srgba::rgb(0.851, 0.627, 0.400);
    pub const PEACH_BEIGE: Srgba = Srgba::rgb(0.929, 0.765, 0.604);
    pub const SUN_YELLOW: Srgba = Srgba::rgb(0.984, 0.949, 0.212);
    pub const BLOOD_RED: Srgba = Srgba::rgb(0.678, 0.196, 0.196);
    pub const ROSE_RED: Srgba = Srgba::rgb(0.851, 0.341, 0.388);
    pub const PINK_BLOSSOM: Srgba = Srgba::rgb(0.843, 0.482, 0.729);
    pub const BRONZE_GOLD: Srgba = Srgba::rgb(0.541, 0.435, 0.188);
}

fn random_warm_color() -> egui::Color32 {
    use rand::RngExt;
    let warm = [
        dawnbringer32::RUST_BROWN,
        dawnbringer32::COPPER_TAN,
        dawnbringer32::PUMPKIN_ORANGE,
        dawnbringer32::SANDY_GOLD,
        dawnbringer32::PEACH_BEIGE,
        dawnbringer32::SUN_YELLOW,
        dawnbringer32::BLOOD_RED,
        dawnbringer32::ROSE_RED,
        dawnbringer32::PINK_BLOSSOM,
        dawnbringer32::BRONZE_GOLD,
    ];
    let mut rng = rand::rng();
    let c = warm[rng.random_range(0..warm.len())];
    egui::Color32::from_rgb(
        (c.red * 255.0) as u8,
        (c.green * 255.0) as u8,
        (c.blue * 255.0) as u8,
    )
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
            name: generate_name(),
            show_other_cursors: true,
            color: random_warm_color(),
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
    pub fn commit_color(&mut self, c: egui::Color32) {
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

/// Trigger a browser file download with the given content.
/// On native this is a no-op (the file is already saved to disk).
#[cfg(target_arch = "wasm32")]
pub(crate) fn download_ron_file(content: &str) {
    let arr = js_sys::Array::new();
    arr.push(&JsValue::from_str(content));
    let blob = web_sys::Blob::new_with_str_sequence(arr.as_ref()).ok();
    let Some(blob) = blob else { return };
    let url = web_sys::Url::create_object_url_with_blob(&blob).ok();
    let Some(url) = url else { return };
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
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
