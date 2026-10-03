//! The sprite-annotation resource: the loaded per-sprite annotations from
//! `assets/sprite_annotations.ron`.

use bevy::prelude::*;
use omdurman_types::SpriteAnnotations;

/// The loaded per-sprite annotations (possibly empty).
#[derive(Resource, Default, Deref)]
pub struct SpriteAnnotationsResource(pub SpriteAnnotations);
