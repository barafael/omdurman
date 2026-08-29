//! Tool-local editor tab state: which editing tool is active. The tool has no
//! game modes -- the tab bar is the top-level UI.

use bevy::prelude::States;

/// The editor's sub-tool, selected via the tab bar. Board-specific tabs
/// (Overlay, Terrain, Hexside) act on the active board; the rest are
/// board-agnostic.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EditorTab {
    /// Hex-grid alignment calibration over the map image.
    Overlay,
    /// Terrain painting / Nile flow / hex names / roads.
    #[default]
    Terrain,
    /// Hexside-feature (edge) editor.
    Hexside,
    /// Sprite-sheet cutting-grid editor.
    UnitSheet,
    /// Sprite browser (cut counters) + annotation editor.
    Sprites,
}

impl EditorTab {
    /// All tabs, in display order.
    pub const ALL: [EditorTab; 5] = [
        EditorTab::Overlay,
        EditorTab::Terrain,
        EditorTab::Hexside,
        EditorTab::UnitSheet,
        EditorTab::Sprites,
    ];

    /// Whether this tab edits the active board (vs board-agnostic tools).
    pub fn is_board_specific(self) -> bool {
        matches!(
            self,
            EditorTab::Overlay | EditorTab::Terrain | EditorTab::Hexside
        )
    }

    /// Whether this tab shows the map plane (vs the unit sheet / sprites).
    pub fn shows_map_plane(self) -> bool {
        !matches!(self, EditorTab::UnitSheet | EditorTab::Sprites)
    }

    pub fn label(self) -> &'static str {
        match self {
            EditorTab::Overlay => "Overlay",
            EditorTab::Terrain => "Terrain",
            EditorTab::Hexside => "Hexside",
            EditorTab::UnitSheet => "Unit sheet",
            EditorTab::Sprites => "Sprites",
        }
    }
}
