//! Line of Sight Table (rulebook §6.3, back cover).
//!
//! The LOS system uses a 3×3 matrix indexed by the firer's and target's
//! terrain level (`Ground`, `Rough`, `Hilltop`). Each cell lists which
//! intervening features (terrain, hexsides, units) block LOS, subject to
//! positional conditions (Details footnotes 1–7) and special notes (a–f).
//!
//! The authoritative source is `Boardgame - Remember_Gordon/tables/los_table.ron`.
//!
//! ## The table
//!
//! Locate the firing unit's level in the left column and the target's level
//! in the top row; the features in the intersecting box block line of sight,
//! subject to the superscript Details below.
//!
//! | Firer ↓ · Target → | Ground                                           | Rough                                            | Hilltop                          |
//! | :----------------- | :----------------------------------------------- | :----------------------------------------------- | :------------------------------- |
//! | **Ground**         | Units, Huts¹, Wall, Rough, Trees¹                | Units³ ⁶, Huts¹ ³, Wall, Crest², Trees¹, Hilltop | Units³, Huts¹ ³, Crest³, Hilltop |
//! | **Rough**          | Units⁴ ⁵, Huts¹ ⁴, Wall, Crest², Trees¹, Hilltop | Units⁷, Hilltop, Crest²                          | Units³, Crest² ³, Hilltop        |
//! | **Hilltop**        | Units⁴, Huts¹ ⁴, Crest⁴, Hilltop                 | Units⁴, Hilltop, Crest² ⁴                        | Units⁸                           |
//!
//! Each cell is a [`blocking_rules`] slice of [`BlockingRule`]s; a feature
//! blocks only when *all* of its [`LosCondition`]s hold. The cells are the
//! `static` in [`crate::tables_data`], transcribed from the RON file above.
//!
//! ## How it works
//!
//! 1. Determine the firer's LOS level from the terrain at the firing hex.
//! 2. Determine the target's LOS level from the terrain at the target hex.
//! 3. Look up the blocking rules for that `(firer, target)` pair.
//! 4. Walk the LOS ray hex by hex. For each intervening hex and hexside,
//!    check whether it matches a blocking feature and whether all positional
//!    conditions are satisfied. The ray is the exact segment between the two
//!    hex centres ([`los_ray_sides`]): every hex it passes through, a clipped
//!    corner included (note f), and every hexside it crosses.
//!
//! ## The three terrain levels
//!
//! - **Ground** — Clear, Swamp, Nile, Huts, Building (and forts per note c).
//! - **Rough** — Rough terrain (and gunboats / wall-adjacent city units per note b).
//! - **Hilltop** — Hilltop terrain.
//!
//! ## Detail footnotes (conditions)
//!
//! 1. Blocks only if the ray passes through more than two such features.
//! 2. Not blocked if the firer and/or target is adjacent to all crest hexsides
//!    fired through.
//! 3. Blocks only if the feature is closer to the firer, or halfway between.
//! 4. Blocks only if the feature is closer to the target, or halfway between.
//! 5. Blocks only if adjacent to, and at the same level as, the firing unit.
//! 6. Blocks only if adjacent to, and at the same level as, the target unit.
//! 7. Does not block if the feature is at a lower level.
//! 8. Only units on a hilltop block (the Hilltop → Hilltop box).
//!
//! ## Special LOS Notes
//!
//! - **(a)** Gunboats and forts never block LOS.
//! - **(b)** Gunboats and units inside a walled city adjacent to a wall
//!   hexside are considered at rough level.
//! - **(c)** Forts are considered at ground level.
//! - **(d)** Units may fire down (along the length of) one wall hexside.
//! - **(e)** Firing along the length of a crest hexside has the same effect
//!   as firing through it.
//! - **(f)** Terrain types fill their entire hex for LOS purposes.

use omdurman_types::{AlongHexside, HexCoord, HexsideKind, Terrain, UnitKind};

mod ray;
pub use ray::los_ray_sides;

// ─── Types ──────────────────────────────────────────────────────────────

/// Three terrain levels for LOS purposes (rulebook §6.3).
///
/// Ordered lowest to highest: `Ground < Rough < Hilltop`.
#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug,
)]
pub enum LosLevel {
    Ground,
    Rough,
    Hilltop,
}

impl LosLevel {
    /// Zero-based grid index (Ground=0, Rough=1, Hilltop=2) into
    /// `tables_data::LOS_CELLS`, matching the authored 3×3 table order (§6.3).
    pub fn index(self) -> usize {
        match self {
            LosLevel::Ground => 0,
            LosLevel::Rough => 1,
            LosLevel::Hilltop => 2,
        }
    }
}

/// A feature on the LOS ray that may block (rulebook §6.3).
///
/// The `Rough`/`Hilltop` table entries are named `RoughTerrain`/`HilltopTerrain`
/// in code (unambiguous against [`LosLevel`]); `serde` maps them back to the
/// authored RON spellings.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosFeature {
    /// A hex containing units (gunboats/forts excluded per note a).
    Units,
    /// Huts or Building terrain in an intervening hex (rulebook §5.44 groups
    /// "hut or building" together; Building is treated as Huts for LOS).
    Huts,
    /// Wall hexside crossed by the ray.
    Wall,
    /// Trees terrain in an intervening hex.
    Trees,
    /// Crest hexside crossed by the ray.
    Crest,
    /// Rough terrain as an intervening hex.
    #[serde(rename = "Rough")]
    RoughTerrain,
    /// Hilltop terrain as an intervening hex.
    #[serde(rename = "Hilltop")]
    HilltopTerrain,
}

impl std::fmt::Display for LosFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LosFeature::Units => "units",
            LosFeature::Huts => "huts",
            LosFeature::Wall => "a wall",
            LosFeature::Trees => "trees",
            LosFeature::Crest => "a crest",
            LosFeature::RoughTerrain => "rough ground",
            LosFeature::HilltopTerrain => "a hilltop",
        })
    }
}

/// A positional condition from the LOS table Detail footnotes.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LosCondition {
    /// (1) Blocks only if the ray passes through more than two such features.
    MoreThanTwo,
    /// (2) Not blocked if firer/target adjacent to all crest hexsides on ray.
    CrestAdjacency,
    /// (3) Blocks only if closer to firer, or halfway between.
    CloserToFirer,
    /// (4) Blocks only if closer to target, or halfway between.
    CloserToTarget,
    /// (5) Blocks only if adjacent to firer and at same level.
    AdjSameLevelFirer,
    /// (6) Blocks only if adjacent to target and at same level.
    AdjSameLevelTarget,
    /// (7) Does not block if the feature is at a lower level.
    NotAtLowerLevel,
    /// (Hilltop→Hilltop cell) Only units at hilltop level block.
    HilltopOnly,
}

/// One row of the authored LOS table (§6.3): a feature that may block, plus
/// the positional conditions (from the numbered Details) that must *all*
/// hold for it to block. The conditions are a `&'static` slice so the whole
/// table lives in `static` data (see [`crate::tables_data`]); the authored
/// RON's owned `Vec` form is mirrored by the parity tests. A tuple struct
/// to match the authored `(Units, [CloserToFirer])` form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlockingRule(pub LosFeature, pub &'static [LosCondition]);

/// The result of analysing one step along the LOS path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LosStepResult {
    /// This hex/hexside does not block LOS.
    Clear,
    /// LOS is blocked by this feature.
    Blocked { feature: LosFeature, hex: HexCoord },
    /// A wall or crest hexside between `a` and `b` blocks LOS.
    BlockedHexside {
        a: HexCoord,
        b: HexCoord,
        feature: LosFeature,
    },
}

// ─── Level mapping ──────────────────────────────────────────────────────

/// Map a terrain type to its LOS level (rulebook §6.3).
///
/// The terrain→level grouping is authored in
/// `Boardgame - Remember_Gordon/tables/los_table.ron` (embedded at compile
/// time by [`crate::tables_data`]); this strips the [`Terrain`] payloads the
/// table doesn't model and inverts that grouping. Terrains not listed
/// anywhere (Trees on the printed table) sit at ground level.
pub fn los_level(terrain: Terrain) -> LosLevel {
    use crate::tables_data::LosTerrainName as N;
    let name = match terrain {
        Terrain::Clear => N::Clear,
        Terrain::Rough => N::Rough,
        Terrain::Trees => N::Trees,
        Terrain::Swamp => N::Swamp,
        Terrain::Nile { .. } => N::Nile,
        Terrain::Hilltop => N::Hilltop,
        Terrain::Huts => N::Huts,
        Terrain::Building => N::Building,
    };
    for (level, names) in crate::tables_data::LOS_LEVELS {
        if names.contains(&name) {
            return level;
        }
    }
    LosLevel::Ground
}

/// Compute the LOS level of a unit at a given hex, applying Special LOS Notes
/// (b) and (c) (rulebook §6.3):
///
/// - **Note (b):** Gunboats are at rough level. Units inside a walled city
///   (Building terrain) adjacent to a wall hexside are at rough level.
/// - **Note (c):** Forts are at ground level.
///
/// For all other units, the level is derived from the terrain at `hex`.
pub fn los_level_for_unit(
    kind: UnitKind,
    hex: HexCoord,
    board: &crate::board::BoardInfo,
) -> LosLevel {
    // Note (b): gunboats are at rough level.
    if matches!(kind, UnitKind::Gunboat { .. }) {
        return LosLevel::Rough;
    }
    // Note (c): forts are at ground level.
    if matches!(kind, UnitKind::Fort { .. }) {
        return LosLevel::Ground;
    }
    let terrain = board.terrain_at(hex).unwrap_or_default();
    // Note (b): units inside a walled city adjacent to a wall hexside -- on
    // its ramparts -- are at rough level (not those outside it, whatever
    // their terrain).
    if hex
        .neighbors()
        .iter()
        .any(|n| board.is_inside_of_wall(hex, *n))
    {
        return LosLevel::Rough;
    }
    los_level(terrain)
}

// ─── Blocking rules table ──────────────────────────────────────────────

/// The blocking rules for a `(firer, target)` level pair (rulebook §6.3).
///
/// The table is the `static` 3×3 grid `tables_data::LOS_CELLS`, transcribed
/// from `Boardgame - Remember_Gordon/tables/los_table.ron` (parity-tested in
/// [`crate::tables_data`]). Each cell returns its [`BlockingRule`] entries in
/// printed order. A feature blocks only if ALL of its conditions are
/// satisfied (AND semantics); an empty conditions list means the feature
/// always blocks. Indexing is in-bounds by construction (both enums have
/// exactly three variants).
pub fn blocking_rules(firer: LosLevel, target: LosLevel) -> &'static [BlockingRule] {
    crate::tables_data::LOS_CELLS[firer.index()][target.index()]
}

// ─── Condition evaluation ──────────────────────────────────────────────

/// Context for evaluating positional conditions at a specific hex on the ray.
struct CondCtx {
    /// Twice this feature's distance from the firer, in hexes: `2d` for a
    /// hex at distance `d`, the sum of its two hexes' distances for a
    /// hexside -- which sits halfway between them, what "halfway between"
    /// (details 3/4) weighs.
    from_firer2: u32,
    /// Twice this feature's distance from the target, likewise.
    from_target2: u32,
    /// Cumulative count of hexes of this hex's feature (huts, or trees)
    /// seen so far, including this one (detail 1).
    hut_tree_count: usize,
    /// The LOS level of this hex's terrain.
    hex_level: LosLevel,
    /// The firer's LOS level.
    firer_level: LosLevel,
    /// The target's LOS level.
    target_level: LosLevel,
    /// Whether this hex is adjacent to the firer's hex.
    adjacent_to_firer: bool,
    /// Whether this hex is adjacent to the target's hex.
    adjacent_to_target: bool,
    /// Whether the crest-adjacency exception applies (firer/target adjacent
    /// to all crest hexsides on the ray).
    crest_adjacency_exception: bool,
    /// The LOS level of units in this hex (None = no blocking units).
    unit_level: Option<LosLevel>,
}

/// Evaluate whether a feature at this position blocks, given its conditions.
/// Returns `true` if ALL conditions are satisfied (feature blocks): a box
/// entry carrying two footnotes, e.g. "Units (3,6)", blocks only when both
/// hold. The printed table does not say how two footnotes combine; "and" is
/// the chosen reading (§6.3).
fn conditions_met(conditions: &[LosCondition], ctx: &CondCtx) -> bool {
    for &cond in conditions {
        let ok = match cond {
            LosCondition::MoreThanTwo => ctx.hut_tree_count > 2,
            LosCondition::CrestAdjacency => !ctx.crest_adjacency_exception,
            // Detail 3: "if closer to firing unit, or halfway between".
            LosCondition::CloserToFirer => ctx.from_firer2 <= ctx.from_target2,
            // Detail 4: "if closer to target unit, or half way between".
            LosCondition::CloserToTarget => ctx.from_firer2 >= ctx.from_target2,
            LosCondition::AdjSameLevelFirer => {
                ctx.adjacent_to_firer && ctx.hex_level == ctx.firer_level
            }
            LosCondition::AdjSameLevelTarget => {
                ctx.adjacent_to_target && ctx.hex_level == ctx.target_level
            }
            LosCondition::NotAtLowerLevel => {
                // "LOS not blocked if at lower level" — feature blocks
                // unless it's at a lower level than the firer.
                let feature_level = ctx.unit_level.unwrap_or(ctx.hex_level);
                feature_level >= ctx.firer_level
            }
            LosCondition::HilltopOnly => {
                // Authored form of the Hilltop→Hilltop special case: only
                // units at hilltop level block (a unit below the crest
                // doesn't interrupt hilltop-to-hilltop sight).
                ctx.unit_level.is_some_and(|lvl| lvl == LosLevel::Hilltop)
            }
        };
        if !ok {
            return false;
        }
    }
    true
}

// ─── Shared LOS walk ────────────────────────────────────────────────────

/// Where the LOS walk stopped: the ray is blocked (§6.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LosBlock {
    /// An intervening hex blocks via `feature` (terrain or units).
    Hex { hex: HexCoord, feature: LosFeature },
    /// The hexside between `a` and `b` blocks: crossed by the ray (`a` the
    /// hex before it, `b` the one beyond), or run along (notes d and e:
    /// `a` the hex of this ray, `b` the one across the hexside).
    Hexside {
        a: HexCoord,
        b: HexCoord,
        feature: LosFeature,
    },
}

/// Whether a unit in `hex` is adjacent to the hexside between the adjacent
/// hexes `a` and `b` -- detail 2's "adjacent to all crest hexsides fired
/// through" (§6.3). A hexside crossed by the ray is adjacent to the unit in
/// either hex sharing it; a hexside the ray runs along (note e) starts or
/// ends at a corner of the firer's or target's hex, which touches it there
/// and is adjacent to both hexes sharing it.
fn adjacent_to_hexside(hex: HexCoord, a: HexCoord, b: HexCoord) -> bool {
    hex == a || hex == b || (hex.is_adjacent_to(a) && hex.is_adjacent_to(b))
}

/// The single LOS walk behind both [`has_los`] and [`los_path_analysis`]
/// (§6.21, §6.3). Returns the ray path `[from, intervening..., to]` and, if
/// the ray is blocked, where and by what.
///
/// `path` is one side of the exact ray ([`los_ray_sides`]): every hex it
/// passes through, consecutive hexes sharing the hexside it crosses; `along`
/// the hexsides the line runs along ([`HexCoord::hexsides_along_line`]), the
/// same for either side.
///
/// `breached` reports whether the wall hexside between two hexes has been
/// breached (§6.53/§6.63): a breached wall is an opening and does not block.
///
/// Walk order is the rulebook's: intervening hexes first (each hex's terrain/
/// unit features), then the hexsides crossed, then those run along (notes d
/// and e). Stops at the first block, so a boolean caller pays no more than
/// the annotated path would.
#[allow(clippy::too_many_arguments)]
fn los_walk(
    board: &crate::board::BoardInfo,
    from: HexCoord,
    to: HexCoord,
    path: Vec<HexCoord>,
    along: &[AlongHexside],
    firer_level: LosLevel,
    target_level: LosLevel,
    unit_level_at: impl Fn(HexCoord) -> Option<LosLevel>,
    breached: impl Fn(HexCoord, HexCoord) -> bool,
) -> (Vec<HexCoord>, Option<LosBlock>) {
    let rules = blocking_rules(firer_level, target_level);

    // Adjacency check: is `hex` adjacent to `ref_hex`?
    let adjacent =
        |hex: HexCoord, ref_hex: HexCoord| -> bool { ref_hex.neighbors().contains(&hex) };

    // The hexside between two adjacent hexes as LOS sees it: a breached
    // wall is an opening (§6.53/§6.63).
    let hexside_kind = |a: HexCoord, b: HexCoord| -> Option<HexsideKind> {
        board.hexside_between(a, b).map(|hs| {
            if hs == HexsideKind::Wall && breached(a, b) {
                HexsideKind::Breach
            } else {
                hs
            }
        })
    };

    if path.len() < 2 {
        return (path, None); // same hex
    }

    // A hex's doubled distances from the firer and the target (details 3
    // and 4 weigh them; see [`CondCtx::from_firer2`]).
    let distances2 = |hex: HexCoord| (2 * from.distance(hex), 2 * to.distance(hex));

    // Every crest hexside "fired through" (detail 2): those the ray crosses,
    // between consecutive ray hexes, and those it runs along (note e:
    // "firing along the length of a crest hexside has the same effect on LOS
    // as firing through a crest hexside"). A crest on some other side of an
    // intervening hex is neither.
    let crest_hexsides: Vec<(HexCoord, HexCoord)> = path
        .windows(2)
        .map(|w| (w[0], w[1]))
        .chain(along.iter().map(|ah| (ah.a, ah.b)))
        .filter(|&(a, b)| hexside_kind(a, b) == Some(HexsideKind::Crest))
        .collect();

    // Condition 2: "Not blocked if firing units and/or target units are
    // adjacent to all crest hexsides fired through."
    let crest_adjacency_exception = !crest_hexsides.is_empty() && {
        let adjacent_to_all = |unit: HexCoord| {
            crest_hexsides
                .iter()
                .all(|&(a, b)| adjacent_to_hexside(unit, a, b))
        };
        adjacent_to_all(from) || adjacent_to_all(to)
    };

    // The conditions context of the hexside between `a` and `b`: halfway
    // between the two along the ray (see [`CondCtx::from_firer2`]),
    // `beyond` the one across it from this ray's own hex. Detail 1 is a hex
    // feature's; no hexside entry carries it.
    let hexside_ctx = |a: HexCoord, b: HexCoord, beyond: HexCoord| CondCtx {
        from_firer2: from.distance(a) + from.distance(b),
        from_target2: to.distance(a) + to.distance(b),
        hut_tree_count: 0,
        hex_level: los_level(board.terrain_at(beyond).unwrap_or_default()),
        firer_level,
        target_level,
        adjacent_to_firer: adjacent(beyond, from),
        adjacent_to_target: adjacent(beyond, to),
        crest_adjacency_exception,
        unit_level: unit_level_at(beyond),
    };

    // Track the cumulative counts for detail 1 ("if fire through more than
    // two" -- of *these*: huts, or trees, each counted on its own table
    // entry). Building counts as Huts (rulebook §5.44 groups "hut or
    // building" together).
    let mut huts_count = 0usize;
    let mut trees_count = 0usize;

    // Walk the ray, checking each intervening hex (the firer's and the
    // target's are not).
    for &hex in &path[1..path.len() - 1] {
        let terrain = board.terrain_at(hex).unwrap_or_default();
        let hex_level = los_level(terrain);
        let unit_level = unit_level_at(hex);

        // Update the cumulative huts / trees counts (Building counts as Huts).
        let hut_tree_count = match terrain {
            Terrain::Huts | Terrain::Building => {
                huts_count += 1;
                huts_count
            }
            Terrain::Trees => {
                trees_count += 1;
                trees_count
            }
            _ => 0,
        };

        let (from_firer2, from_target2) = distances2(hex);
        let ctx = CondCtx {
            from_firer2,
            from_target2,
            hut_tree_count,
            hex_level,
            firer_level,
            target_level,
            adjacent_to_firer: adjacent(hex, from),
            adjacent_to_target: adjacent(hex, to),
            crest_adjacency_exception,
            unit_level,
        };

        // Check each blocking rule against this hex's features.
        for rule in rules {
            let feature = rule.0;
            let conditions = rule.1;
            let feature_matches = match feature {
                LosFeature::Units => unit_level.is_some(),
                // Fix 3: Building treated as Huts (rulebook §5.44).
                LosFeature::Huts => {
                    matches!(terrain, Terrain::Huts | Terrain::Building)
                }
                LosFeature::Trees => matches!(terrain, Terrain::Trees),
                LosFeature::RoughTerrain => {
                    matches!(terrain, Terrain::Rough)
                }
                LosFeature::HilltopTerrain => {
                    matches!(terrain, Terrain::Hilltop)
                }
                // Wall and Crest are hexside features, checked separately.
                LosFeature::Wall | LosFeature::Crest => false,
            };

            // The Hilltop→Hilltop "only hilltop-level units block" special
            // case is authored as the HilltopOnly condition in the table.
            if feature_matches && conditions_met(conditions, &ctx) {
                return (path, Some(LosBlock::Hex { hex, feature }));
            }
        }
    }

    // The hexside features (Wall, Crest) and their table entry for this
    // level pair; a hexside of any other kind, or one the cell does not
    // list, never blocks.
    let hexside_rule = |hs: HexsideKind| -> Option<BlockingRule> {
        let feature = match hs {
            HexsideKind::Wall => LosFeature::Wall,
            HexsideKind::Crest => LosFeature::Crest,
            _ => return None,
        };
        rules.iter().find(|r| r.0 == feature).copied()
    };

    // Check crossed hexsides between consecutive ray hexes.
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        let Some(BlockingRule(feature, conditions)) = hexside_kind(a, b).and_then(hexside_rule)
        else {
            continue;
        };
        if conditions_met(conditions, &hexside_ctx(a, b, b)) {
            return (path, Some(LosBlock::Hexside { a, b, feature }));
        }
    }

    // Check the hexsides the ray runs along (notes d and e). Each sits
    // between this ray's hex at its step and the hex across it, on the
    // line's other side.
    let mut walls_along = 0usize;
    for ah in along {
        let Some(BlockingRule(feature, conditions)) =
            hexside_kind(ah.a, ah.b).and_then(hexside_rule)
        else {
            continue;
        };
        let (near, beyond) = if path.contains(&ah.a) {
            (ah.a, ah.b)
        } else {
            (ah.b, ah.a)
        };
        let blocks = match feature {
            // Note d: "units may fire down, i.e. along the length of, ONE
            // wall hexside" -- the second one blocks.
            LosFeature::Wall => {
                walls_along += 1;
                walls_along > 1
            }
            // Note e: the same effect as a crest fired through.
            _ => true,
        };
        if blocks && conditions_met(conditions, &hexside_ctx(near, beyond, beyond)) {
            return (
                path,
                Some(LosBlock::Hexside {
                    a: near,
                    b: beyond,
                    feature,
                }),
            );
        }
    }

    (path, None)
}

// ─── has_los ────────────────────────────────────────────────────────────

/// Whether the firer at `from` has line of sight to `to` (rulebook §6.21,
/// §6.3).
///
/// Howitzer fire ignores LOS entirely (§6.64), so it is always permitted.
///
/// `firer_level` and `target_level` are pre-computed by the caller using
/// [`los_level_for_unit`] (which applies Special Notes b and c). The
/// `unit_level_at` closure returns the LOS level of blocking units
/// (non-gunboat, non-fort per note a) in an intervening hex, or `None`.
/// The `breached` closure reports §6.53/§6.63 wall breaches, which do not
/// block (pass `|_, _| false` when the caller holds no game state, e.g. in
/// board-only tools).
#[allow(clippy::too_many_arguments)]
pub fn has_los(
    board: &crate::board::BoardInfo,
    from: HexCoord,
    to: HexCoord,
    kind: crate::FireKind,
    firer_level: LosLevel,
    target_level: LosLevel,
    unit_level_at: impl Fn(HexCoord) -> Option<LosLevel>,
    breached: impl Fn(HexCoord, HexCoord) -> bool,
) -> bool {
    use crate::FireKind;

    if kind == FireKind::Howitzer {
        return true;
    }

    los_rays(
        board,
        from,
        to,
        firer_level,
        target_level,
        unit_level_at,
        breached,
    )
    .1
    .is_none()
}

/// Walk the LOS ray from `from` to `to` (§6.3). A ray that touches hex
/// corners -- it runs along hexsides (notes d and e), or passes exactly
/// through a corner -- has two candidate hex paths, one on either side
/// ([`los_ray_sides`]); it is clear if either is clear (as note d lets units
/// fire down the length of a wall hexside), which also keeps line of sight
/// reciprocal. Returns the path walked -- the clear one if any -- and the
/// block on the first path when both are blocked.
fn los_rays(
    board: &crate::board::BoardInfo,
    from: HexCoord,
    to: HexCoord,
    firer_level: LosLevel,
    target_level: LosLevel,
    unit_level_at: impl Fn(HexCoord) -> Option<LosLevel>,
    breached: impl Fn(HexCoord, HexCoord) -> bool,
) -> (Vec<HexCoord>, Option<LosBlock>) {
    let along = from.hexsides_along_line(to);
    // (Kani cannot encode array-binding patterns; take the two sides
    // through the iterator instead.)
    let mut rays = los_ray_sides(from, to).into_iter();
    let side = rays.next().expect("the near side always exists");
    let other_side = rays.next().expect("the far side always exists");
    let first = los_walk(
        board,
        from,
        to,
        side,
        &along,
        firer_level,
        target_level,
        &unit_level_at,
        &breached,
    );
    if first.1.is_none() || other_side == first.0 {
        return first;
    }
    let second = los_walk(
        board,
        from,
        to,
        other_side,
        &along,
        firer_level,
        target_level,
        &unit_level_at,
        &breached,
    );
    if second.1.is_none() { second } else { first }
}

/// The clear ray(s) from `from` to `to` (§6.21, §6.3), each as the path
/// `[from, intervening..., to]`: the hexes the straight line passes
/// through, or, where it touches hex corners, whichever of its two sides
/// ([`los_ray_sides`]) is clear -- both when both are. Empty when line of
/// sight is blocked. Howitzer fire ignores LOS (§6.64): the line's first
/// side, always.
///
/// This is what a fire modifier that reads the line of fire -- the hexside
/// it enters the target through (§6.23), a thorn hedge it crosses (§9.231)
/// -- must look along: the shot is seen along a clear ray, which may be
/// either side of a tie.
#[allow(clippy::too_many_arguments)]
pub fn los_clear_rays(
    board: &crate::board::BoardInfo,
    from: HexCoord,
    to: HexCoord,
    kind: crate::FireKind,
    firer_level: LosLevel,
    target_level: LosLevel,
    unit_level_at: impl Fn(HexCoord) -> Option<LosLevel>,
    breached: impl Fn(HexCoord, HexCoord) -> bool,
) -> Vec<Vec<HexCoord>> {
    use crate::FireKind;

    // (Kani cannot encode array-binding patterns; take the two sides
    // through the iterator instead.)
    let mut rays = los_ray_sides(from, to).into_iter();
    let side = rays.next().expect("the near side always exists");
    let other_side = rays.next().expect("the far side always exists");
    if kind == FireKind::Howitzer {
        return vec![side];
    }
    let along = from.hexsides_along_line(to);
    let tied = side != other_side;
    let mut rays = Vec::with_capacity(2);
    for path in std::iter::once(side).chain(tied.then_some(other_side)) {
        let (path, block) = los_walk(
            board,
            from,
            to,
            path,
            &along,
            firer_level,
            target_level,
            &unit_level_at,
            &breached,
        );
        if block.is_none() {
            rays.push(path);
        }
    }
    rays
}

// ─── los_path_analysis ──────────────────────────────────────────────────

/// Annotate every step of the LOS ray from `from` to `to` (§6.21, §6.3).
///
/// Returns a list of `(hex, step_result)` pairs. The first entry is always
/// `(from, Clear)`. If a step blocks, subsequent steps are not included (a
/// blocking hexside is reported as a terminal [`LosStepResult::BlockedHexside`]
/// record).
///
/// Howitzer fire bypasses LOS (§6.64); every step is `Clear`.
///
/// Like [`has_los`], this takes pre-computed `firer_level` and `target_level`
/// (use [`los_level_for_unit`] at the call site) and a `breached` closure for
/// §6.53/§6.63 wall breaches (pass `|_, _| false` when no game state is at
/// hand).
#[allow(clippy::too_many_arguments)]
pub fn los_path_analysis(
    board: &crate::board::BoardInfo,
    from: HexCoord,
    to: HexCoord,
    kind: crate::FireKind,
    firer_level: LosLevel,
    target_level: LosLevel,
    unit_level_at: impl Fn(HexCoord) -> Option<LosLevel>,
    breached: impl Fn(HexCoord, HexCoord) -> bool,
) -> Vec<(HexCoord, LosStepResult)> {
    use crate::FireKind;

    // Howitzer fire bypasses LOS (§6.64); every step is `Clear`.
    if kind == FireKind::Howitzer {
        let path = los_ray_sides(from, to)
            .into_iter()
            .next()
            .expect("the near side always exists");
        return path
            .into_iter()
            .map(|h| (h, LosStepResult::Clear))
            .collect();
    }

    let (path, block) = los_rays(
        board,
        from,
        to,
        firer_level,
        target_level,
        unit_level_at,
        breached,
    );

    let mut result: Vec<(HexCoord, LosStepResult)> = vec![(from, LosStepResult::Clear)];
    match block {
        // Unblocked: every remaining step (including the target hex) is clear.
        None => {
            for hex in path.into_iter().skip(1) {
                result.push((hex, LosStepResult::Clear));
            }
        }
        // A blocking hex: clear steps up to it, then the blocked record.
        Some(LosBlock::Hex { hex, feature }) => {
            for step in path.into_iter().skip(1) {
                if step == hex {
                    result.push((hex, LosStepResult::Blocked { feature, hex }));
                    break;
                }
                result.push((step, LosStepResult::Clear));
            }
        }
        // A blocking hexside: the whole path is clear, then the terminal
        // hexside record.
        Some(LosBlock::Hexside { a, b, feature }) => {
            for hex in path.into_iter().skip(1) {
                result.push((hex, LosStepResult::Clear));
            }
            result.push((b, LosStepResult::BlockedHexside { a, b, feature }));
        }
    }
    result
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FireKind;
    use crate::board::BoardInfo;
    use omdurman_types::{GroundKind, HexsideKind, HexsideRef, Terrain};
    use strum::IntoEnumIterator;
    use traceability_macro::rulebook;

    fn board_with_terrain(hexes: &[(i32, i32, Terrain)]) -> BoardInfo {
        let mut board = BoardInfo::default();
        for &(q, r, t) in hexes {
            board.terrain.insert(HexCoord::new(q, r), t);
        }
        board
    }

    fn board_with_hexsides(
        hexes: &[(i32, i32, Terrain)],
        sides: &[(HexCoord, HexCoord, HexsideKind)],
    ) -> BoardInfo {
        let mut board = board_with_terrain(hexes);
        for &(a, b, k) in sides {
            board.hexsides.insert(HexsideRef::new(a, b), k);
        }
        board
    }

    /// No-unit closure for tests that don't need unit blocking.
    fn no_units() -> impl Fn(HexCoord) -> Option<LosLevel> {
        |_| None
    }

    /// Test convenience: call `has_los` with firer/target levels auto-derived
    /// from terrain (the common case for tests that don't test note b/c).
    fn has_los_auto(
        board: &BoardInfo,
        from: HexCoord,
        to: HexCoord,
        kind: crate::FireKind,
        units: impl Fn(HexCoord) -> Option<LosLevel>,
    ) -> bool {
        let fl = board
            .terrain_at(from)
            .map(los_level)
            .unwrap_or(LosLevel::Ground);
        let tl = board
            .terrain_at(to)
            .map(los_level)
            .unwrap_or(LosLevel::Ground);
        has_los(board, from, to, kind, fl, tl, units, |_, _| false)
    }

    // ── Level mapping ──

    #[rulebook("§6.3")]
    #[test]
    fn los_level_mapping() {
        assert_eq!(
            los_level(Terrain::ground(GroundKind::Clear)),
            LosLevel::Ground
        );
        assert_eq!(
            los_level(Terrain::ground(GroundKind::Rough)),
            LosLevel::Rough
        );
        assert_eq!(
            los_level(Terrain::ground(GroundKind::Hilltop)),
            LosLevel::Hilltop
        );
        assert_eq!(
            los_level(Terrain::ground(GroundKind::Huts)),
            LosLevel::Ground
        );
        assert_eq!(
            los_level(Terrain::ground(GroundKind::Trees)),
            LosLevel::Ground
        );
        assert_eq!(
            los_level(Terrain::ground(GroundKind::Building)),
            LosLevel::Ground
        );
        assert_eq!(
            los_level(Terrain::ground(GroundKind::Swamp)),
            LosLevel::Ground
        );
    }

    // ── Basic has_los tests ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_empty_board_is_clear() {
        let board = BoardInfo::default();
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(5, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_adjacent_clear() {
        let board = board_with_terrain(&[(0, 0, Terrain::default()), (1, 0, Terrain::default())]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_howitzer_bypasses() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Wall)]);
        assert!(has_los_auto(&board, a, b, FireKind::Howitzer, no_units()));
    }

    // ── Wall hexside blocking ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_wall_hexside_blocks_ground_to_ground() {
        // Ground→Ground: Wall always blocks
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Wall)]);
        assert!(!has_los_auto(&board, a, b, FireKind::Direct, no_units()));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_gate_hexside_passes() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Gate)]);
        assert!(has_los_auto(&board, a, b, FireKind::Direct, no_units()));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_breach_hexside_passes() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(&[], &[(a, b, HexsideKind::Breach)]);
        assert!(has_los_auto(&board, a, b, FireKind::Direct, no_units()));
    }

    // ── Terrain blocking (Ground→Ground cell) ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_rough_intervening_blocks_ground_to_ground() {
        // Ground→Ground: Rough terrain always blocks
        let board = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Rough))]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_two_tree_hexes_pass_ground_to_ground() {
        // Ground→Ground: Trees block only if >2 (footnote 1)
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Trees)),
            (2, 0, Terrain::ground(GroundKind::Trees)),
        ]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(3, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_three_tree_hexes_block_ground_to_ground() {
        // Ground→Ground: Trees block if >2 (footnote 1)
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Trees)),
            (2, 0, Terrain::ground(GroundKind::Trees)),
            (3, 0, Terrain::ground(GroundKind::Trees)),
        ]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(4, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_two_hut_hexes_pass_ground_to_ground() {
        // Ground→Ground: Huts block only if >2 (footnote 1)
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Huts)),
            (2, 0, Terrain::ground(GroundKind::Huts)),
        ]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(3, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_three_hut_hexes_block_ground_to_ground() {
        // Ground→Ground: Huts block if >2 (footnote 1)
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Huts)),
            (2, 0, Terrain::ground(GroundKind::Huts)),
            (3, 0, Terrain::ground(GroundKind::Huts)),
        ]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(4, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    // ── Hilltop→Ground: Units (4) -- only units nearer the target block ──

    /// The printed Hilltop→Ground cell reads "Units (4)": a unit blocks only
    /// "if closer to target unit, or half way between" -- looking down from
    /// a hilltop, a unit at the firer's feet does not hide the plain beyond.
    #[rulebook("§6.3")]
    #[test]
    fn has_los_hilltop_to_ground_units_block_only_nearer_the_target() {
        let board = board_with_terrain(&[(0, 0, Terrain::ground(GroundKind::Hilltop))]);
        let from = HexCoord::new(0, 0);
        let to = HexCoord::new(4, 0);
        let unit_at = |at: HexCoord| move |hex: HexCoord| (hex == at).then_some(LosLevel::Ground);
        // Nearer the firer (step 1 of 4): does not block.
        assert!(has_los_auto(
            &board,
            from,
            to,
            FireKind::Direct,
            unit_at(HexCoord::new(1, 0))
        ));
        // Halfway (step 2 of 4): blocks.
        assert!(!has_los_auto(
            &board,
            from,
            to,
            FireKind::Direct,
            unit_at(HexCoord::new(2, 0))
        ));
        // Nearer the target (step 3 of 4): blocks.
        assert!(!has_los_auto(
            &board,
            from,
            to,
            FireKind::Direct,
            unit_at(HexCoord::new(3, 0))
        ));
    }

    // ── Hilltop→Hilltop: only units on a hilltop block ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_hilltop_to_hilltop_clear_no_units() {
        let board = board_with_terrain(&[
            (0, 0, Terrain::ground(GroundKind::Hilltop)),
            (1, 0, Terrain::ground(GroundKind::Huts)), // would normally block
            (2, 0, Terrain::ground(GroundKind::Hilltop)),
        ]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_hilltop_to_hilltop_blocked_by_hilltop_unit() {
        let board = board_with_terrain(&[
            (0, 0, Terrain::ground(GroundKind::Hilltop)),
            (1, 0, Terrain::ground(GroundKind::Hilltop)),
            (2, 0, Terrain::ground(GroundKind::Hilltop)),
        ]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            |_| Some(LosLevel::Hilltop), // unit on hilltop at (1,0)
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_hilltop_to_hilltop_not_blocked_by_ground_unit() {
        let board = board_with_terrain(&[
            (0, 0, Terrain::ground(GroundKind::Hilltop)),
            (1, 0, Terrain::default()),
            (2, 0, Terrain::ground(GroundKind::Hilltop)),
        ]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            |_| Some(LosLevel::Ground), // unit at ground level
        ));
    }

    // ── Rough→Rough: Units (7) — not blocked if at lower level ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_rough_to_rough_unit_at_lower_level_passes() {
        let board = board_with_terrain(&[
            (0, 0, Terrain::ground(GroundKind::Rough)),
            (1, 0, Terrain::default()), // ground level intervening
            (2, 0, Terrain::ground(GroundKind::Rough)),
        ]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            |_| Some(LosLevel::Ground), // unit at ground (lower) level
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_rough_to_rough_unit_at_same_level_blocks() {
        let board = board_with_terrain(&[
            (0, 0, Terrain::ground(GroundKind::Rough)),
            (1, 0, Terrain::ground(GroundKind::Rough)),
            (2, 0, Terrain::ground(GroundKind::Rough)),
        ]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            |_| Some(LosLevel::Rough), // unit at rough (same) level
        ));
    }

    // ── Rough→Rough: Hilltop terrain always blocks ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_rough_to_rough_hilltop_blocks() {
        let board = board_with_terrain(&[
            (0, 0, Terrain::ground(GroundKind::Rough)),
            (1, 0, Terrain::ground(GroundKind::Hilltop)),
            (2, 0, Terrain::ground(GroundKind::Rough)),
        ]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    // ── Ground→Hilltop: Hilltop terrain blocks ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_ground_to_hilltop_intervening_hilltop_blocks() {
        let _board = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Hilltop))]);
        // target at (2,0) is hilltop
        let board = board_with_terrain(&[
            (0, 0, Terrain::default()),
            (1, 0, Terrain::ground(GroundKind::Hilltop)),
            (2, 0, Terrain::ground(GroundKind::Hilltop)),
        ]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    // ── All 9 cells compile (exhaustive match check) ──

    #[rulebook("§6.3")]
    #[test]
    fn blocking_rules_all_cells_covered() {
        for firer in [LosLevel::Ground, LosLevel::Rough, LosLevel::Hilltop] {
            for target in [LosLevel::Ground, LosLevel::Rough, LosLevel::Hilltop] {
                let rules = blocking_rules(firer, target);
                assert!(
                    !rules.is_empty(),
                    "cell ({firer:?},{target:?}) has no rules"
                );
            }
        }
    }

    // ── Fix 3: Building treated as Huts (§5.44) ──

    #[rulebook("§6.3")]
    #[test]
    fn has_los_building_blocks_like_huts_ground_to_ground() {
        // Ground→Ground: Huts (with >2 condition) blocks. Building should
        // behave the same.
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Building)),
            (2, 0, Terrain::ground(GroundKind::Building)),
            (3, 0, Terrain::ground(GroundKind::Building)),
        ]);
        assert!(!has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(4, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn has_los_two_building_hexes_pass_ground_to_ground() {
        // Ground→Ground: Huts (and Building) block only if >2.
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Building)),
            (2, 0, Terrain::ground(GroundKind::Building)),
        ]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(3, 0),
            FireKind::Direct,
            no_units(),
        ));
    }

    // ── Fix 1: Notes (b) and (c) — gunboat/fort level classification ──

    #[rulebook("§6.3")]
    #[test]
    fn los_level_for_unit_gunboat_is_rough() {
        let board = BoardInfo::default(); // no terrain needed
        assert_eq!(
            los_level_for_unit(
                UnitKind::Gunboat {
                    fire: 0,
                    upstream: 0,
                    downstream: 0
                },
                HexCoord::new(0, 0),
                &board
            ),
            LosLevel::Rough
        );
    }

    #[rulebook("§6.3")]
    #[test]
    fn los_level_for_unit_fort_is_ground() {
        let board = board_with_terrain(&[(0, 0, Terrain::ground(GroundKind::Hilltop))]);
        assert_eq!(
            los_level_for_unit(
                UnitKind::Fort { fire: 0, melee: 0 },
                HexCoord::new(0, 0),
                &board
            ),
            LosLevel::Ground // even on a hilltop, fort is Ground (note c)
        );
    }

    #[rulebook("§6.3")]
    #[test]
    fn los_level_for_unit_walled_city_adj_wall_is_rough() {
        // Note b: "units inside a walled city adjacent to a wall hexside" --
        // the city side of the wall only, whatever the terrain outside.
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let mut board = board_with_hexsides(
            &[
                (0, 0, Terrain::ground(GroundKind::Building)),
                (1, 0, Terrain::ground(GroundKind::Building)),
            ],
            &[(a, b, HexsideKind::Wall)],
        );
        board.walled_city.insert(a);
        let infantry = UnitKind::Infantry {
            fire: 0,
            melee: 0,
            movement: 0,
        };
        assert_eq!(los_level_for_unit(infantry, a, &board), LosLevel::Rough);
        assert_eq!(los_level_for_unit(infantry, b, &board), LosLevel::Ground);
    }

    #[rulebook("§6.3")]
    #[test]
    fn gunboat_firer_uses_rough_row_not_ground() {
        // 3 hut hexes close to the firer, then clear terrain to the target.
        // Ground→Ground: Huts (1) blocks if >2 → always blocks with 3.
        // Rough→Ground: Huts (1,4) blocks if >2 AND closer to target.
        // With huts at positions 1,2,3 and target at position 8, the huts
        // are closer to the firer (not target), so Rough→Ground does NOT block.
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Huts)),
            (2, 0, Terrain::ground(GroundKind::Huts)),
            (3, 0, Terrain::ground(GroundKind::Huts)),
        ]);
        // Ground firer: Huts (>2) blocks unconditionally
        assert!(!has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(8, 0),
            FireKind::Direct,
            LosLevel::Ground,
            LosLevel::Ground,
            no_units(),
            |_, _| false,
        ));
        // Rough firer (gunboat): Huts (1,4) — blocks only if >2 AND closer
        // to target. Huts are closer to firer, so does NOT block.
        assert!(has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(8, 0),
            FireKind::Direct,
            LosLevel::Rough,
            LosLevel::Ground,
            no_units(),
            |_, _| false,
        ));
    }

    // ── Hexsides the ray runs along (notes d and e) ──────────────────────
    //
    // The corner-direction lines used here and their along-hexsides (see
    // `HexCoord::hexsides_along_line`):
    //   (0,0) -> (2,1): step 1 (1,1)|(1,0)
    //   (0,0) -> (4,2): step 1 (1,1)|(1,0), step 3 (3,2)|(3,1)
    //   (0,0) -> (6,3): step 1 (1,1)|(1,0), step 3 (3,2)|(3,1), step 5 (5,3)|(5,2)

    /// `has_los` with explicit levels and no units or breaches.
    fn has_los_levels(
        board: &BoardInfo,
        from: HexCoord,
        to: HexCoord,
        firer: LosLevel,
        target: LosLevel,
    ) -> bool {
        has_los(
            board,
            from,
            to,
            FireKind::Direct,
            firer,
            target,
            no_units(),
            |_, _| false,
        )
    }

    /// Note e is about crest hexsides the ray runs *along*: a crest on a
    /// lateral side of an intervening hex -- one the straight ray neither
    /// crosses nor follows -- is not "fired through" and blocks nothing, in
    /// every cell whose Crest entry could otherwise block here (Ground→Rough,
    /// Ground→Hilltop, Hilltop→Ground).
    #[rulebook("§6.3")]
    #[test]
    fn a_crest_beside_a_straight_ray_does_not_block() {
        let from = HexCoord::new(0, 0);
        let to = HexCoord::new(4, 0);
        let board = board_with_hexsides(
            &[],
            &[(HexCoord::new(2, 0), HexCoord::new(2, 1), HexsideKind::Crest)],
        );
        assert!(from.hexsides_along_line(to).is_empty());
        for (firer, target) in [
            (LosLevel::Ground, LosLevel::Rough),
            (LosLevel::Ground, LosLevel::Hilltop),
            (LosLevel::Hilltop, LosLevel::Ground),
            (LosLevel::Rough, LosLevel::Rough),
        ] {
            assert!(
                has_los_levels(&board, from, to, firer, target),
                "{firer:?} -> {target:?}"
            );
        }
    }

    /// Note e: "firing along the length of a crest hexside has the same
    /// effect on LOS as firing through a crest hexside" -- with the crest's
    /// positional footnote weighed at the step the ray runs along it.
    /// Ground→Hilltop's Crest (3) blocks "if closer to firing unit, or
    /// halfway between"; Hilltop→Ground's Crest (4) the mirror image.
    #[rulebook("§6.3")]
    #[test]
    fn a_crest_the_ray_runs_along_blocks_like_one_fired_through() {
        let from = HexCoord::new(0, 0);
        let to = HexCoord::new(4, 2);
        let near = (HexCoord::new(1, 1), HexCoord::new(1, 0)); // step 1 of 4
        let far = (HexCoord::new(3, 2), HexCoord::new(3, 1)); // step 3 of 4
        let crest_at =
            |(a, b): (HexCoord, HexCoord)| board_with_hexsides(&[], &[(a, b, HexsideKind::Crest)]);
        let (g, h) = (LosLevel::Ground, LosLevel::Hilltop);
        // Ground firer, hilltop target: the crest nearer the firer blocks,
        // the one nearer the target does not.
        assert!(!has_los_levels(&crest_at(near), from, to, g, h));
        assert!(has_los_levels(&crest_at(far), from, to, g, h));
        // Hilltop firer, ground target: the other way round.
        assert!(has_los_levels(&crest_at(near), from, to, h, g));
        assert!(!has_los_levels(&crest_at(far), from, to, h, g));
        // The analysis names the crest hexside run along, on this ray's
        // side: the ray walked (1,1) and the crest lies between it and (1,0).
        let steps = los_path_analysis(
            &crest_at(near),
            from,
            to,
            FireKind::Direct,
            g,
            h,
            no_units(),
            |_, _| false,
        );
        assert_eq!(
            steps.last().map(|s| s.1),
            Some(LosStepResult::BlockedHexside {
                a: HexCoord::new(1, 1),
                b: HexCoord::new(1, 0),
                feature: LosFeature::Crest,
            })
        );
    }

    /// Detail 2 on a crest run along: "not blocked if firing units and/or
    /// target units are adjacent to all crest hexsides fired through". The
    /// hexside starts at a corner of the firer's hex (or ends at one of the
    /// target's), which is adjacent to both hexes sharing it; a crest run
    /// along in the middle of the ray is adjacent to neither, and one such
    /// crest spoils the firer's adjacency to "all" of them.
    #[rulebook("§6.3")]
    #[test]
    fn crest_adjacency_rescues_a_crest_the_ray_runs_along() {
        let from = HexCoord::new(0, 0);
        let to = HexCoord::new(6, 3);
        let step1 = (HexCoord::new(1, 1), HexCoord::new(1, 0));
        let step3 = (HexCoord::new(3, 2), HexCoord::new(3, 1));
        let step5 = (HexCoord::new(5, 3), HexCoord::new(5, 2));
        let crests = |sides: &[(HexCoord, HexCoord)]| {
            let sides: Vec<_> = sides
                .iter()
                .map(|&(a, b)| (a, b, HexsideKind::Crest))
                .collect();
            board_with_hexsides(&[], &sides)
        };
        let (g, r) = (LosLevel::Ground, LosLevel::Rough);
        // Ground→Rough: Crest (2). Mid-ray: blocked.
        assert!(!has_los_levels(&crests(&[step3]), from, to, g, r));
        // At the firer's corner: the firer is adjacent to it.
        assert!(has_los_levels(&crests(&[step1]), from, to, g, r));
        // At the target's corner: the target is adjacent to it.
        assert!(has_los_levels(&crests(&[step5]), from, to, g, r));
        // Both ends: neither unit is adjacent to *all* of them.
        assert!(!has_los_levels(&crests(&[step1, step5]), from, to, g, r));
        // A crossed crest and a run-along one, both at the firer: clear.
        let mut both = crests(&[step1]);
        both.hexsides.insert(
            HexsideRef::new(from, HexCoord::new(1, 1)),
            HexsideKind::Crest,
        );
        assert!(has_los_levels(&both, from, to, g, r));
        // Rough→Rough reads the same Crest (2) entry.
        assert!(!has_los_levels(&crests(&[step3]), from, to, r, r));
        assert!(has_los_levels(&crests(&[step1]), from, to, r, r));
    }

    /// Note d: "units may fire down, i.e. along the length of, one wall
    /// hexside" -- the ray may run along one wall hexside, not two. A
    /// breach or a gate in one of them is no wall. A cell without a Wall
    /// entry (a hilltop firer) is not concerned.
    #[rulebook("§6.3")]
    #[test]
    fn fire_along_one_wall_hexside_is_clear_along_two_blocked() {
        let from = HexCoord::new(0, 0);
        let near = (HexCoord::new(1, 1), HexCoord::new(1, 0));
        let far = (HexCoord::new(3, 2), HexCoord::new(3, 1));
        let walls = |sides: &[(HexCoord, HexCoord, HexsideKind)]| board_with_hexsides(&[], sides);
        let g = LosLevel::Ground;
        // One wall hexside, the whole ray long: clear.
        let one = walls(&[(near.0, near.1, HexsideKind::Wall)]);
        assert!(has_los_levels(&one, from, HexCoord::new(2, 1), g, g));
        // Two wall hexsides along a longer ray: blocked, on the second.
        let two = walls(&[
            (near.0, near.1, HexsideKind::Wall),
            (far.0, far.1, HexsideKind::Wall),
        ]);
        let to = HexCoord::new(4, 2);
        assert!(!has_los_levels(&two, from, to, g, g));
        let steps = los_path_analysis(
            &two,
            from,
            to,
            FireKind::Direct,
            g,
            g,
            no_units(),
            |_, _| false,
        );
        assert!(matches!(
            steps.last().map(|s| s.1),
            Some(LosStepResult::BlockedHexside {
                a: HexCoord { q: 3, .. },
                feature: LosFeature::Wall,
                ..
            })
        ));
        // Ground→Rough and Rough→Ground list the wall too.
        assert!(!has_los_levels(&two, from, to, g, LosLevel::Rough));
        assert!(!has_los_levels(&two, from, to, LosLevel::Rough, g));
        // A breached wall (§6.63) is an opening: one wall left.
        assert!(has_los(
            &two,
            from,
            to,
            FireKind::Direct,
            g,
            g,
            no_units(),
            |a, b| HexsideRef::new(a, b) == HexsideRef::new(far.0, far.1),
        ));
        // A gate beside one wall: one wall.
        let gate = walls(&[
            (near.0, near.1, HexsideKind::Wall),
            (far.0, far.1, HexsideKind::Gate),
        ]);
        assert!(has_los_levels(&gate, from, to, g, g));
        // A hilltop firer shoots over walls (no Wall entry in its cells).
        assert!(has_los_levels(&two, from, to, LosLevel::Hilltop, g));
    }

    /// The clear rays of a shot (`los_clear_rays`): the straight line when
    /// it hits no tie; on a tie, whichever side is clear -- both when both
    /// are, none when neither; howitzer fire sees the straight line always
    /// (§6.64).
    #[rulebook("§6.3")]
    #[test]
    fn los_clear_rays_are_the_clear_sides_of_the_line() {
        let from = HexCoord::new(0, 0);
        let to = HexCoord::new(2, 1);
        let g = LosLevel::Ground;
        let rays = |board: &BoardInfo, kind: FireKind, units: &dyn Fn(HexCoord) -> bool| {
            los_clear_rays(
                board,
                from,
                to,
                kind,
                g,
                g,
                |h| units(h).then_some(g),
                |_, _| false,
            )
        };
        let open = BoardInfo::default();
        let via = |mid: (i32, i32)| vec![from, HexCoord::new(mid.0, mid.1), to];
        assert_eq!(
            rays(&open, FireKind::Direct, &|_| false),
            vec![via((1, 1)), via((1, 0))]
        );
        // A unit on one side leaves the other.
        let at = |h: HexCoord| move |x: HexCoord| x == h;
        assert_eq!(
            rays(&open, FireKind::Direct, &at(HexCoord::new(1, 1))),
            vec![via((1, 0))]
        );
        assert_eq!(
            rays(&open, FireKind::Direct, &at(HexCoord::new(1, 0))),
            vec![via((1, 1))]
        );
        // Units on both: blocked, no ray.
        assert!(rays(&open, FireKind::Direct, &|_| true).is_empty());
        // Howitzer fire: the straight line, whatever stands in it.
        assert_eq!(
            rays(&open, FireKind::Howitzer, &|_| true),
            vec![via((1, 1))]
        );
        // No tie: the one line.
        assert_eq!(
            los_clear_rays(
                &open,
                from,
                HexCoord::new(2, 0),
                FireKind::Direct,
                g,
                g,
                no_units(),
                |_, _| false,
            ),
            vec![vec![from, HexCoord::new(1, 0), HexCoord::new(2, 0)]]
        );
    }

    // ── The exact ray (note f): every hex passed, every hexside crossed ──

    /// A unit standing at any of `hexes`, at ground level.
    fn units_at(hexes: &[HexCoord]) -> impl Fn(HexCoord) -> Option<LosLevel> + '_ {
        move |h| hexes.contains(&h).then_some(LosLevel::Ground)
    }

    /// (0,0) -> (1,4) crosses the (0,1)|(1,2) hexside, then the
    /// (1,2)|(0,2) hexside exactly at its midpoint. It passes through both
    /// of those hexes -- a unit in either blocks -- and a wall on the
    /// hexside it crosses blocks, whatever the hexside the sampled line's
    /// "other side" would have stepped through (a gate here). The Campaign
    /// fire from (30,37) into Omdurman at (31,41) is this ray.
    #[rulebook("§6.3")]
    #[test]
    fn a_ray_through_a_hexside_midpoint_passes_both_hexes_and_crosses_it() {
        let from = HexCoord::new(0, 0);
        let to = HexCoord::new(1, 4);
        let g = LosLevel::Ground;
        let open = BoardInfo::default();
        let sees = |board: &BoardInfo, units: &[HexCoord]| {
            has_los(
                board,
                from,
                to,
                FireKind::Direct,
                g,
                g,
                units_at(units),
                |_, _| false,
            )
        };
        assert!(sees(&open, &[]));
        assert!(!sees(&open, &[HexCoord::new(1, 2)]));
        assert!(!sees(&open, &[HexCoord::new(0, 2)]));
        let walled = board_with_hexsides(
            &[],
            &[
                (HexCoord::new(0, 1), HexCoord::new(1, 2), HexsideKind::Wall),
                (HexCoord::new(0, 1), HexCoord::new(0, 2), HexsideKind::Gate),
            ],
        );
        assert!(!sees(&walled, &[]));
        let steps = los_path_analysis(
            &walled,
            from,
            to,
            FireKind::Direct,
            g,
            g,
            no_units(),
            |_, _| false,
        );
        assert_eq!(
            steps.last().map(|s| s.1),
            Some(LosStepResult::BlockedHexside {
                a: HexCoord::new(0, 1),
                b: HexCoord::new(1, 2),
                feature: LosFeature::Wall,
            })
        );
    }

    /// The same ray on the Campaign board: Ground→Ground fire at range 4
    /// from (30,37), outside Omdurman's wall, at (31,41) inside the city
    /// crosses the wall hexside (30,38)|(31,39) and is blocked (the sampled
    /// line walked (30,38)->(30,39) through the gate beside it). Reciprocal.
    #[rulebook("§6.3")]
    #[test]
    fn campaign_fire_into_omdurman_through_the_wall_is_blocked() {
        let board = crate::board::BoardInfo::from_map_data(crate::board_data::campaign_map_data());
        let (outside, inside) = (HexCoord::new(30, 37), HexCoord::new(31, 41));
        let (a, b) = (HexCoord::new(30, 38), HexCoord::new(31, 39));
        assert_eq!(board.hexside_between(a, b), Some(HexsideKind::Wall));
        let g = LosLevel::Ground;
        let steps = los_path_analysis(
            &board,
            outside,
            inside,
            FireKind::Direct,
            g,
            g,
            no_units(),
            |_, _| false,
        );
        assert!(
            steps.iter().any(|s| matches!(
                s.1,
                LosStepResult::BlockedHexside {
                    feature: LosFeature::Wall,
                    ..
                }
            )),
            "{steps:?}"
        );
        let sees = |x, y| {
            has_los(&board, x, y, FireKind::Direct, g, g, no_units(), |_, _| {
                false
            })
        };
        assert!(!sees(outside, inside));
        assert!(!sees(inside, outside));
        // Breach the wall hexside (§6.63) and the shot is clear.
        assert!(has_los(
            &board,
            outside,
            inside,
            FireKind::Direct,
            g,
            g,
            no_units(),
            |x, y| HexsideRef::new(x, y) == HexsideRef::new(a, b),
        ));
    }

    /// At range 7 the ray (0,0) -> (1,7) clips the corners of hexes the
    /// one-sample-per-hex line skips; terrain fills its whole hex (note f),
    /// so rough ground in a clipped hex blocks Ground→Ground fire. Likewise
    /// at ranges 9 and 10.
    #[rulebook("§6.3")]
    #[test]
    fn rough_ground_in_a_clipped_corner_blocks() {
        let from = HexCoord::new(0, 0);
        for to in [
            HexCoord::new(1, 7),
            HexCoord::new(1, 9),
            HexCoord::new(1, 10),
        ] {
            let [ray, _] = los_ray_sides(from, to);
            let sampled = from.line_between(to);
            let other = from.line_between_other_side(to);
            let clipped: Vec<HexCoord> = ray[1..ray.len() - 1]
                .iter()
                .copied()
                .filter(|h| !sampled.contains(h) && !other.contains(h))
                .collect();
            assert!(!clipped.is_empty(), "{to}");
            for h in clipped {
                let board = board_with_terrain(&[(h.q, h.r, Terrain::ground(GroundKind::Rough))]);
                assert!(
                    !has_los_auto(&board, from, to, FireKind::Direct, no_units()),
                    "{to}: rough at {h}"
                );
            }
        }
    }

    /// A ray along hexsides passes either side of them: a unit on one side
    /// leaves the other side clear; units on both sides block. A ray
    /// exactly through a hex corner is the same tie: the hexes only one of
    /// its two sides passes do not block alone, one on each side does.
    #[rulebook("§6.3")]
    #[test]
    fn a_tied_ray_is_blocked_only_when_both_sides_are() {
        let open = BoardInfo::default();
        let g = LosLevel::Ground;
        let sees = |from, to, units: &[HexCoord]| {
            has_los(
                &open,
                from,
                to,
                FireKind::Direct,
                g,
                g,
                units_at(units),
                |_, _| false,
            )
        };
        // Along hexsides: (0,0) -> (2,1) runs along (1,1)|(1,0).
        let (from, to) = (HexCoord::new(0, 0), HexCoord::new(2, 1));
        let (left, right) = (HexCoord::new(1, 1), HexCoord::new(1, 0));
        assert!(sees(from, to, &[left]));
        assert!(sees(from, to, &[right]));
        assert!(!sees(from, to, &[left, right]));
        // Through a corner: (0,0) -> (1,5), at range 5, runs along no
        // hexside but passes exactly through hex corners.
        let to = HexCoord::new(1, 5);
        assert!(!from.line_runs_along_hexsides(to));
        let [one, other] = los_ray_sides(from, to);
        let only = |a: &[HexCoord], b: &[HexCoord]| -> Vec<HexCoord> {
            a.iter().copied().filter(|h| !b.contains(h)).collect()
        };
        let (one_only, other_only) = (only(&one, &other), only(&other, &one));
        assert!(!one_only.is_empty() && !other_only.is_empty());
        for &h in one_only.iter().chain(&other_only) {
            assert!(sees(from, to, &[h]), "{h} alone");
        }
        assert!(!sees(from, to, &[one_only[0], other_only[0]]));
        // A hex both sides pass blocks alone.
        let common = one[1..one.len() - 1]
            .iter()
            .copied()
            .find(|h| other.contains(h))
            .unwrap();
        assert!(!sees(from, to, &[common]));
    }

    /// Line of sight is reciprocal on any board in the Ground→Ground box
    /// (whose entries carry no positional footnote): pseudo-random terrain,
    /// walls, gates and units, every pair within range 8 -- along hexsides,
    /// through corners, clipping them -- sees both ways or neither.
    #[rulebook("§6.3")]
    #[test]
    fn ground_to_ground_los_is_reciprocal_on_a_random_board() {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut board = BoardInfo::default();
        let mut units = Vec::new();
        let kinds = [
            GroundKind::Clear,
            GroundKind::Clear,
            GroundKind::Clear,
            GroundKind::Huts,
            GroundKind::Trees,
            GroundKind::Building,
            GroundKind::Rough,
        ];
        for q in -2..=12 {
            for r in -2..=12 {
                let h = HexCoord::new(q, r);
                let roll = next();
                board
                    .terrain
                    .insert(h, Terrain::ground(kinds[(roll % 7) as usize]));
                if roll % 23 == 0 {
                    units.push(h);
                }
                for n in [HexCoord::new(q + 1, r), HexCoord::new(q, r + 1)] {
                    match next() % 17 {
                        0 => board
                            .hexsides
                            .insert(HexsideRef::new(h, n), HexsideKind::Wall),
                        1 => board
                            .hexsides
                            .insert(HexsideRef::new(h, n), HexsideKind::Gate),
                        _ => None,
                    };
                }
            }
        }
        let g = LosLevel::Ground;
        let sees = |a, b| {
            has_los(
                &board,
                a,
                b,
                FireKind::Direct,
                g,
                g,
                units_at(&units),
                |_, _| false,
            )
        };
        let hexes: Vec<HexCoord> = (0..=10)
            .flat_map(|q| (0..=10).map(move |r| HexCoord::new(q, r)))
            .collect();
        let (mut clear, mut blocked) = (0, 0);
        for &a in &hexes {
            for &b in &hexes {
                if a.distance(b) > 8 || a >= b {
                    continue;
                }
                let ab = sees(a, b);
                assert_eq!(ab, sees(b, a), "{a} <-> {b}");
                if ab { clear += 1 } else { blocked += 1 }
            }
        }
        assert!(
            clear > 100 && blocked > 100,
            "{clear} clear, {blocked} blocked"
        );
    }

    /// Not a check: how many ordered pairs at range 2-10 on each board the
    /// exact ray judges differently from the one-sample-per-hex line it
    /// replaced (levels from the terrain, no units). Run with
    /// `cargo test -p omdurman-rules --lib exact_ray_scan -- --ignored --nocapture`.
    #[test]
    #[ignore = "a report, not a check"]
    fn exact_ray_scan() {
        let boards = [
            ("Campaign", crate::board_data::campaign_map_data()),
            (
                "Fall of Khartoum",
                crate::board_data::fall_of_khartoum_map_data(),
            ),
        ];
        for (name, data) in boards {
            let board = BoardInfo::from_map_data(data);
            let hexes: Vec<HexCoord> = board.terrain.keys().copied().collect();
            let level = |h| los_level(board.terrain_at(h).unwrap_or_default());
            let sampled = |from: HexCoord, to: HexCoord| {
                let with_ends = |line: Vec<HexCoord>| {
                    let mut path = vec![from];
                    path.extend(line);
                    path.push(to);
                    path
                };
                let along = from.hexsides_along_line(to);
                let walk = |path| {
                    los_walk(
                        &board,
                        from,
                        to,
                        path,
                        &along,
                        level(from),
                        level(to),
                        no_units(),
                        |_, _| false,
                    )
                    .1
                };
                walk(with_ends(from.line_between(to))).is_none()
                    || walk(with_ends(from.line_between_other_side(to))).is_none()
            };
            let (mut now_blocked, mut by_wall, mut now_clear) = (0, 0, 0);
            for &a in &hexes {
                for &b in &hexes {
                    if !(2..=10).contains(&a.distance(b)) {
                        continue;
                    }
                    let (old, new) = (
                        sampled(a, b),
                        los_rays(&board, a, b, level(a), level(b), no_units(), |_, _| false),
                    );
                    match (old, new.1) {
                        (true, Some(block)) => {
                            now_blocked += 1;
                            if matches!(
                                block,
                                LosBlock::Hexside {
                                    feature: LosFeature::Wall,
                                    ..
                                }
                            ) {
                                by_wall += 1;
                            }
                        }
                        (false, None) => now_clear += 1,
                        _ => {}
                    }
                }
            }
            println!(
                "{name}: {now_blocked} pairs clear before are blocked now ({by_wall} by a wall), \
                 {now_clear} blocked before are clear now"
            );
        }
    }

    // ── Property tests: reflexivity + symmetry ───────────────────────────

    #[rulebook("§6.3")]
    #[test]
    fn los_reflexive_all_terrains() {
        for kind in GroundKind::iter() {
            let terrain = Terrain::ground(kind);
            let board = board_with_terrain(&[(0, 0, terrain)]);
            let level = los_level(terrain);
            assert!(
                has_los(
                    &board,
                    HexCoord::new(0, 0),
                    HexCoord::new(0, 0),
                    FireKind::Direct,
                    level,
                    level,
                    no_units(),
                    |_, _| false,
                ),
                "LOS not reflexive for {kind:?}"
            );
        }
    }

    #[rulebook("§6.3")]
    #[test]
    fn los_reflexive_hilltop() {
        let board = board_with_terrain(&[(0, 0, Terrain::ground(GroundKind::Hilltop))]);
        assert!(has_los(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(0, 0),
            FireKind::Direct,
            LosLevel::Hilltop,
            LosLevel::Hilltop,
            no_units(),
            |_, _| false,
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn los_symmetric_ground_to_ground_no_units() {
        // Build a line of hexes with varying terrain and check symmetry.
        let board = board_with_terrain(&[
            (0, 0, Terrain::ground(GroundKind::Clear)),
            (1, 0, Terrain::ground(GroundKind::Huts)),
            (2, 0, Terrain::ground(GroundKind::Clear)),
            (3, 0, Terrain::ground(GroundKind::Trees)),
            (4, 0, Terrain::ground(GroundKind::Clear)),
            (5, 0, Terrain::ground(GroundKind::Rough)),
            (6, 0, Terrain::ground(GroundKind::Clear)),
        ]);
        let coords: Vec<HexCoord> = (0..=6).map(|q| HexCoord::new(q, 0)).collect();
        for &a in &coords {
            for &b in &coords {
                let ab = has_los_auto(&board, a, b, FireKind::Direct, no_units());
                let ba = has_los_auto(&board, b, a, FireKind::Direct, no_units());
                assert_eq!(
                    ab, ba,
                    "LOS not symmetric: los({a:?},{b:?})={ab} but los({b:?},{a:?})={ba}"
                );
            }
        }
    }

    // §6.3: the ray is the straight line between the hex centres. Across
    // the open plain south of the Zariba it crosses Jebel Surgham's rough
    // slope at (35,19); the old greedy walk ran six hexes down the r axis
    // first, round the slope, and saw the Historical Dervish set-up ground.
    #[rulebook("§6.3")]
    #[test]
    fn los_ray_is_the_straight_line_on_the_campaign_board() {
        let board = crate::board::BoardInfo::from_map_data(crate::board_data::campaign_map_data());
        assert!(matches!(
            board.terrain_at(HexCoord::new(35, 19)),
            Some(Terrain::Rough)
        ));
        let (zariba, plain) = (HexCoord::new(36, 17), HexCoord::new(33, 23));
        let sees = |a, b| {
            has_los(
                &board,
                a,
                b,
                crate::FireKind::Direct,
                LosLevel::Ground,
                LosLevel::Ground,
                |_| None,
                |_, _| false,
            )
        };
        assert!(!sees(zariba, plain));
        assert!(!sees(plain, zariba), "line of sight is reciprocal");
    }

    #[rulebook("§6.3")]
    #[test]
    fn los_howitzer_always_has_los() {
        // Howitzer fire bypasses LOS (§6.64): even with intervening blockers.
        let board = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Rough)),
            (2, 0, Terrain::ground(GroundKind::Hilltop)),
        ]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(3, 0),
            FireKind::Howitzer,
            no_units(),
        ));
    }

    #[rulebook("§6.3")]
    #[test]
    fn los_howitzer_same_hex() {
        let board = board_with_terrain(&[(0, 0, Terrain::ground(GroundKind::Hilltop))]);
        assert!(has_los_auto(
            &board,
            HexCoord::new(0, 0),
            HexCoord::new(0, 0),
            FireKind::Howitzer,
            no_units(),
        ));
    }

    // ── Exhaustive LOS table structural test ─────────────────────────────

    #[rulebook("§6.3")]
    #[test]
    fn los_blocking_rules_match_reference_table() {
        // Exhaustively verify every cell of the LOS blocking rules table
        // against the authoritative reference (los_table.ron).
        // Each (firer, target) pair lists which features should appear and
        // which conditions they carry.

        use LosCondition::*;
        use LosFeature::*;

        // (firer_level, target_level, expected_features_with_conditions)
        #[allow(clippy::type_complexity)]
        let cases: Vec<(LosLevel, LosLevel, Vec<(LosFeature, Vec<LosCondition>)>)> = vec![
            // Ground → Ground: Units, Huts(1), Wall, Rough, Trees(1)
            (
                LosLevel::Ground,
                LosLevel::Ground,
                vec![
                    (Units, vec![]),
                    (Huts, vec![MoreThanTwo]),
                    (Wall, vec![]),
                    (RoughTerrain, vec![]),
                    (Trees, vec![MoreThanTwo]),
                ],
            ),
            // Ground → Rough: Units(3,6), Huts(1,3), Wall, Crest(2), Trees(1), Hilltop
            (
                LosLevel::Ground,
                LosLevel::Rough,
                vec![
                    (Units, vec![CloserToFirer, AdjSameLevelTarget]),
                    (Huts, vec![MoreThanTwo, CloserToFirer]),
                    (Wall, vec![]),
                    (Crest, vec![CrestAdjacency]),
                    (Trees, vec![MoreThanTwo]),
                    (HilltopTerrain, vec![]),
                ],
            ),
            // Ground → Hilltop: Units(3), Huts(1,3), Crest(3), Hilltop
            (
                LosLevel::Ground,
                LosLevel::Hilltop,
                vec![
                    (Units, vec![CloserToFirer]),
                    (Huts, vec![MoreThanTwo, CloserToFirer]),
                    (Crest, vec![CloserToFirer]),
                    (HilltopTerrain, vec![]),
                ],
            ),
            // Rough → Ground: Units(4,5), Huts(1,4), Wall, Crest(2), Trees(1), Hilltop
            (
                LosLevel::Rough,
                LosLevel::Ground,
                vec![
                    (Units, vec![CloserToTarget, AdjSameLevelFirer]),
                    (Huts, vec![MoreThanTwo, CloserToTarget]),
                    (Wall, vec![]),
                    (Crest, vec![CrestAdjacency]),
                    (Trees, vec![MoreThanTwo]),
                    (HilltopTerrain, vec![]),
                ],
            ),
            // Rough → Rough: Units(7), Hilltop, Crest(2)
            (
                LosLevel::Rough,
                LosLevel::Rough,
                vec![
                    (Units, vec![NotAtLowerLevel]),
                    (HilltopTerrain, vec![]),
                    (Crest, vec![CrestAdjacency]),
                ],
            ),
            // Rough → Hilltop: Units(3), Crest(2,3), Hilltop
            (
                LosLevel::Rough,
                LosLevel::Hilltop,
                vec![
                    (Units, vec![CloserToFirer]),
                    (Crest, vec![CrestAdjacency, CloserToFirer]),
                    (HilltopTerrain, vec![]),
                ],
            ),
            // Hilltop → Ground: Units(4), Huts(1,4), Crest(4), Hilltop
            (
                LosLevel::Hilltop,
                LosLevel::Ground,
                vec![
                    (Units, vec![CloserToTarget]),
                    (Huts, vec![MoreThanTwo, CloserToTarget]),
                    (Crest, vec![CloserToTarget]),
                    (HilltopTerrain, vec![]),
                ],
            ),
            // Hilltop → Rough: Units(4), Hilltop, Crest(2,4)
            (
                LosLevel::Hilltop,
                LosLevel::Rough,
                vec![
                    (Units, vec![CloserToTarget]),
                    (HilltopTerrain, vec![]),
                    (Crest, vec![CrestAdjacency, CloserToTarget]),
                ],
            ),
            // Hilltop → Hilltop: Units, only at hilltop level (HilltopOnly)
            (
                LosLevel::Hilltop,
                LosLevel::Hilltop,
                vec![(Units, vec![HilltopOnly])],
            ),
        ];

        for (firer, target, expected) in &cases {
            let rules = blocking_rules(*firer, *target);
            assert_eq!(
                rules.len(),
                expected.len(),
                "wrong number of blocking rules for {firer:?}→{target:?}: got {} expected {}",
                rules.len(),
                expected.len(),
            );
            for (i, got) in rules.iter().enumerate() {
                let (want_feature, want_conds) = &expected[i];
                assert_eq!(
                    got.0, *want_feature,
                    "feature mismatch at {firer:?}→{target:?} index {i}: got {:?} want {want_feature:?}",
                    got.0
                );
                assert_eq!(
                    got.1, *want_conds,
                    "conditions mismatch at {firer:?}→{target:?} feature {:?}: got {:?} want {want_conds:?}",
                    got.0, got.1
                );
            }
        }
    }

    // ── Exhaustive LOS table behavioral test ─────────────────────────────

    #[rulebook("§6.3")]
    #[test]
    fn los_ground_to_ground_features_block_as_expected() {
        // Ground → Ground: test each feature in isolation on a straight line.
        let base = board_with_terrain(&[]);

        // Units block (always, no conditions)
        let _board = board_with_terrain(&[]);
        let unit_blocking = |hex: HexCoord| -> Option<LosLevel> {
            if hex == HexCoord::new(1, 0) {
                Some(LosLevel::Ground)
            } else {
                None
            }
        };
        assert!(!has_los(
            &base,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            LosLevel::Ground,
            LosLevel::Ground,
            unit_blocking,
            |_, _| false,
        ));

        // Huts block only when > 2
        let board2 = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Huts))]);
        assert!(has_los_auto(
            &board2,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units(),
        ));
        let board3 = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Huts)),
            (2, 0, Terrain::ground(GroundKind::Huts)),
            (3, 0, Terrain::ground(GroundKind::Huts)),
        ]);
        assert!(!has_los_auto(
            &board3,
            HexCoord::new(0, 0),
            HexCoord::new(4, 0),
            FireKind::Direct,
            no_units(),
        ));

        // Rough always blocks
        let board4 = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Rough))]);
        assert!(!has_los_auto(
            &board4,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units(),
        ));

        // Trees block only when > 2
        let board5 = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Trees))]);
        assert!(has_los_auto(
            &board5,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units(),
        ));
        let board6 = board_with_terrain(&[
            (1, 0, Terrain::ground(GroundKind::Trees)),
            (2, 0, Terrain::ground(GroundKind::Trees)),
            (3, 0, Terrain::ground(GroundKind::Trees)),
        ]);
        assert!(!has_los_auto(
            &board6,
            HexCoord::new(0, 0),
            HexCoord::new(4, 0),
            FireKind::Direct,
            no_units(),
        ));
    }
}

/// Kani proof harnesses over the pure LOS-table functions (`cargo kani`,
/// see `scripts/kani.sh`). The full `has_los` ray-walk over a symbolic
/// board is out of Kani's reach, but the pieces every ray is built from --
/// the terrain-to-level mapping, the per-unit level overrides, and the
/// blocking-rule grid -- are small closed domains and are proven exactly.
#[cfg(kani)]
mod verification {
    use super::{LosFeature, LosLevel, los_level, los_level_for_unit};
    use crate::tables_data::LOS_CELLS;
    use omdurman_types::{GroundKind, HexCoord, Terrain, UnitKind};

    /// A symbolic ground kind.
    fn any_ground(i: usize) -> GroundKind {
        match i {
            0 => GroundKind::Clear,
            1 => GroundKind::Rough,
            2 => GroundKind::Trees,
            3 => GroundKind::Swamp,
            4 => GroundKind::Hilltop,
            5 => GroundKind::Huts,
            _ => GroundKind::Building,
        }
    }

    /// An arbitrary level pair into the authored 3×3 grid.
    fn any_levels() -> (LosLevel, LosLevel) {
        let f: usize = kani::any();
        let t: usize = kani::any();
        let level = |i: usize| match i % 3 {
            0 => LosLevel::Ground,
            1 => LosLevel::Rough,
            _ => LosLevel::Hilltop,
        };
        (level(f), level(t))
    }

    /// The terrain-to-level mapping is a property of the *ground* alone:
    /// the Nile flow direction never changes a hex's LOS level. (Roads are
    /// a property of the hexside links, so they cannot leak into the level
    /// lookup by construction.)
    // §6.3
    #[traceability_macro::rulebook("§6.3")]
    #[kani::proof]
    #[kani::unwind(14)]
    fn los_level_depends_only_on_the_ground() {
        let g: usize = kani::any();
        let kind = any_ground(g % 7);
        let plain = los_level(Terrain::ground(kind));
        // The Nile's level is flow-independent.
        let d0: u8 = kani::any();
        let nile = Terrain::Nile {
            direction: omdurman_types::HexDirection::from_index(d0 % 6),
        };
        assert!(
            los_level(nile)
                == los_level(Terrain::Nile {
                    direction: omdurman_types::HexDirection::East
                })
        );
    }

    /// Special LOS notes (b) and (c) override the underlying terrain
    /// without consulting the board: gunboats fight from rough level and
    /// forts sit at ground level, for every hex of a rule-neutral board.
    /// These two early returns are what keep a gunboat target hittable
    /// over a wall from a lower-flying firer, so they must not degrade
    /// into terrain lookups.
    // §6.3
    #[traceability_macro::rulebook("§6.3")]
    #[kani::proof]
    #[kani::unwind(14)]
    fn los_level_overrides_hold_for_gunboats_and_forts() {
        let q: i32 = kani::any();
        let r: i32 = kani::any();
        kani::assume(q >= -2 && q <= 2);
        kani::assume(r >= -2 && r <= 2);
        let board = crate::board::BoardInfo::default();
        let hex = HexCoord::new(q, r);
        let gunboat = UnitKind::Gunboat {
            fire: 3,
            upstream: 10,
            downstream: 16,
        };
        assert!(los_level_for_unit(gunboat, hex, &board) == LosLevel::Rough);
        let fort = UnitKind::Fort { fire: 3, melee: 4 };
        assert!(los_level_for_unit(fort, hex, &board) == LosLevel::Ground);
    }

    /// The blocking-rule grid is total over every level pair (no cell is
    /// empty), and the printed wall rule is pinned cell-exact: an intact
    /// wall blocks *unconditionally* (no positional conditions) exactly for
    /// Ground→Ground, Ground→Rough and Rough→Ground. Every other cell has
    /// no Wall entry at all -- hilltop firers shoot over walls, and a
    /// rough-level firer sees along the wall to a rough-level target. The
    /// cell is selected by `match` (a case split into nine concrete slices)
    /// rather than by indexing with symbolic levels: symbolic indexing into
    /// the nested `&[&[BlockingRule]]` static makes the pointer reads
    /// intractable for the solver (measured: hours vs seconds).
    // §6.3
    #[traceability_macro::rulebook("§6.3")]
    #[kani::proof]
    #[kani::unwind(14)]
    fn blocking_grid_is_total_and_walls_block_ground_firers() {
        let (firer, target) = any_levels();
        use super::LosLevel as L;
        let cell = match (firer, target) {
            (L::Ground, L::Ground) => LOS_CELLS[0][0],
            (L::Ground, L::Rough) => LOS_CELLS[0][1],
            (L::Ground, L::Hilltop) => LOS_CELLS[0][2],
            (L::Rough, L::Ground) => LOS_CELLS[1][0],
            (L::Rough, L::Rough) => LOS_CELLS[1][1],
            (L::Rough, L::Hilltop) => LOS_CELLS[1][2],
            (L::Hilltop, L::Ground) => LOS_CELLS[2][0],
            (L::Hilltop, L::Rough) => LOS_CELLS[2][1],
            (L::Hilltop, L::Hilltop) => LOS_CELLS[2][2],
        };
        // Total: every cell of the authored grid is non-empty.
        assert!(!cell.is_empty());
        // The authored wall rule, cell-exact (see the harness doc).
        let wall_always = cell
            .iter()
            .any(|rule| rule.0 == LosFeature::Wall && rule.1.is_empty());
        let wall_expected = matches!(
            (firer, target),
            (L::Ground, L::Ground) | (L::Ground, L::Rough) | (L::Rough, L::Ground)
        );
        assert!(wall_always == wall_expected);
    }
}
