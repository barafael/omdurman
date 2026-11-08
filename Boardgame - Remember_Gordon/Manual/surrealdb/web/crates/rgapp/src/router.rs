//! Hash routes, so every view is a shareable link and the site works as plain static files:
//! `#/`, `#/armies[/unit:sultan]`, `#/field`, `#/clock`, `#/scenarios[/scenario:campaign]`, `#/office`,
//! `#/rules[/5.21]`, `#/dispatches`, `#/inquiries[/<question id>]`, `#/notebook?q=…`,
//! `#/record/<record id>`, `#/about`.

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Home,
    Armies(Option<String>),
    Field,
    Clock,
    Scenarios(Option<String>),
    Office,
    Rules(Option<String>),
    Dispatches,
    Inquiries(Option<String>),
    Notebook(Option<String>),
    Record(String),
    About,
}

fn decode(s: &str) -> String {
    js_sys::decode_uri_component(s)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| s.to_string())
}

fn encode(s: &str) -> String {
    js_sys::encode_uri_component(s).into()
}

pub fn parse(hash: &str) -> Route {
    let h = hash.trim_start_matches('#').trim_start_matches('/');
    let (path, query) = h.split_once('?').unwrap_or((h, ""));
    let (page, arg) = path
        .split_once('/')
        .map(|(p, a)| (p, Some(decode(a))))
        .unwrap_or((path, None));
    let arg = arg.filter(|a| !a.is_empty());
    match page {
        "" => Route::Home,
        "armies" => Route::Armies(arg),
        "field" => Route::Field,
        "clock" => Route::Clock,
        "scenarios" => Route::Scenarios(arg),
        "office" => Route::Office,
        "rules" => Route::Rules(arg),
        "dispatches" => Route::Dispatches,
        "inquiries" => Route::Inquiries(arg),
        "record" => arg.map(Route::Record).unwrap_or(Route::Home),
        "about" => Route::About,
        _ => Route::Notebook(query.strip_prefix("q=").map(decode)),
    }
}

pub fn href(r: &Route) -> String {
    let with = |p: &str, a: &Option<String>| match a {
        Some(a) => format!("#/{p}/{}", encode(a)),
        None => format!("#/{p}"),
    };
    match r {
        Route::Home => "#/".into(),
        Route::Armies(a) => with("armies", a),
        Route::Field => "#/field".into(),
        Route::Clock => "#/clock".into(),
        Route::Scenarios(a) => with("scenarios", a),
        Route::Office => "#/office".into(),
        Route::Rules(a) => with("rules", a),
        Route::Dispatches => "#/dispatches".into(),
        Route::Inquiries(a) => with("inquiries", a),
        Route::Notebook(None) => "#/notebook".into(),
        Route::Notebook(Some(q)) => format!("#/notebook?q={}", encode(q)),
        Route::Record(id) => format!("#/record/{}", encode(id)),
        Route::About => "#/about".into(),
    }
}

fn current_hash() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
}

/// The current route, kept in sync with `hashchange`.
#[derive(Clone, Copy)]
pub struct Router(pub RwSignal<Route>);

impl Router {
    pub fn start() -> Router {
        let route = RwSignal::new(parse(&current_hash()));
        let on_change = Closure::<dyn FnMut()>::new(move || {
            route.set(parse(&current_hash()));
            if let Some(w) = web_sys::window() {
                w.scroll_to_with_x_and_y(0.0, 0.0);
            }
        });
        if let Some(w) = web_sys::window() {
            let _ = w
                .add_event_listener_with_callback("hashchange", on_change.as_ref().unchecked_ref());
        }
        on_change.forget();
        Router(route)
    }
}

/// Navigate (adds a history entry).
pub fn go(r: Route) {
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash(&href(&r));
    }
}

/// Update the URL without a history entry or a route change.
pub fn replace(r: &Route) {
    if let Some(w) = web_sys::window() {
        let _ = w
            .history()
            .and_then(|h| h.replace_state_with_url(&JsValue::NULL, "", Some(&href(r))));
    }
}

/// "section:`5.21`" / "section:⟨5.21⟩" → "5.21".
pub fn section_num(rid: &str) -> String {
    rid.trim_start_matches("section:")
        .trim_matches(|c| c == '`' || c == '⟨' || c == '⟩')
        .to_string()
}

/// A record key without its table and quoting: "question:`s5_24_x`" → "s5_24_x".
pub fn key_of(rid: &str) -> String {
    rid.split_once(':')
        .map_or(rid, |(_, k)| k)
        .trim_matches(|c| c == '`' || c == '⟨' || c == '⟩')
        .to_string()
}

/// Where a record id from a result leads.
pub fn record_route(table: &str, rid: &str) -> Route {
    match table {
        "unit" | "tribe" | "unit_class" | "command_group" | "formation" => {
            Route::Armies(Some(rid.into()))
        }
        "section" => Route::Rules(Some(section_num(rid))),
        "scenario" => Route::Scenarios(Some(rid.into())),
        "question" => Route::Inquiries(Some(key_of(rid))),
        "open_question" => Route::Dispatches,
        "turn" | "phase" => Route::Clock,
        _ => Route::Record(rid.into()),
    }
}
