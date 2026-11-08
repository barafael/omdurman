//! Any record: its fields, and every edge that touches it, in and out.

use leptos::prelude::*;

use crate::ui::{list, query_chip, rid_link, show, text, use_query, value_view};

const EDGES: [&str; 17] = [
    "is_a",
    "belongs_to",
    "leads",
    "member_of",
    "fires_on",
    "capability",
    "entry",
    "governs",
    "concerns",
    "exception_to",
    "allowed_in",
    "prohibits",
    "next",
    "about",
    "raises",
    "appears_in",
    "cites",
];

fn record_sql(rid: &str) -> String {
    let edges = EDGES
        .iter()
        .map(|e| format!("(SELECT '{e}' AS rel, in, out FROM {e} WHERE in = $r OR out = $r)"))
        .collect::<Vec<_>>()
        .join(",\n  ");
    format!("LET $r = {rid};\nRETURN {{ fields: $r.*, edges: array::flatten([\n  {edges}\n]) }};")
}

#[component]
pub fn RecordPage(id: String) -> impl IntoView {
    let sql = record_sql(&id);
    let s2 = sql.clone();
    let res = use_query(move || sql.clone());
    let rid = id.clone();
    view! {
        <section class="page-head">
            <p class="kicker">"record"</p>
            <h1>{id.clone()}</h1>
        </section>
        {show(res, move |v| {
            let edges = list(&v["edges"]);
            let me = rid.clone();
            let (outgoing, incoming): (Vec<_>, Vec<_>) = edges.into_iter().partition(|e| text(&e["in"]) == me);
            view! {
                <div class="record">
                    <div>{if v["fields"].is_null() { view! { <p class="error">"No such record."</p> }.into_any() } else { value_view(&v["fields"]) }}</div>
                    <div>
                        <h3>"Out"</h3>
                        <ul class="plain">{outgoing.into_iter().map(|e| view! { <li><span class="rel">{text(&e["rel"])}</span>" → "{rid_link(&text(&e["out"]), None)}</li> }).collect_view()}</ul>
                        <h3>"In"</h3>
                        <ul class="plain">{incoming.into_iter().map(|e| view! { <li>{rid_link(&text(&e["in"]), None)}" → "<span class="rel">{text(&e["rel"])}</span></li> }).collect_view()}</ul>
                    </div>
                </div>
            }
        })}
        {query_chip(move || s2.clone())}
    }
}
