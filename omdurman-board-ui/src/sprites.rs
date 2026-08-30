//! The sprite-annotation resource: the loaded per-sprite annotations authored
//! offline by `tools/map-editor` (previously a struct defined in both
//! binaries).

use bevy::prelude::*;
use omdurman_types::SpriteAnnotations;

/// The loaded per-sprite annotations (possibly empty).
#[derive(Resource, Default, Deref)]
pub struct SpriteAnnotationsResource(pub SpriteAnnotations);
