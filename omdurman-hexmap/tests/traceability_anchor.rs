//! Compile anchor for the traceability matrix's single `omdurman-hexmap`
//! citation: `docs/traceability.toml` lists `GameMap::roads` (§6.3 road
//! overlay) as an impl site. The matrix checker requires every cited symbol
//! to be compiler-anchored; for the hexmap symbol that anchor lives here, in
//! the crate that owns the type, so `cargo test -p omdurman-rules` does not
//! have to pull Bevy in via a dev-dependency.

#[test]
fn gamemap_roads_field_exists() {
    let map = omdurman_hexmap::GameMap::default();
    // Field anchor: renaming or removing `roads` breaks this line.
    let _ = &map.roads;
}
