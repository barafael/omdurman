//! The field: the named places of both maps, the terrain, and how zones of control cross it.

use leptos::prelude::*;
use serde_json::Value;

use crate::router::{Route, href};
use crate::ui::{list, query_chip, sec_link, show, strings, text, use_query};

const PLACES: &str = "SELECT id, name, kind, map.name AS map, hexes, bank.name AS bank, (->member_of->location.name)[0] AS part_of,
  description, note, history, section.num AS section FROM location ORDER BY map, kind, name;";
const TERRAIN: &str =
    "SELECT id, name, kind, effects, zoc, section.num AS section FROM terrain ORDER BY kind, name;";
const ZOC: &str = "SELECT boundary.name AS boundary, out_of, into, note, exception FROM zoc_boundary ORDER BY boundary;";

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
