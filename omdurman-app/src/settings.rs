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
    let swatches = swatches();
    swatches[rand::rng().random_range(0..swatches.len())]
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

/// The player settings kept between launches: the name and colour others
/// see, and whether their cursors show.
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct StoredSettings {
    name: String,
    color: [u8; 3],
    show_other_cursors: bool,
}

impl LocalPlayerSettings {
    fn stored(&self) -> StoredSettings {
        let (r, g, b) = self.color_u8();
        StoredSettings {
            name: self.name.clone(),
            color: [r, g, b],
            show_other_cursors: self.show_other_cursors,
        }
    }

    fn from_stored(stored: StoredSettings) -> Self {
        let [r, g, b] = stored.color;
        Self {
            name: stored.name,
            show_other_cursors: stored.show_other_cursors,
            color: egui::Color32::from_rgb(r, g, b),
            dirty: false,
        }
    }
}

/// Read the stored settings: a file beside this instance's player key
/// (native), `localStorage` (web).
fn read_stored() -> Option<StoredSettings> {
    #[cfg(not(target_arch = "wasm32"))]
    let text = std::fs::read_to_string(crate::player_key_store::settings_path()?).ok()?;
    #[cfg(target_arch = "wasm32")]
    let text = web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(STORAGE_KEY)
        .ok()??;
    ron::from_str(&text).ok()
}

fn write_stored(stored: &StoredSettings) {
    let Ok(text) = ron::to_string(stored) else {
        return;
    };
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(path) = crate::player_key_store::settings_path()
        && let Err(error) = std::fs::write(&path, text)
    {
        warn!(%error, ?path, "could not save the player settings");
    }
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(STORAGE_KEY, &text);
    }
}

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "omdurman.player_settings";

/// At startup, once this instance holds its player slot: the stored name,
/// colour and cursor preference replace the random ones.
pub(crate) fn load_player_settings(mut settings: ResMut<LocalPlayerSettings>) {
    if let Some(stored) = read_stored() {
        *settings = LocalPlayerSettings::from_stored(stored);
    }
}

/// Save the settings whenever they change, so the next launch keeps them.
pub(crate) fn persist_player_settings(
    settings: Res<LocalPlayerSettings>,
    mut saved: Local<Option<StoredSettings>>,
) {
    if !settings.is_changed() {
        return;
    }
    let stored = settings.stored();
    if saved.as_ref() == Some(&stored) {
        return;
    }
    // The first look saves too: a name and colour rolled on a first launch
    // are the player's from then on.
    write_stored(&stored);
    *saved = Some(stored);
}

/// The warm swatches a player colour is drawn from.
fn swatches() -> Vec<egui::Color32> {
    [
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
    ]
    .into_iter()
    .map(|c| {
        egui::Color32::from_rgb(
            (c.red * 255.0) as u8,
            (c.green * 255.0) as u8,
            (c.blue * 255.0) as u8,
        )
    })
    .collect()
}

/// The colour a player whose colour another player already wears should
/// switch to: the first swatch nobody wears, `None` when every one is taken.
pub(crate) fn free_color(taken: &[egui::Color32]) -> Option<egui::Color32> {
    swatches().into_iter().find(|c| !taken.contains(c))
}

/// Two players in one colour cannot be told apart: the one with the larger
/// player key moves to a free swatch (a deterministic rule, so the two never
/// both switch). The change is announced like any colour change.
pub(crate) fn avoid_color_clash(
    mut settings: ResMut<LocalPlayerSettings>,
    peers: Query<(&crate::peers::PeerColor, &crate::peers::PeerPlayerKey)>,
    local_key: Option<Res<crate::seats::LocalPlayerKey>>,
) {
    let Some(local_key) = local_key else { return };
    let mine = settings.color();
    let clash = peers
        .iter()
        .any(|(color, key)| color.0 == mine && key.0 < local_key.0);
    if !clash {
        return;
    }
    let taken: Vec<egui::Color32> = peers.iter().map(|(c, _)| c.0).collect();
    if let Some(free) = free_color(&taken) {
        settings.commit_color(free);
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
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
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

#[cfg(test)]
mod persistence_tests {
    use super::*;

    /// What is stored comes back as it was.
    #[test]
    fn the_stored_settings_round_trip() {
        let mut settings = LocalPlayerSettings::default();
        settings.set_name("Gordon of Khartoum".into());
        settings.show_other_cursors = false;
        let stored = settings.stored();
        let text = ron::to_string(&stored).unwrap();
        let back = LocalPlayerSettings::from_stored(ron::from_str(&text).unwrap());
        assert_eq!(back.stored(), stored);
    }

    /// Of two players in one colour, only the one with the larger key moves,
    /// to a swatch nobody wears.
    #[test]
    fn the_larger_key_gives_way_in_a_colour_clash() {
        let worn = swatches()[0];
        let run = |local: u64| {
            let mut app = App::new();
            let settings = LocalPlayerSettings {
                color: worn,
                ..Default::default()
            };
            app.insert_resource(settings)
                .insert_resource(crate::seats::LocalPlayerKey(omdurman_net::PlayerKey(local)))
                .add_systems(Update, avoid_color_clash);
            app.world_mut().spawn((
                crate::peers::PeerColor(worn),
                crate::peers::PeerPlayerKey(omdurman_net::PlayerKey(5)),
            ));
            app.update();
            app.world().resource::<LocalPlayerSettings>().color()
        };
        assert_eq!(run(3), worn, "the smaller key keeps its colour");
        let moved = run(9);
        assert_ne!(moved, worn);
        assert!(swatches().contains(&moved));
    }
}
