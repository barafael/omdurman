//! Unit sprite textures, loaded from `assets/sprites/`.
//!
//! Sprite files are cut from the sheet grid named by the `SectionName` serde
//! convention (`Ali_Wad_Helu_0_0.webp`, `Jaalin_II_1_3.webp`), while
//! `units.ron` unit ids use the CamelCase enum names (`AliWadHelu_0_0`,
//! `JaalinII_1_3`). [`sprite_filename`] bridges the two with an
//! acronym-aware camel split, so both forms resolve.
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

        for filename in filename_candidates(unit_id) {
            let path = self.dir.join(&filename);
            if let Some(tex) = self.load(ctx, &path, unit_id) {
                log::trace!("loaded sprite {unit_id} from {filename}");
                self.textures
                    .borrow_mut()
                    .insert(unit_id.to_string(), tex.clone());
                return Some(tex);
            }
        }
        log::debug!("no sprite for {unit_id}");
        self.missing.borrow_mut().insert(unit_id.to_string());
        None
    }

    fn load(&self, ctx: &egui::Context, path: &std::path::Path, id: &str) -> Option<TextureHandle> {
        let img = image::open(path).ok()?;
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        Some(ctx.load_texture(id, image, TextureOptions::default()))
    }
}

/// Filename candidates for `unit_id`, most specific first: the id verbatim,
/// then the `SectionName`-convention form (`AliWadHelu_0_0` →
/// `Ali_Wad_Helu_0_0`, `JaalinII_3_1` → `Jaalin_II_3_1`).
fn filename_candidates(unit_id: &str) -> Vec<String> {
    let mut out = vec![format!("{unit_id}.webp")];
    if let Some((prefix, col, row)) = split_position(unit_id) {
        out.push(format!("{}_{}_{}.webp", camel_to_words(prefix), col, row));
    }
    out
}

/// Split `Prefix_col_row` into `(prefix, col, row)`.
fn split_position(unit_id: &str) -> Option<(&str, &str, &str)> {
    let mut parts = unit_id.rsplitn(3, '_');
    let row = parts.next()?;
    let col = parts.next()?;
    let prefix = parts.next()?;
    if row.is_empty() || col.is_empty() || prefix.is_empty() {
        return None;
    }
    Some((prefix, col, row))
}

/// CamelCase → underscore-separated words, keeping acronyms/roman numerals
/// whole: `AliWadHelu` → `Ali_Wad_Helu`, `JaalinII` → `Jaalin_II`.
/// Already-separated input passes through unchanged.
fn camel_to_words(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if i > 0
            && c.is_uppercase()
            && (chars[i - 1].is_lowercase()
                || chars.get(i + 1).is_some_and(|next| next.is_lowercase()))
        {
            out.push('_');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_splits() {
        assert_eq!(camel_to_words("AliWadHelu"), "Ali_Wad_Helu");
        assert_eq!(camel_to_words("JaalinII"), "Jaalin_II");
        assert_eq!(camel_to_words("JaalinI"), "Jaalin_I");
        assert_eq!(camel_to_words("MulazminII"), "Mulazmin_II");
        assert_eq!(camel_to_words("BritishArmy"), "British_Army");
        assert_eq!(camel_to_words("HadendowaForts"), "Hadendowa_Forts");
        assert_eq!(camel_to_words("SheikElDin"), "Sheik_El_Din");
        assert_eq!(camel_to_words("KhalifaAbdullah"), "Khalifa_Abdullah");
        assert_eq!(camel_to_words("OsmanDigna"), "Osman_Digna");
        assert_eq!(camel_to_words("Taiasha"), "Taiasha");
        assert_eq!(camel_to_words("Kitchener"), "Kitchener");
    }

    #[test]
    fn candidates() {
        assert_eq!(
            filename_candidates("AliWadHelu_2_1"),
            vec!["AliWadHelu_2_1.webp", "Ali_Wad_Helu_2_1.webp"]
        );
        assert_eq!(
            filename_candidates("JaalinII_3_1"),
            vec!["JaalinII_3_1.webp", "Jaalin_II_3_1.webp"]
        );
    }

    #[test]
    fn every_real_unit_id_resolves_to_a_sprite_file() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
        let ron =
            std::fs::read_to_string(base.join("Boardgame - Remember_Gordon/tables/units.ron"))
                .unwrap();
        let sprites = base.join("omdurman-app/assets/sprites");
        let ids: Vec<&str> = ron
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix('"'))
            .filter_map(|l| l.split('"').next())
            .filter(|s| split_position(s).is_some())
            .collect();
        assert!(
            ids.len() > 200,
            "expected the full counter set, got {}",
            ids.len()
        );

        let mut unresolved = Vec::new();
        for id in ids {
            let found = filename_candidates(id)
                .iter()
                .any(|f| sprites.join(f).exists());
            if !found {
                unresolved.push(id);
            }
        }
        assert!(
            unresolved.is_empty(),
            "no sprite file for {} unit ids: {unresolved:?}",
            unresolved.len()
        );
    }
}
