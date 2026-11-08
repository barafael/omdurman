//! The field: a schematic of the theatre (placed only from what the set-up rules and historical notes
//! say), the named places of both maps, the terrain, and how zones of control cross it.

use leptos::prelude::*;
use serde_json::Value;

use crate::router::{Route, go, href};
use crate::ui::{list, query_chip, sec_link, show, strings, text, use_query};

const PLACES: &str = "SELECT id, name, kind, map.name AS map, hexes, bank.name AS bank, (->member_of->location.name)[0] AS part_of,
  description, note, history, section.num AS section FROM location ORDER BY map, kind, name;";
const TERRAIN: &str =
    "SELECT id, name, kind, effects, zoc, section.num AS section FROM terrain ORDER BY kind, name;";
const ZOC: &str = "SELECT boundary.name AS boundary, out_of, into, note, exception FROM zoc_boundary ORDER BY boundary;";

/// A named place on the schematic; clicking it opens the place's record. (An `<a>` built outside the
/// `<svg>` literal would be an HTML element, which SVG does not draw, so the link is a click handler.)
fn place(
    x: f64,
    y: f64,
    rid: &'static str,
    label: &'static str,
    anchor: &'static str,
) -> impl IntoView {
    let tx = match anchor {
        "end" => x - 8.0,
        "middle" => x,
        _ => x + 8.0,
    };
    let ty = if anchor == "middle" { y - 9.0 } else { y + 4.0 };
    view! {
        <g class="map-place" role="link" on:click=move |_| go(Route::Record(rid.into()))>
            <title>{rid}</title>
            <circle cx=x cy=y r="4"/>
            <text x=tx y=ty text-anchor=anchor>{label}</text>
        </g>
    }
}

fn east(y: f64, rid: &'static str, label: &'static str) -> impl IntoView {
    view! { <g class="map-place" role="link" on:click=move |_| go(Route::Record(rid.into()))><text x="452" y=y>{label}</text></g> }
}

fn schematic() -> impl IntoView {
    view! {
        <figure class="schematic">
            <svg viewBox="0 0 640 760" role="img" aria-label="Schematic of the Omdurman battlefield">
                <rect class="bank west" x="0" y="0" width="400" height="760"/>
                <rect class="bank east" x="400" y="0" width="240" height="760"/>
                <path class="nile" d="M405 0 C 385 120, 440 230, 410 350 S 360 590, 395 760"/>
                {(0..5).map(|i| { let y = 680.0 - i as f64 * 150.0; view! { <path class="current" d=format!("M398 {y} l0 -26 m-5 8 l5 -8 l5 8")/> } }).collect_view()}
                <ellipse class="island" cx="388" cy="610" rx="7" ry="20"/>
                <ellipse class="island" cx="400" cy="690" rx="6" ry="14"/>
                <path class="khor" d="M30 548 C 140 560, 260 545, 386 585"/>
                <text class="khor-label" x="120" y="540">"Khor Shambat"</text>
                <path class="zariba" d="M402 238 C 330 245, 330 330, 402 338"/>
                <rect class="walled" x="250" y="640" width="70" height="52" rx="4"/>
                <ellipse class="town" cx="285" cy="666" rx="92" ry="58"/>
                <text class="edge" x="196" y="20" text-anchor="middle">"NORTH EDGE · Anglo-Egyptian gunboats enter (9.113)"</text>
                <text class="edge" x="196" y="752" text-anchor="middle">"SOUTH EDGE · the Khartoum mini-map adjoins it (2.1)"</text>
                <text class="edge vertical" x="16" y="380" text-anchor="middle" transform="rotate(-90 16 380)">"WEST EDGE · Dervish reinforcements, south of the Khor Shambat (9.112)"</text>
                <text class="bank-label" x="200" y="60" text-anchor="middle">"WEST BANK"</text>
                <text class="bank-label" x="525" y="60" text-anchor="middle">"EAST BANK"</text>
                <text class="river-label" x="430" y="470" transform="rotate(80 430 470)">"NILE · current runs north"</text>
                {place(300.0, 92.0, "location:ae_entrance_area", "Anglo-Egyptian entrance area", "middle")}
                {place(250.0, 160.0, "location:kerreri", "Kerreri (huts)", "end")}
                {place(140.0, 120.0, "location:kerreri_hills", "Kerreri Hills (§11)", "end")}
                {place(350.0, 290.0, "location:zariba", "the Zariba", "end")}
                {place(200.0, 420.0, "location:jebel_surgham", "Jebel Surgham (§11)", "end")}
                {place(285.0, 650.0, "location:walled_city", "walled city", "middle")}
                {place(300.0, 676.0, "location:mahdis_tomb", "Mahdi's Tomb", "start")}
                {place(200.0, 712.0, "location:omdurman", "Omdurman", "end")}
                {place(378.0, 630.0, "location:nile_islands", "islands", "end")}
                <g class="eastbox">
                    <rect x="440" y="300" width="186" height="118" rx="6"/>
                    <text x="452" y="322">"East bank places"</text>
                    {east(346.0, "location:abu_alim", "· Abu Alim (Friendlies enter)")}
                    {east(368.0, "location:halfaya", "· Halfaya huts")}
                    {east(390.0, "location:el_debeba", "· El Debeba (Isa Zachneih)")}
                    <text x="452" y="410" class="tiny">"their order along the bank is not in the rules"</text>
                </g>
            </svg>
            <figcaption>"Schematic, not to scale. The rules give no coordinates: places are set from the set-up rules
               (§9.111–9.113, §9.211), the map description (§2.1) and the historical notes (§11). The real board lives on the mapsheet."</figcaption>
        </figure>
    }
}

#[component]
pub fn Field() -> impl IntoView {
    let places = use_query(|| PLACES.into());
    let terrain = use_query(|| TERRAIN.into());
    let zoc = use_query(|| ZOC.into());
    view! {
        <section class="page-head">
            <h1>"The Field"</h1>
            <p class="lede">"About a hundred square miles north of Omdurman, in hexes of four hundred yards: the Nile and its
               two banks, the dry khor, the huts of Kerreri, the walled city with the Mahdi's Tomb. And, for the bonus game,
               Khartoum as it stood in 1885."</p>
        </section>
        <div class="field-top">
            {schematic()}
            <div>
                <h2 class="rule">"Zones of control across the ground"</h2>
                <p class="small">"Every unit but an Anglo-Egyptian leader projects a zone of control into its six neighbours (§5.41).
                   Whether it crosses a boundary depends on the boundary (§5.44):"</p>
                {show(zoc, |v| view! {
                    <table class="zoc">
                        <thead><tr><th>"boundary"</th><th>"out of"</th><th>"into"</th><th></th></tr></thead>
                        <tbody>{list(&v).into_iter().map(|z| view! {
                            <tr>
                                <td>{text(&z["boundary"])}</td>
                                <td class=if z["out_of"] == Value::Bool(true) { "yes" } else { "no" }>{if z["out_of"] == Value::Bool(true) { "✓" } else { "✗" }}</td>
                                <td class=if z["into"] == Value::Bool(true) { "yes" } else { "no" }>{if z["into"] == Value::Bool(true) { "✓" } else { "✗" }}</td>
                                <td class="small">{[text(&z["note"]), text(&z["exception"])].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("; ")}</td>
                            </tr>
                        }).collect_view()}</tbody>
                    </table>
                })}
                {query_chip(|| ZOC.into())}
            </div>
        </div>
        <section>
            <h2 class="rule">"Places"</h2>
            {show(places, |v| {
                let rows = list(&v);
                let maps: Vec<String> = { let mut m: Vec<String> = rows.iter().map(|r| text(&r["map"])).collect(); m.dedup(); m };
                maps.into_iter().map(|m| {
                    let these: Vec<Value> = rows.iter().filter(|r| text(&r["map"]) == m).cloned().collect();
                    view! {
                        <h3>{if m.is_empty() { "Beyond the maps (historical notes)".to_string() } else { m.clone() }}</h3>
                        <div class="cards">{these.into_iter().map(|p| view! {
                            <a class="card place" href=href(&Route::Record(text(&p["id"])))>
                                <h4>{text(&p["name"])}</h4>
                                <p class="small">{[text(&p["kind"]), text(&p["bank"]), text(&p["part_of"]).is_empty().then(String::new).unwrap_or(format!("in {}", text(&p["part_of"])))]
                                    .into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ")}</p>
                                {[text(&p["description"]), text(&p["note"]), text(&p["history"])].into_iter().filter(|s| !s.is_empty()).map(|s| view! { <p>{s}</p> }).collect_view()}
                                <p class="small">{format!("§{}", text(&p["section"]))}</p>
                            </a>
                        }).collect_view()}</div>
                    }
                }).collect_view()
            })}
            {query_chip(|| PLACES.into())}
        </section>
        <section>
            <h2 class="rule">"Terrain"</h2>
            {show(terrain, |v| view! {
                <div class="cards">{list(&v).into_iter().map(|t| view! {
                    <div class="card">
                        <h4>{text(&t["name"])}<span class="small">{format!(" · {}", text(&t["kind"]))}</span></h4>
                        <ul class="plain small">{strings(&t["effects"]).into_iter().map(|e| view! { <li>{e}</li> }).collect_view()}</ul>
                        {(!text(&t["zoc"]).is_empty()).then(|| view! { <p class="small">{format!("ZOC: {}", text(&t["zoc"]))}</p> })}
                        <p class="small">{sec_link(&text(&t["section"]))}</p>
                    </div>
                }).collect_view()}</div>
            })}
            <p class="small">"Movement costs and fire modifiers per terrain are on the Terrain Effects Chart printed on the mapsheet, not in the rulebook text."</p>
            {query_chip(|| TERRAIN.into())}
        </section>
    }
}
