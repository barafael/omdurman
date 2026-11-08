//! REMEMBER GORDON! — an Omdurman archive. A static, client-side site: the rulebook corpus runs as
//! SurrealDB (WebAssembly) in a Web Worker (crates/rgworker); this crate is the UI. Views are hash
//! routes (router.rs). Built after the bavovniatko knowledge-base explorer.

mod armies;
mod clock;
mod field;
mod home;
mod inquiries;
mod kb;
mod notebook;
mod office;
mod record;
mod router;
mod rules;
mod scenarios;
mod ui;

use leptos::prelude::*;

use kb::{Kb, Status};
use router::{Route, Router, href};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Night turn: the archive's dark theme (stored per browser).
fn set_night(on: bool) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = el.set_attribute("data-theme", if on { "dark" } else { "light" });
    }
    if let Some(s) = storage() {
        let _ = s.set_item("rg-night", if on { "1" } else { "0" });
    }
}

#[component]
fn App() -> impl IntoView {
    let kb = Kb::start();
    provide_context(kb);
    let Router(route) = Router::start();
    provide_context(Router(route));
    let night = RwSignal::new(
        storage()
            .and_then(|s| s.get_item("rg-night").ok().flatten())
            .as_deref()
            == Some("1"),
    );
    Effect::new(move |_| set_night(night.get()));

    let tab = move |target: Route, label: &'static str| {
        let t = target.clone();
        let active = move || std::mem::discriminant(&route.get()) == std::mem::discriminant(&t);
        view! { <a class="tab" class:active=active href=href(&target)>{label}</a> }
    };
    view! {
        <header>
            <a class="brand" href="#/">
                <span class="title">"Remember Gordon!"</span>
                <span class="subtitle">"The Battle of Omdurman · an archive of the 1982 rules"</span>
            </a>
            <nav>
                {tab(Route::Home, "Prologue")}
                {tab(Route::Armies(None), "Armies")}
                {tab(Route::Field, "Field")}
                {tab(Route::Clock, "Clock")}
                {tab(Route::Scenarios(None), "Scenarios")}
                {tab(Route::Office, "War Office")}
                {tab(Route::Rules(None), "Rulebook")}
                {tab(Route::Dispatches, "Dispatches")}
                {tab(Route::Inquiries(None), "Inquiries")}
                {tab(Route::Notebook(None), "Notebook")}
            </nav>
            <div class="header-side">
                <LoadStatus />
                <button class="night" title="Night game turn: the dark theme" on:click=move |_| night.update(|n| *n = !*n)>
                    {move || if night.get() { "☀ day turn" } else { "☾ night turn" }}
                </button>
            </div>
        </header>
        <main>
            {move || match route.get() {
                Route::Home => view! { <home::Home /> }.into_any(),
                Route::Armies(sel) => view! { <armies::Armies selected=sel /> }.into_any(),
                Route::Field => view! { <field::Field /> }.into_any(),
                Route::Clock => view! { <clock::Clock /> }.into_any(),
                Route::Scenarios(sel) => view! { <scenarios::Scenarios selected=sel /> }.into_any(),
                Route::Office => view! { <office::Office /> }.into_any(),
                Route::Rules(sel) => view! { <rules::Rulebook selected=sel /> }.into_any(),
                Route::Dispatches => view! { <rules::Dispatches /> }.into_any(),
                Route::Inquiries(sel) => view! { <inquiries::Inquiries selected=sel /> }.into_any(),
                Route::Notebook(q) => view! { <notebook::Notebook initial=q /> }.into_any(),
                Route::Record(id) => view! { <record::RecordPage id=id /> }.into_any(),
                Route::About => view! { <About /> }.into_any(),
            }}
        </main>
        <footer>
            "Rules text: "<em>"REMEMBER GORDON! — The Battle of Omdurman"</em>", Phoenix Enterprises, Ltd., 1982 (design Peter Bertram) · "
            "runs SurrealDB in your browser ("<a href="https://github.com/surrealdb/surrealdb/blob/main/LICENSE">"Business Source License 1.1"</a>") · "
            <a href="#/about">"about this archive"</a>" · "
            <a href="../">"play the game"</a>" · "
            <a href="../traceability.pdf">"rules ↔ code traceability"</a>
        </footer>
    }
}

#[component]
fn LoadStatus() -> impl IntoView {
    let kb = expect_context::<Kb>();
    move || match kb.status.get() {
        Status::Loading(p) => {
            let pct = (p.done_bytes as f64 / p.total_bytes.max(1) as f64 * 100.0).round();
            view! {
                <div class="status loading" title=p.file.clone()>
                    <span>{format!("Assembling the archive … {pct}%")}</span>
                    <progress max="100" value=pct></progress>
                </div>
            }
            .into_any()
        }
        Status::Ready { tables, ms, .. } => {
            let rows: u64 = tables.values().sum();
            view! { <div class="status ready">{format!("{rows} records · ready in {:.1} s", ms / 1000.0)}</div> }.into_any()
        }
        Status::Failed(e) => {
            view! { <div class="status failed">{format!("The archive failed to load: {e}")}</div> }
                .into_any()
        }
    }
}

#[component]
fn About() -> impl IntoView {
    let kb = expect_context::<Kb>();
    let counts = move || kb.tables.get();
    view! {
        <article class="about prose">
            <h1>"About this archive"</h1>
            <p>"This is the rulebook of "<em>"Remember Gordon! — The Battle of Omdurman"</em>" (Phoenix Enterprises, 1982),
               taken apart into a graph: its nouns (units, tribes, places, terrain, phases, turns), its verbs (move,
               fire, melee, desert…) and its rules, each tied to the paragraph of the manual it comes from. On top of it
               sit 748 questions a player at the table might ask, each written as a SurrealQL query with the answer the
               manual supports."</p>
            <p>"The whole database runs in your browser: SurrealDB 4 compiled to WebAssembly, loaded fresh on each
               visit into memory in a Web Worker, then sealed as a read-only guest session. Your queries never leave
               your machine. Nothing here is fetched from a server except the site's own files and its fonts."</p>
            <h2>"How to read it"</h2>
            <ul>
                <li>"Every fact carries its section of the manual. "<strong>"explicit"</strong>" means the manual says it; "
                    <strong>"implied"</strong>" means it follows from the text, with a note saying how."</li>
                <li>"Where the manual is silent or contradicts itself, the archive records an open question (see "
                    <a href="#/dispatches">"Dispatches"</a>") instead of picking an answer; the rule functions then answer "
                    <code>"NONE"</code>" or \"undetermined\"."</li>
                <li>"The cells of the charts printed on the mapsheet (terrain costs, range multipliers, the Combat Results
                     Table) are not in the manual's text, so they are not in the archive."</li>
            </ul>
            <h2>"What it holds"</h2>
            {move || { let c = counts(); view! {
                <div class="chips wide">{c.into_iter().filter(|(_, n)| *n > 0).map(|(t, n)| view! {
                    <span class="chip">{format!("{t} {n}")}</span>
                }).collect_view()}</div>
            } }}
            <h2>"Credits"</h2>
            <p>"Game design Peter Bertram; development Peter Bertram and Fred Chatham; rules editing Randall Mac Innis
               (from the rulebook's credits). Built with Leptos and SurrealDB, after the "
               <a href="https://github.com/barafael/bavovniatko">"bavovniatko"</a>" knowledge-base explorer."</p>
        </article>
    }
}
