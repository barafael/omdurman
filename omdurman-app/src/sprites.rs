//! Sprite-annotation support for the game's unit picker.
//!
//! The annotations themselves are authored offline by `tools/map-editor` and
//! loaded at startup from `assets/sprite_annotations.ron`
//! ([`crate::board_state::load_annotations`]); the picker overlays them on the
//! compiled `omdurman_rules::sprite_data` fallback.

use bevy::prelude::*;
use omdurman_types::{SectionName, SpriteAnnotations};

/// The loaded per-sprite annotations (possibly empty).
#[derive(Resource, Default, Deref)]
pub struct SpriteAnnotationsResource(pub SpriteAnnotations);

/// The canonical order in which counter sections appear, top to bottom.
/// Single source of truth: [`SectionName::SHEET_ORDER`] in `omdurman-types`,
/// shared with the map editor's sprite browser.
pub fn section_order() -> &'static [SectionName] {
    &SectionName::SHEET_ORDER
}
