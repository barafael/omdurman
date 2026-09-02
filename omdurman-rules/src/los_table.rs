//! Line of Sight Table (rulebook §6.3, back cover).
//!
//! The LOS system uses a 3×3 matrix indexed by the firer's and target's
//! terrain level (`Ground`, `Rough`, `Hilltop`). Each cell lists which
//! intervening features (terrain, hexsides, units) block LOS, subject to
//! positional conditions (Details footnotes 1–7) and special notes (a–f).
//!
//! The authoritative source is `Boardgame - Remember_Gordon/tables/los_table.ron`.
//!
//! ## How it works
//!
//! 1. Determine the firer's LOS level from the terrain at the firing hex.
//! 2. Determine the target's LOS level from the terrain at the target hex.
//! 3. Look up the blocking rules for that `(firer, target)` pair.
//! 4. Walk the LOS ray hex by hex. For each intervening hex and hexside,
//!    check whether it matches a blocking feature and whether all positional
//!    conditions are satisfied.
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

use omdurman_types::{HexCoord, HexsideKind, Terrain, UnitKind};

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
        Terrain::Clear { .. } => N::Clear,
        Terrain::Rough { .. } => N::Rough,
        Terrain::Trees { .. } => N::Trees,
        Terrain::Swamp { .. } => N::Swamp,
        Terrain::Nile { .. } => N::Nile,
        Terrain::Hilltop { .. } => N::Hilltop,
        Terrain::Huts { .. } => N::Huts,
        Terrain::Building { .. } => N::Building,
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
    // Note (b): units inside a walled city (Building terrain) adjacent to a
    // wall hexside are at rough level.
    if matches!(terrain, Terrain::Building { .. }) {
        let adj_to_wall = hex.neighbors().iter().any(|n| {
            board
                .hexside_between(hex, *n)
                .is_some_and(|s| s == HexsideKind::Wall)
        });
        if adj_to_wall {
            return LosLevel::Rough;
        }
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
    /// Index of this hex along the ray (0 = firer).
    index: usize,
    /// Total number of steps in the ray.
    total_steps: usize,
    /// Cumulative count of hut/tree hexes seen so far (including this one).
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
/// Returns `true` if ALL conditions are satisfied (feature blocks).
fn conditions_met(conditions: &[LosCondition], ctx: &CondCtx) -> bool {
    for &cond in conditions {
        let ok = match cond {
            LosCondition::MoreThanTwo => ctx.hut_tree_count > 2,
            LosCondition::CrestAdjacency => !ctx.crest_adjacency_exception,
            LosCondition::CloserToFirer => ctx.index <= ctx.total_steps / 2,
            LosCondition::CloserToTarget => {
                let dist_from_target = ctx.total_steps - ctx.index;
                dist_from_target <= ctx.total_steps / 2
            }
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
pub fn has_los(
    board: &crate::board::BoardInfo,
    from: HexCoord,
    to: HexCoord,
    kind: crate::FireKind,
    firer_level: LosLevel,
    target_level: LosLevel,
    unit_level_at: impl Fn(HexCoord) -> Option<LosLevel>,
) -> bool {
    use crate::FireKind;

    if kind == FireKind::Howitzer {
        return true;
    }

    let rules = blocking_rules(firer_level, target_level);

    // Adjacency check: is `hex` adjacent to `ref_hex`?
    let adjacent =
        |hex: HexCoord, ref_hex: HexCoord| -> bool { ref_hex.neighbors().contains(&hex) };

    // Build the ray path: [from, intervening..., to].
    let mut path = vec![from];
    path.extend(from.line_between(to));
    path.push(to);

    let total_steps = path.len().saturating_sub(1);
    if total_steps == 0 {
        return true; // same hex
    }

    // Determine which crest entry (if any) the blocking rules use, so we
    // know whether parallel-crest scanning (note e) is needed.
    let crest_conditions: Option<&[LosCondition]> =
        rules.iter().find(|r| r.0 == LosFeature::Crest).map(|r| r.1);

    // Pre-scan: collect ALL crest hexsides on or along the ray (note e).
    // Crossed crests: between consecutive ray hexes.
    // Parallel crests: on intervening hexes' non-crossed hexsides.
    let mut all_crest_hexsides: Vec<(HexCoord, HexCoord)> = Vec::new();

    // Crossed crests.
    for w in path.windows(2) {
        if board
            .hexside_between(w[0], w[1])
            .is_some_and(|s| s == HexsideKind::Crest)
        {
            all_crest_hexsides.push((w[0], w[1]));
        }
    }

    // Parallel crests (note e) — only relevant if Crest is a blocking feature.
    if crest_conditions.is_some() {
        for (i, &hex) in path.iter().enumerate() {
            if hex == from || hex == to {
                continue;
            }
            let prev = if i > 0 { Some(path[i - 1]) } else { None };
            let next = path.get(i + 1).copied();
            for neighbor in hex.neighbors() {
                // Skip the entry and exit hexsides — those are crossed crests.
                if prev == Some(neighbor) || next == Some(neighbor) {
                    continue;
                }
                if board
                    .hexside_between(hex, neighbor)
                    .is_some_and(|s| s == HexsideKind::Crest)
                {
                    all_crest_hexsides.push((hex, neighbor));
                }
            }
        }
    }

    // Condition 2: "Not blocked if firing units and/or target units are
    // adjacent to all crest hexsides fired through."
    //
    // "Adjacent to a crest hexside" means the unit is IN one of the two
    // hexes that share the crest hexside (not merely a neighbor of one).
    let crest_adjacency_exception = if all_crest_hexsides.is_empty() {
        false
    } else {
        let firer_on_all = all_crest_hexsides
            .iter()
            .all(|&(a, b)| from == a || from == b);
        let target_on_all = all_crest_hexsides.iter().all(|&(a, b)| to == a || to == b);
        firer_on_all || target_on_all
    };

    // Track cumulative hut/tree count (condition 1). Building counts as
    // Huts (rulebook §5.44 groups "hut or building" together).
    let mut hut_tree_count = 0usize;

    // Walk the ray, checking each intervening hex (skip endpoints).
    for (i, &hex) in path.iter().enumerate() {
        // Skip firer and target hexes — they are not "intervening".
        if hex == from || hex == to {
            continue;
        }

        let terrain = board.terrain_at(hex).unwrap_or_default();
        let hex_level = los_level(terrain);
        let unit_level = unit_level_at(hex);

        // Update cumulative hut/tree count (Building counts as Huts).
        let is_hut_or_tree = matches!(
            terrain,
            Terrain::Huts { .. } | Terrain::Building { .. } | Terrain::Trees { .. }
        );
        if is_hut_or_tree {
            hut_tree_count += 1;
        }

        let ctx = CondCtx {
            index: i,
            total_steps,
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
                    matches!(terrain, Terrain::Huts { .. } | Terrain::Building { .. })
                }
                LosFeature::Trees => matches!(terrain, Terrain::Trees { .. }),
                LosFeature::RoughTerrain => {
                    matches!(terrain, Terrain::Rough { .. })
                }
                LosFeature::HilltopTerrain => {
                    matches!(terrain, Terrain::Hilltop { .. })
                }
                // Wall and Crest are hexside features, checked separately.
                LosFeature::Wall | LosFeature::Crest => false,
            };

            // The Hilltop→Hilltop "only hilltop-level units block" special
            // case is authored as the HilltopOnly condition in the table.
            if feature_matches && conditions_met(conditions, &ctx) {
                return false; // blocked
            }
        }

        // Fix 2 (note e): check for parallel crest hexsides on this hex.
        if let Some(crest_conds) = crest_conditions {
            let prev = if i > 0 { Some(path[i - 1]) } else { None };
            let next = path.get(i + 1).copied();
            for neighbor in hex.neighbors() {
                if prev == Some(neighbor) || next == Some(neighbor) {
                    continue; // entry/exit hexside — crossed, not parallel
                }
                if !board
                    .hexside_between(hex, neighbor)
                    .is_some_and(|s| s == HexsideKind::Crest)
                {
                    continue;
                }
                // Parallel crest found — apply the same conditions.
                if conditions_met(crest_conds, &ctx) {
                    return false; // blocked
                }
                break; // one parallel crest is enough to block
            }
        }
    }

    // Check crossed hexsides (Wall, Crest) between consecutive ray hexes.
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        let Some(hs) = board.hexside_between(a, b) else {
            continue;
        };

        let feature = match hs {
            HexsideKind::Wall => LosFeature::Wall,
            HexsideKind::Crest => LosFeature::Crest,
            _ => continue,
        };

        let entry = rules.iter().find(|r| r.0 == feature);
        let Some(conditions) = entry.map(|r| r.1) else {
            continue;
        };

        let hexside_index = path.iter().position(|&h| h == b).unwrap_or(0);
        let terrain_at_b = board.terrain_at(b).unwrap_or_default();
        let ctx = CondCtx {
            index: hexside_index,
            total_steps,
            hut_tree_count,
            hex_level: los_level(terrain_at_b),
            firer_level,
            target_level,
            adjacent_to_firer: adjacent(b, from),
            adjacent_to_target: adjacent(b, to),
            crest_adjacency_exception,
            unit_level: unit_level_at(b),
        };

        if conditions_met(conditions, &ctx) {
            return false; // blocked
        }
    }

    true
}

// ─── los_path_analysis ──────────────────────────────────────────────────

/// Annotate every step of the LOS ray from `from` to `to` (§6.21, §6.3).
///
/// Returns a list of `(hex, step_result)` pairs. The first entry is always
/// `(from, Clear)`. If a step blocks, subsequent steps are not included.
///
/// Howitzer fire bypasses LOS (§6.64); every step is `Clear`.
///
/// Like [`has_los`], this takes pre-computed `firer_level` and `target_level`
/// (use [`los_level_for_unit`] at the call site).
pub fn los_path_analysis(
    board: &crate::board::BoardInfo,
    from: HexCoord,
    to: HexCoord,
    kind: crate::FireKind,
    firer_level: LosLevel,
    target_level: LosLevel,
    unit_level_at: impl Fn(HexCoord) -> Option<LosLevel>,
) -> Vec<(HexCoord, LosStepResult)> {
    use crate::FireKind;

    let mut result = Vec::new();

    if kind == FireKind::Howitzer {
        result.push((from, LosStepResult::Clear));
        let mut path = vec![from];
        path.extend(from.line_between(to));
        path.push(to);
        for h in path.into_iter().skip(1) {
            result.push((h, LosStepResult::Clear));
        }
        return result;
    }

    let rules = blocking_rules(firer_level, target_level);
    let adjacent =
        |hex: HexCoord, ref_hex: HexCoord| -> bool { ref_hex.neighbors().contains(&hex) };

    let mut path = vec![from];
    path.extend(from.line_between(to));
    path.push(to);
    let total_steps = path.len().saturating_sub(1);

    result.push((from, LosStepResult::Clear));

    if total_steps == 0 {
        return result;
    }

    // Pre-scan crest hexsides (crossed + parallel per note e).
    let crest_conditions: Option<&[LosCondition]> =
        rules.iter().find(|r| r.0 == LosFeature::Crest).map(|r| r.1);

    let mut all_crest_hexsides: Vec<(HexCoord, HexCoord)> = Vec::new();
    for w in path.windows(2) {
        if board
            .hexside_between(w[0], w[1])
            .is_some_and(|s| s == HexsideKind::Crest)
        {
            all_crest_hexsides.push((w[0], w[1]));
        }
    }
    if crest_conditions.is_some() {
        for (i, &hex) in path.iter().enumerate() {
            if hex == from || hex == to {
                continue;
            }
            let prev = if i > 0 { Some(path[i - 1]) } else { None };
            let next = path.get(i + 1).copied();
            for neighbor in hex.neighbors() {
                if prev == Some(neighbor) || next == Some(neighbor) {
                    continue;
                }
                if board
                    .hexside_between(hex, neighbor)
                    .is_some_and(|s| s == HexsideKind::Crest)
                {
                    all_crest_hexsides.push((hex, neighbor));
                }
            }
        }
    }

    let crest_adjacency_exception = if all_crest_hexsides.is_empty() {
        false
    } else {
        let firer_on_all = all_crest_hexsides
            .iter()
            .all(|&(a, b)| from == a || from == b);
        let target_on_all = all_crest_hexsides.iter().all(|&(a, b)| to == a || to == b);
        firer_on_all || target_on_all
    };

    let mut hut_tree_count = 0usize;

    for (i, &hex) in path.iter().enumerate() {
        if hex == from {
            continue;
        }
        if hex == to {
            result.push((hex, LosStepResult::Clear));
            break;
        }

        let terrain = board.terrain_at(hex).unwrap_or_default();
        let hex_level = los_level(terrain);
        let unit_level = unit_level_at(hex);

        let is_hut_or_tree = matches!(
            terrain,
            Terrain::Huts { .. } | Terrain::Building { .. } | Terrain::Trees { .. }
        );
        if is_hut_or_tree {
            hut_tree_count += 1;
        }

        let ctx = CondCtx {
            index: i,
            total_steps,
            hut_tree_count,
            hex_level,
            firer_level,
            target_level,
            adjacent_to_firer: adjacent(hex, from),
            adjacent_to_target: adjacent(hex, to),
            crest_adjacency_exception,
            unit_level,
        };

        let mut blocked = false;
        for rule in rules {
            let feature = rule.0;
            let conditions = rule.1;
            let feature_matches = match feature {
                LosFeature::Units => unit_level.is_some(),
                LosFeature::Huts => {
                    matches!(terrain, Terrain::Huts { .. } | Terrain::Building { .. })
                }
                LosFeature::Trees => matches!(terrain, Terrain::Trees { .. }),
                LosFeature::RoughTerrain => {
                    matches!(terrain, Terrain::Rough { .. })
                }
                LosFeature::HilltopTerrain => {
                    matches!(terrain, Terrain::Hilltop { .. })
                }
                LosFeature::Wall | LosFeature::Crest => false,
            };

            // Hilltop→Hilltop's "only hilltop-level units block" special case
            // is authored as the HilltopOnly condition in the table.
            if feature_matches && conditions_met(conditions, &ctx) {
                result.push((hex, LosStepResult::Blocked { feature, hex }));
                blocked = true;
                break;
            }
        }

        // Parallel crest check (note e).
        if !blocked && let Some(crest_conds) = crest_conditions {
            let prev = if i > 0 { Some(path[i - 1]) } else { None };
            let next = path.get(i + 1).copied();
            for neighbor in hex.neighbors() {
                if prev == Some(neighbor) || next == Some(neighbor) {
                    continue;
                }
                if board
                    .hexside_between(hex, neighbor)
                    .is_some_and(|s| s == HexsideKind::Crest)
                {
                    if conditions_met(crest_conds, &ctx) {
                        result.push((
                            hex,
                            LosStepResult::Blocked {
                                feature: LosFeature::Crest,
                                hex,
                            },
                        ));
                        blocked = true;
                    }
                    break;
                }
            }
        }

        if blocked {
            break;
        }
        result.push((hex, LosStepResult::Clear));
    }

    // Check crossed hexsides (if not already blocked).
    if result.last().map(|(_, r)| *r) != Some(LosStepResult::Clear)
        || result.len() < total_steps + 1
    {
        return result;
    }

    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        let Some(hs) = board.hexside_between(a, b) else {
            continue;
        };
        let feature = match hs {
            HexsideKind::Wall => LosFeature::Wall,
            HexsideKind::Crest => LosFeature::Crest,
            _ => continue,
        };
        let Some(conditions) = rules.iter().find(|r| r.0 == feature).map(|r| r.1) else {
            continue;
        };
        let hexside_index = path.iter().position(|&h| h == b).unwrap_or(0);
        let terrain_at_b = board.terrain_at(b).unwrap_or_default();
        let ctx = CondCtx {
            index: hexside_index,
            total_steps,
            hut_tree_count,
            hex_level: los_level(terrain_at_b),
            firer_level,
            target_level,
            adjacent_to_firer: adjacent(b, from),
            adjacent_to_target: adjacent(b, to),
            crest_adjacency_exception,
            unit_level: unit_level_at(b),
        };
        if conditions_met(conditions, &ctx) {
            result.push((b, LosStepResult::BlockedHexside { a, b, feature }));
            break;
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
        has_los(board, from, to, kind, fl, tl, units)
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
            no_units()
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
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let board = board_with_hexsides(
            &[(0, 0, Terrain::ground(GroundKind::Building))],
            &[(a, b, HexsideKind::Wall)],
        );
        assert_eq!(
            los_level_for_unit(
                UnitKind::Infantry {
                    fire: 0,
                    melee: 0,
                    movement: 0
                },
                a,
                &board
            ),
            LosLevel::Rough
        );
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
            no_units()
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
            no_units()
        ));
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
            // Hilltop → Ground: Units(3), Huts(1,4), Crest(4), Hilltop
            (
                LosLevel::Hilltop,
                LosLevel::Ground,
                vec![
                    (Units, vec![CloserToFirer]),
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
        ));

        // Huts block only when > 2
        let board2 = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Huts))]);
        assert!(has_los_auto(
            &board2,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units()
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
            no_units()
        ));

        // Rough always blocks
        let board4 = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Rough))]);
        assert!(!has_los_auto(
            &board4,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units()
        ));

        // Trees block only when > 2
        let board5 = board_with_terrain(&[(1, 0, Terrain::ground(GroundKind::Trees))]);
        assert!(has_los_auto(
            &board5,
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            FireKind::Direct,
            no_units()
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
            no_units()
        ));
    }
}
