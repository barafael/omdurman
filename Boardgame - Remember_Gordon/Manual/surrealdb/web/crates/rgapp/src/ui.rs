//! Shared pieces: queries as resources, result rendering with record links, the "query behind this"
//! chip, rulebook text, and the game's counters drawn as cardboard chits.

use leptos::prelude::*;
use serde_json::Value;

use crate::kb::Kb;
use crate::router::{Route, href, record_route, section_num};

// ------------------------------------------------------------------ values

pub fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

pub fn list(v: &Value) -> Vec<Value> {
    v.as_array().cloned().unwrap_or_default()
}

pub fn strings(v: &Value) -> Vec<String> {
    list(v).iter().map(text).filter(|s| !s.is_empty()).collect()
}

/// A d10, 1..=10.
pub fn d10() -> i64 {
    (js_sys::Math::random() * 10.0).floor() as i64 + 1
}

// ------------------------------------------------------------------ queries as resources

/// A query that runs once the archive is loaded and whenever the signals `sql` reads change.
/// `Err("")` means "not loaded yet".
pub fn use_query(sql: impl Fn() -> String + 'static) -> LocalResource<Result<Value, String>> {
    let kb = expect_context::<Kb>();
    LocalResource::new(move || {
        let ready = kb.ready.get();
        let q = sql();
        async move {
            if ready {
                kb.last(q).await
            } else {
                Err(String::new())
            }
        }
    })
}

pub fn loading() -> AnyView {
    view! { <p class="loading">"A rider is bringing the dispatch…"</p> }.into_any()
}

/// Render a query resource: loading, error, or `f(value)`.
pub fn show<V: IntoView + 'static>(
    res: LocalResource<Result<Value, String>>,
    f: impl Fn(Value) -> V + Send + Sync + 'static,
) -> impl IntoView {
    move || match res.get() {
        None => loading(),
        Some(Err(e)) if e.is_empty() => loading(),
        Some(Err(e)) => view! { <p class="error">{e}</p> }.into_any(),
        Some(Ok(v)) => f(v).into_any(),
    }
}

// ------------------------------------------------------------------ links and result rendering

/// A pretty label for a record id: "section:`5.21`" → "§5.21", others unchanged.
pub fn pretty_rid(rid: &str) -> String {
    if rid.starts_with("section:") {
        format!("§{}", section_num(rid))
    } else {
        rid.to_string()
    }
}

/// A link to a record's view if `rid` is a record id; plain text otherwise.
pub fn rid_link(rid: &str, label: Option<String>) -> AnyView {
    let kb = expect_context::<Kb>();
    match kb.record_table(rid) {
        Some(table) => {
            let h = href(&record_route(&table, rid));
            let named = label.is_some();
            let label = label.unwrap_or_else(|| pretty_rid(rid));
            view! { <a class="rid" class:named=named href=h title=rid.to_string()>{label}</a> }
                .into_any()
        }
        None => label.unwrap_or_else(|| rid.to_string()).into_any(),
    }
}

/// A link to a rulebook section by number.
pub fn sec_link(num: &str) -> AnyView {
    if num.is_empty() {
        return ().into_any();
    }
    view! { <a class="sec" href=href(&Route::Rules(Some(num.to_string())))>{format!("§{num}")}</a> }
        .into_any()
}

const LABEL_FIELDS: [&str; 4] = ["name", "title", "label", "text"];

fn label_of(v: &Value) -> Option<String> {
    let o = v.as_object()?;
    LABEL_FIELDS
        .iter()
        .find_map(|k| o.get(*k)?.as_str().map(String::from))
}

/// Any query result, as tables (arrays of objects), chips (arrays of scalars) or key/value lists.
/// Record ids are links; an `{id, name}` object is one link labelled with its name.
pub fn value_view(v: &Value) -> AnyView {
    match v {
        Value::Null => view! { <span class="null">"NONE"</span> }.into_any(),
        Value::Bool(b) => view! { <span class=if *b { "bool yes" } else { "bool no" }>{if *b { "true" } else { "false" }}</span> }.into_any(),
        Value::Number(n) => view! { <span class="num">{n.to_string()}</span> }.into_any(),
        Value::String(s) => view! { <span class="str">{rid_link(s, None)}</span> }.into_any(),
        Value::Array(a) if a.is_empty() => view! { <span class="null">"[ ]"</span> }.into_any(),
        Value::Array(a) if a.iter().all(|x| !x.is_object() && !x.is_array()) => view! {
            <span class="chips">{a.iter().map(|x| view! { <span class="chip">{value_view(x)}</span> }).collect_view()}</span>
        }.into_any(),
        Value::Array(a) if a.iter().all(|x| x.as_object().is_some_and(|o| o.len() == 2 && o.contains_key("id")) && label_of(x).is_some()) => view! {
            <span class="chips">{a.iter().map(|x| view! { <span class="chip">{rid_link(&text(&x["id"]), label_of(x))}</span> }).collect_view()}</span>
        }.into_any(),
        Value::Array(a) if a.iter().all(|x| x.is_object()) => {
            let mut cols: Vec<String> = vec![];
            for row in a {
                for k in row.as_object().into_iter().flat_map(|o| o.keys()) {
                    if !cols.contains(k) {
                        cols.push(k.clone());
                    }
                }
            }
            if cols.len() > 9 || cols.is_empty() {
                return view! { <ol class="rows">{a.iter().map(|x| view! { <li>{value_view(x)}</li> }).collect_view()}</ol> }.into_any();
            }
            let head = cols.clone();
            view! {
                <div class="table-wrap"><table class="result">
                    <thead><tr>{head.into_iter().map(|c| view! { <th>{c}</th> }).collect_view()}</tr></thead>
                    <tbody>{a.iter().map(|row| {
                        let cells = cols.iter().map(|c| row.get(c).cloned().unwrap_or(Value::Null)).collect::<Vec<_>>();
                        view! { <tr>{cells.iter().map(|x| view! { <td>{value_view(x)}</td> }).collect_view()}</tr> }
                    }).collect_view()}</tbody>
                </table></div>
            }.into_any()
        }
        Value::Array(a) => view! { <ol class="rows">{a.iter().map(|x| view! { <li>{value_view(x)}</li> }).collect_view()}</ol> }.into_any(),
        Value::Object(o) => {
            if let (Some(Value::String(id)), Some(label)) = (o.get("id"), label_of(v)) {
                if o.len() == 2 {
                    return rid_link(id, Some(label));
                }
            }
            view! {
                <dl class="obj">{o.iter().map(|(k, x)| view! { <dt>{k.clone()}</dt><dd>{value_view(x)}</dd> }).collect_view()}</dl>
            }.into_any()
        }
    }
}

// ------------------------------------------------------------------ the query behind a view

/// A folded "dispatch" showing the SurrealQL a panel ran, with a link to open it in the notebook.
pub fn query_chip(sql: impl Fn() -> String + Send + Sync + 'static) -> impl IntoView {
    let open = RwSignal::new(false);
    let sql = std::sync::Arc::new(sql);
    let s2 = sql.clone();
    view! {
        <div class="query-chip">
            <button class="link" on:click=move |_| open.update(|o| *o = !*o)>
                {move || if open.get() { "▾ the query" } else { "▸ the query" }}
            </button>
            <Show when=move || open.get()>
                {let s = s2.clone(); move || { let q = s(); view! {
                    <pre class="sql">{q.clone()}</pre>
                    <a class="small" href=href(&Route::Notebook(Some(q)))>"open in the notebook →"</a>
                } }}
            </Show>
        </div>
    }
}

// ------------------------------------------------------------------ rulebook text

/// Manual text (light markdown): paragraphs, "- " bullets, **bold**, tables kept as rows.
pub fn manual_text(t: &str) -> impl IntoView + use<> {
    let mut blocks: Vec<AnyView> = vec![];
    let mut bullets: Vec<String> = vec![];
    let mut para: Vec<String> = vec![];
    let flush_para = |para: &mut Vec<String>, blocks: &mut Vec<AnyView>| {
        if !para.is_empty() {
            let p = para.join(" ");
            blocks.push(view! { <p>{inline(&p)}</p> }.into_any());
            para.clear();
        }
    };
    let flush_bullets = |bullets: &mut Vec<String>, blocks: &mut Vec<AnyView>| {
        if !bullets.is_empty() {
            let items = std::mem::take(bullets);
            blocks.push(view! { <ul>{items.into_iter().map(|b| view! { <li>{inline(&b)}</li> }).collect_view()}</ul> }.into_any());
        }
    };
    for line in t.lines() {
        let l = line.trim();
        if l.is_empty() {
            flush_para(&mut para, &mut blocks);
            flush_bullets(&mut bullets, &mut blocks);
        } else if let Some(b) = l.strip_prefix("- ") {
            flush_para(&mut para, &mut blocks);
            bullets.push(b.to_string());
        } else if l.starts_with('|') {
            flush_para(&mut para, &mut blocks);
            flush_bullets(&mut bullets, &mut blocks);
            if !l.starts_with("|---") {
                let cells: Vec<String> = l
                    .trim_matches('|')
                    .split('|')
                    .map(|c| c.trim().to_string())
                    .collect();
                blocks.push(view! { <div class="mrow">{cells.into_iter().map(|c| view! { <span>{c}</span> }).collect_view()}</div> }.into_any());
            }
        } else {
            flush_bullets(&mut bullets, &mut blocks);
            para.push(l.to_string());
        }
    }
    flush_para(&mut para, &mut blocks);
    flush_bullets(&mut bullets, &mut blocks);
    view! { <div class="manual">{blocks}</div> }
}

/// **bold** and *italic* spans.
fn inline(s: &str) -> impl IntoView + use<> {
    let mut out: Vec<AnyView> = vec![];
    for (i, part) in s.split("**").enumerate() {
        if i % 2 == 1 {
            out.push(view! { <strong>{part.to_string()}</strong> }.into_any());
        } else {
            for (j, p) in part.split('*').enumerate() {
                if j % 2 == 1 {
                    out.push(view! { <em>{p.to_string()}</em> }.into_any())
                } else {
                    out.push(p.to_string().into_any())
                }
            }
        }
    }
    out
}

// ------------------------------------------------------------------ counters

/// The glyph family for a unit class id.
pub fn kind_of(class: &str) -> &'static str {
    let c = class.trim_start_matches("unit_class:");
    match c {
        "ae_cavalry" | "cavalry" => "cavalry",
        "d_camel" | "camel_corps" | "camel" => "camel",
        "d_artillery" | "ae_artillery" | "artillery" => "artillery",
        "maxim_battery" => "maxim",
        "d_gunboat" | "old_gunboat" | "named_gunboat" | "gunboat" => "gunboat",
        "d_leader" | "ae_leader" | "leader" => "leader",
        "fort" => "fort",
        "royal_engineers" => "engineers",
        _ => "infantry",
    }
}

/// A unit-type symbol in a 40×28 box, after the map symbols of the period's wargames.
pub fn glyph(kind: &str) -> impl IntoView + use<> {
    let body = match kind {
        "infantry" => view! { <rect x="6" y="5" width="28" height="18"/><path d="M6 5 L34 23 M34 5 L6 23"/> }.into_any(),
        "cavalry" => view! { <rect x="6" y="5" width="28" height="18"/><path d="M6 23 L34 5"/> }.into_any(),
        "camel" => view! { <rect x="6" y="5" width="28" height="18"/><path d="M9 19 Q14 8 20 15 Q26 8 31 19"/> }.into_any(),
        "artillery" => view! { <rect x="6" y="5" width="28" height="18"/><circle cx="20" cy="14" r="3.5" class="fill"/> }.into_any(),
        "maxim" => view! { <rect x="6" y="5" width="28" height="18"/><circle cx="15" cy="14" r="2.5" class="fill"/><circle cx="25" cy="14" r="2.5" class="fill"/> }.into_any(),
        "engineers" => view! { <rect x="6" y="5" width="28" height="18"/><path d="M12 19 V10 H28 V19 M20 10 V19"/> }.into_any(),
        "gunboat" => view! { <path d="M3 16 H37 L32 23 H8 Z"/><path d="M17 16 V6 M17 7 L26 11 L17 13"/> }.into_any(),
        "leader" => view! { <path class="fill" d="M20 3 L23.5 11 L32 11.5 L25.5 17 L27.5 25 L20 20.5 L12.5 25 L14.5 17 L8 11.5 L16.5 11 Z"/> }.into_any(),
        "fort" => view! { <path d="M20 3 L35 13 L30 25 H10 L5 13 Z"/><path d="M20 9 L28 14 L25 21 H15 L12 14 Z"/> }.into_any(),
        _ => view! { <rect x="6" y="5" width="28" height="18"/> }.into_any(),
    };
    view! { <svg class="glyph" viewBox="0 0 40 28" aria-hidden="true">{body}</svg> }
}

/// A unit as a cardboard counter.
pub fn counter(
    name: String,
    side: &str,
    kind: &str,
    values: String,
    color: Option<String>,
    small: bool,
) -> impl IntoView + use<> {
    let class = format!(
        "counter {} {}{}",
        side,
        kind,
        if small { " small" } else { "" }
    );
    let style = color.map(|c| format!("--tribe:{c}")).unwrap_or_default();
    let long = values.chars().count() > 9;
    view! {
        <div class=class style=style>
            <div class="c-name">{name}</div>
            {glyph(kind)}
            <div class="c-vals" class:long=long>{if values.is_empty() { "· · ·".to_string() } else { values }}</div>
        </div>
    }
}

/// The CSS colour for a tribe's command colour, when the manual names one.
pub fn tribe_color(name: &str) -> Option<String> {
    match name {
        "green" => Some("var(--green-command)".into()),
        _ => None,
    }
}

/// "faction:anglo_egyptian" → "ae", "faction:dervish" → "dv".
pub fn side_of(faction: &str) -> &'static str {
    if faction.contains("anglo") {
        "ae"
    } else {
        "dv"
    }
}
