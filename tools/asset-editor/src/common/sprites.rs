//! Unit sprite textures, loaded from `assets/sprites/{unit_id}.webp`.
//!
//! Interior mutability lets render code call [`SpriteCache::get`] with `&self`
//! while the document is borrowed immutably — no clone gymnastics at call
//! sites. Failed lookups are cached so a missing file isn't re-opened (and
//! re-decoded) every frame.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use egui::{TextureHandle, TextureOptions};

pub struct SpriteCache {
    dir: PathBuf,
    textures: RefCell<HashMap<String, TextureHandle>>,
    missing: RefCell<HashSet<String>>,
}

impl SpriteCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        SpriteCache {
            dir: dir.into(),
            textures: RefCell::new(HashMap::new()),
            missing: RefCell::new(HashSet::new()),
        }
    }

    /// The texture for `unit_id`, or `None` if no sprite file exists.
    /// Cloning the handle is cheap (it is refcounted internally).
    pub fn get(&self, ctx: &egui::Context, unit_id: &str) -> Option<TextureHandle> {
        if let Some(tex) = self.textures.borrow().get(unit_id) {
            return Some(tex.clone());
        }
        if self.missing.borrow().contains(unit_id) {
            log::trace!("sprite {unit_id}: cache miss (known missing)");
            return None;
        }

        let path = self.dir.join(format!("{unit_id}.webp"));
        let Some(tex) = self.load(ctx, &path, unit_id) else {
            log::debug!("no sprite for {unit_id} at {}", path.display());
            self.missing.borrow_mut().insert(unit_id.to_string());
            return None;
        };
        log::trace!("loaded sprite {unit_id}");
        self.textures.borrow_mut().insert(unit_id.to_string(), tex.clone());
        Some(tex)
    }

    fn load(&self, ctx: &egui::Context, path: &std::path::Path, id: &str) -> Option<TextureHandle> {
        let img = image::open(path).ok()?;
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        Some(ctx.load_texture(id, image, TextureOptions::default()))
    }
}
