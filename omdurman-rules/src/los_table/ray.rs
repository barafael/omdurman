//! The exact line of sight between two hex centres (rulebook §6.3).
//!
//! Note f has terrain "fill their entire hex for LOS purposes", so the ray
//! is the straight segment between the two hex centres in the board's real
//! geometry, and what it meets is exactly the hexes whose interior it passes
//! through -- including a hex whose corner it merely clips -- and the
//! hexsides it crosses on the way. Sampling the line once per hex of range
//! (cube rounding, [`HexCoord::line_between`]) misses both: at ranges 7, 9
//! and 10 it skips hexes the line clips, and where a sample lands exactly on
//! a hexside's midpoint it keeps only one of the two hexes the line passes
//! through there, so the hexside between consecutive sampled hexes is not
//! the one crossed.
//!
//! ## Geometry
//!
//! Hexes are pointy-topped, and `(q, r + 1)` lies half a hex to the *left*
//! of `(q, r)` one row down (see [`HexCoord::distance`]). Whether a line
//! meets a hex's interior, crosses a hexside or passes through a corner is
//! an affine invariant, so any affine image of the board answers it. This
//! module uses the integer lattice that puts the centre of `(q, r)` at
//! `(2q - r, 3r)`: the hex's corners then sit at the centre plus `(0, ±2)`
//! and `(±1, ±1)`, and its interior is the intersection of three open slabs,
//! `|x| < 1`, `|x + y| < 2` and `|y - x| < 2` about the centre, each slab's
//! two boundaries the hexsides toward two opposite neighbours. Everything is
//! integer arithmetic; there are no epsilons.
//!
//! ## The ties: a ray along a hexside, or through a corner
//!
//! A line in one of the six corner directions runs along hexsides (notes d
//! and e, [`HexCoord::hexsides_along_line`]); some other lines pass exactly
//! through a hex corner. There the line touches hexes it does not enter and
//! could be taken to pass on either side. The LOS table settles the hexsides
//! run along (notes d and e) but not which hexes such a line passes; the
//! engine reads it as passing on either side: shifted an infinitesimal
//! distance one way or the other, the line is generic -- it meets no corner
//! and runs along no hexside -- and [`los_ray_sides`] walks it each way. A
//! tied ray is clear if either side is (so a lone blocker beside a corner or
//! along a hexside does not block), which also keeps line of sight
//! reciprocal. A line that crosses a hexside at its midpoint, at an angle,
//! is no tie at all: it passes through both hexes and crosses the hexside
//! between them, on either side.
//!
//! The shift is symbolic: a crossing's parameter along the ray is a pair
//! `t0 + ε·t1` compared lexicographically, `ε` an infinitesimal.

use omdurman_types::HexCoord;

/// The centre of `h` on the integer lattice (see the module docs).
fn centre(h: HexCoord) -> [i64; 2] {
    let (q, r) = (i64::from(h.q), i64::from(h.r));
    [2 * q - r, 3 * r]
}

fn dot(a: [i64; 2], b: [i64; 2]) -> i64 {
    a[0] * b[0] + a[1] * b[1]
}

/// One of the three slabs whose intersection is a hex's interior: points `p`
/// with `|axis · (p - centre)| < bound`. Its `+bound` boundary is the
/// hexside toward the neighbour at offset `plus`, its `-bound` boundary the
/// one toward `minus` (`-plus`).
struct Slab {
    axis: [i64; 2],
    bound: i64,
    plus: (i32, i32),
}

const SLABS: [Slab; 3] = [
    // The vertical hexsides: east (q + 1, r) and west (q - 1, r).
    Slab {
        axis: [1, 0],
        bound: 1,
        plus: (1, 0),
    },
    // South-east (q + 1, r + 1) and north-west (q - 1, r - 1).
    Slab {
        axis: [1, 1],
        bound: 2,
        plus: (1, 1),
    },
    // South-west (q, r + 1) and north-east (q, r - 1).
    Slab {
        axis: [-1, 1],
        bound: 2,
        plus: (0, 1),
    },
];

/// A parameter along the shifted ray: `(t0 + ε·t1) / den`, `den > 0`.
#[derive(Clone, Copy, Debug)]
struct Param {
    t0: i64,
    t1: i64,
    den: i64,
}

impl Param {
    /// `self < other`, lexicographically in `(t0, t1)` -- exact, by cross
    /// multiplication with the (positive) denominators.
    fn lt(self, other: Param) -> bool {
        let lhs = (self.t0 * other.den, self.t1 * other.den);
        let rhs = (other.t0 * self.den, other.t1 * self.den);
        lhs < rhs
    }
}

/// The hexes the ray from `from` to `to` passes through, shifted
/// infinitesimally to either side, each as `[from, intervening..., to]` in
/// order (§6.3, note f): every hex whose interior the shifted segment
/// between the hex centres enters, consecutive hexes adjacent across the
/// hexside crossed. The two are equal unless the straight line touches a
/// hex corner (it runs along hexsides, or passes exactly through a corner);
/// then the first is the side [`HexCoord::line_between`] resolves ties to.
/// `[from]` twice for one hex.
pub fn los_ray_sides(from: HexCoord, to: HexCoord) -> [Vec<HexCoord>; 2] {
    if from == to {
        return [vec![from], vec![from]];
    }
    let [cx, cy] = centre(from);
    let [tx, ty] = centre(to);
    let d = [tx - cx, ty - cy];
    // The shift: perpendicular to the ray, so no slab parallel to the ray
    // keeps the shifted line on its boundary. Oriented like the tie-break
    // nudge of `HexLine`, the cube vector (1, 2, -3) -- (-1, 9) on this
    // lattice -- so that the first side is the one `line_between` walks.
    let perp = [-d[1], d[0]];
    let side = if 9 * d[0] + d[1] >= 0 { 1 } else { -1 };
    let shift = |s: i64| [perp[0] * s, perp[1] * s];
    [
        walk_side(from, to, d, shift(side)),
        walk_side(from, to, d, shift(-side)),
    ]
}

/// Walk the segment from `from`'s centre along `d`, shifted by `ε·w`, hex
/// by hex: leave each hex through the slab boundary the shifted line reaches
/// first, into the neighbour across it, until `to`.
fn walk_side(from: HexCoord, to: HexCoord, d: [i64; 2], w: [i64; 2]) -> Vec<HexCoord> {
    let origin = centre(from);
    // A segment of hex length n crosses at most 2n hexsides; the bound
    // only guards the loop.
    let limit = 2 * from.distance(to) as usize + 1;
    let mut path = Vec::with_capacity(limit);
    path.push(from);
    let mut cur = from;
    while cur != to && path.len() <= limit {
        let c = centre(cur);
        let c = [c[0] - origin[0], c[1] - origin[1]];
        let mut exit: Option<(Param, (i32, i32))> = None;
        for slab in &SLABS {
            let speed = dot(slab.axis, d);
            if speed == 0 {
                // Parallel to the ray: the shifted line never crosses it.
                continue;
            }
            // axis · (t·d + ε·w - c) = speed·t + ε·k - m reaches the
            // boundary ahead of it at ±bound.
            let k = dot(slab.axis, w);
            let m = dot(slab.axis, c);
            let (param, offset) = if speed > 0 {
                let param = Param {
                    t0: m + slab.bound,
                    t1: -k,
                    den: speed,
                };
                (param, slab.plus)
            } else {
                let param = Param {
                    t0: slab.bound - m,
                    t1: k,
                    den: -speed,
                };
                (param, (-slab.plus.0, -slab.plus.1))
            };
            if exit.is_none_or(|(best, _)| param.lt(best)) {
                exit = Some((param, offset));
            }
        }
        let Some((_, (dq, dr))) = exit else {
            break;
        };
        cur = HexCoord::new(cur.q + dq, cur.r + dr);
        path.push(cur);
    }
    debug_assert_eq!(path.last(), Some(&to), "the walk reaches the target");
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use traceability_macro::rulebook;

    /// Every hex within `n` of `origin`.
    fn disc(origin: HexCoord, n: i32) -> Vec<HexCoord> {
        let mut hexes = Vec::new();
        for dq in -n..=n {
            for dr in -n..=n {
                let h = HexCoord::new(origin.q + dq, origin.r + dr);
                if origin.distance(h) <= n as u32 {
                    hexes.push(h);
                }
            }
        }
        hexes
    }

    /// The corners of `h` on the lattice, in order around the hex.
    fn corners(h: HexCoord) -> [[i64; 2]; 6] {
        let [x, y] = centre(h);
        [(0, -2), (1, -1), (1, 1), (0, 2), (-1, 1), (-1, -1)].map(|(dx, dy)| [x + dx, y + dy])
    }

    fn cross(o: [i64; 2], a: [i64; 2], b: [i64; 2]) -> i64 {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    }

    /// Independent brute force: whether the closed segment `p`–`q` meets
    /// the *open* hexagon `h`, by the separating axis theorem over the
    /// hexagon's edge normals and the segment's normal (an open convex set
    /// and a compact one are disjoint iff some such axis puts them on
    /// either side of a line, the open one strictly).
    fn segment_enters(p: [i64; 2], q: [i64; 2], h: HexCoord) -> bool {
        let poly = corners(h);
        let mut axes: Vec<[i64; 2]> = (0..6)
            .map(|i| {
                let (a, b) = (poly[i], poly[(i + 1) % 6]);
                [b[1] - a[1], a[0] - b[0]]
            })
            .collect();
        axes.push([q[1] - p[1], p[0] - q[0]]);
        axes.iter().all(|&ax| {
            let (lo, hi) = poly.iter().fold((i64::MAX, i64::MIN), |(lo, hi), &v| {
                (lo.min(dot(ax, v)), hi.max(dot(ax, v)))
            });
            let (a, b) = (dot(ax, p), dot(ax, q));
            // Closed [min(a,b), max(a,b)] meets open (lo, hi).
            a.max(b) > lo && a.min(b) < hi
        })
    }

    /// Whether the segment touches a corner of any hex (a degenerate ray).
    fn touches_a_corner(p: [i64; 2], q: [i64; 2], hexes: &[HexCoord]) -> bool {
        hexes.iter().any(|&h| {
            corners(h).iter().any(|&v| {
                cross(p, q, v) == 0
                    && dot([v[0] - p[0], v[1] - p[1]], [q[0] - p[0], q[1] - p[1]]) >= 0
                    && dot([v[0] - q[0], v[1] - q[1]], [p[0] - q[0], p[1] - q[1]]) >= 0
            })
        })
    }

    /// The ray's hexes are exactly the hexes the segment between the hex
    /// centres enters, by an independent brute force (separating axes over
    /// every hex near the ray), for every target within range 12 of a few
    /// origins (both row parities). A ray touching no corner has one path,
    /// consecutive hexes adjacent; a ray through corners has two, each
    /// covering every hex the segment enters plus only hexes it touches.
    #[rulebook("§6.3")]
    #[test]
    fn los_ray_sides_are_the_hexes_the_segment_enters() {
        for origin in [
            HexCoord::new(0, 0),
            HexCoord::new(30, 37),
            HexCoord::new(-3, 5),
        ] {
            let near = disc(origin, 14);
            for to in disc(origin, 12) {
                if to == origin {
                    continue;
                }
                let (p, q) = (centre(origin), centre(to));
                let mut entered: Vec<HexCoord> = near
                    .iter()
                    .copied()
                    .filter(|&h| segment_enters(p, q, h))
                    .collect();
                entered.sort();
                let sides = los_ray_sides(origin, to);
                for side in &sides {
                    assert_eq!(side.first(), Some(&origin));
                    assert_eq!(side.last(), Some(&to));
                    assert!(
                        side.windows(2).all(|w| w[0].is_adjacent_to(w[1])),
                        "{origin} -> {to}: {side:?} steps between neighbours"
                    );
                    let mut sorted = side.clone();
                    sorted.sort();
                    sorted.dedup();
                    assert_eq!(sorted.len(), side.len(), "{origin} -> {to}: no hex twice");
                    assert!(
                        entered.iter().all(|h| side.contains(h)),
                        "{origin} -> {to}: {side:?} misses an entered hex of {entered:?}"
                    );
                }
                if touches_a_corner(p, q, &near) {
                    // The extra hexes of each side are only touched.
                    for side in &sides {
                        for h in side {
                            assert!(
                                entered.contains(h)
                                    || corners(*h).iter().any(|&v| cross(p, q, v) == 0),
                                "{origin} -> {to}: {h} is neither entered nor touched"
                            );
                        }
                    }
                } else {
                    let mut walked = sides[0].clone();
                    walked.sort();
                    assert_eq!(walked, entered, "{origin} -> {to}");
                    assert_eq!(sides[0], sides[1], "{origin} -> {to}: no tie, one path");
                }
            }
        }
    }

    /// On the corner-direction lines, where the ray runs along hexsides,
    /// the two sides are the two sides `HexCoord::line_between` and
    /// `line_between_other_side` resolve the ties to, in that order: those
    /// lines meet no hex but the ones they sample.
    #[rulebook("§6.3")]
    #[test]
    fn los_ray_sides_along_hexsides_are_the_two_tie_sides() {
        let from = HexCoord::new(4, 7);
        for to in disc(from, 12) {
            if !from.line_runs_along_hexsides(to) {
                continue;
            }
            let with_ends = |line: Vec<HexCoord>| {
                let mut path = vec![from];
                path.extend(line);
                path.push(to);
                path
            };
            assert_eq!(
                los_ray_sides(from, to),
                [
                    with_ends(from.line_between(to)),
                    with_ends(from.line_between_other_side(to)),
                ],
                "{from} -> {to}"
            );
        }
    }

    /// The defects of the sampled line, on the exact one: at ranges 7, 9
    /// and 10 a line can clip hex corners between the samples, and a line
    /// crossing a hexside at its midpoint passes through both hexes of
    /// that hexside -- with no tie either side.
    #[rulebook("§6.3")]
    #[test]
    fn los_ray_sides_keep_corner_clips_and_midpoint_crossings() {
        let from = HexCoord::new(0, 0);
        for to in [
            HexCoord::new(1, 7),
            HexCoord::new(1, 9),
            HexCoord::new(1, 10),
        ] {
            let [side, other] = los_ray_sides(from, to);
            assert_eq!(side, other, "{from} -> {to}: no tie");
            // Every sampled hex (either side of a sampled tie), the two
            // endpoints, and at least two clipped corners besides.
            let sampled = from.line_between(to);
            let other_sampled = from.line_between_other_side(to);
            assert!(sampled.iter().all(|h| side.contains(h)), "{side:?}");
            assert!(other_sampled.iter().all(|h| side.contains(h)), "{side:?}");
            assert!(side.len() >= sampled.len() + 4, "{to}: {side:?}");
        }
        // (0,0) -> (1,4) crosses the (0,2)|(1,2) hexside at its midpoint:
        // both hexes, the hexside between them crossed, either side.
        let [side, other] = los_ray_sides(from, HexCoord::new(1, 4));
        assert_eq!(side, other);
        let (a, b) = (HexCoord::new(1, 2), HexCoord::new(0, 2));
        let at = side.iter().position(|h| *h == a).expect("passes (1,2)");
        assert_eq!(side.get(at + 1), Some(&b), "{side:?}");
    }
}
