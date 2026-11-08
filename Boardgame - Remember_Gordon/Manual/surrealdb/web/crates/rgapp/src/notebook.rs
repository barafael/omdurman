//! The notebook: write SurrealQL against the archive (read-only), with a gallery of example
//! questions to start from. Ctrl+Enter runs.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::kb::{Kb, QueryResult};
use crate::router::{Route, href, replace};
use crate::ui::value_view;

/// Example questions, each with the query that answers it.
pub const GALLERY: &[(&str, &str)] = &[
    (
        "Can Osman Digna stack with the Baggara?",
        "RETURN fn::stack_check([unit:osman_digna, tribe:baggara]);",
    ),
    (
        "Who may enter the walled city of Omdurman?",
        "SELECT in.name AS who, allowed, condition FROM entry WHERE out = location:walled_city ORDER BY allowed DESC;",
    ),
    (
        "Which rules are exceptions to the stacking limit?",
        "SELECT section.num AS section, statement FROM rule:r5_51_limit<-exception_to<-rule;",
    ),
    (
        "What does the Sultan fire on, and what kind of unit is it?",
        "RETURN { lines: fn::lines(unit:sultan).name, kinds: unit:sultan.classes.name };",
    ),
    (
        "What may a disrupted unit not do?",
        "RETURN state:disrupted->prohibits->action.name;",
    ),
    (
        "Every night turn on the track",
        "SELECT n, label FROM turn WHERE night ORDER BY n;",
    ),
    (
        "The order of a game turn in the historical scenario",
        "RETURN fn::sequence(scenario:historical);",
    ),
    (
        "A British brigade in integrity fires at Dervishes inside a fort: the die modifier?",
        "RETURN fn::fire_drm({ firer: faction:anglo_egyptian, direct: true, integrity: true, target_in_fort: true });",
    ),
    (
        "Gordon falls on turn 5, and 24 Dervish units are lost. The result?",
        "RETURN fn::fok_result(5, 24);",
    ),
    (
        "104 Dervish units eliminated for 18 Anglo-Egyptian: the historical net result",
        "RETURN fn::historical_result(104, 18);",
    ),
    (
        "The victory point schedule of the campaign, largest first",
        "SELECT points, (beneficiary.name ?? beneficiary) AS to, event FROM vp_award WHERE scenario = scenario:campaign ORDER BY points DESC;",
    ),
    (
        "Which tribes ride in on turn 1 with Yakub, Sherif and Ali Wad Helu?",
        "RETURN command_group:turn1_leaders<-member_of<-tribe.name;",
    ),
    (
        "Which units may sink the river chain, and how?",
        "SELECT in.name AS who, condition FROM capability WHERE out = action:sink_chain AND allowed;",
    ),
    (
        "Which sections point the reader to §6.64 (howitzers)?",
        "RETURN section:`6.64`<-cites<-section.num;",
    ),
    (
        "What the manual leaves open about stacking",
        "SELECT kind, question FROM open_question WHERE refs CONTAINS section:`5.51`;",
    ),
    (
        "Every ability the Royal Engineers have, with its section",
        "SELECT VALUE { action: out.name, allowed: allowed, section: rule.section.num } FROM capability WHERE in IN fn::precedence(unit:royal_engineers) ORDER BY action;",
    ),
    (
        "How a ZOC crosses each kind of boundary",
        "SELECT boundary.name AS boundary, out_of, into FROM zoc_boundary;",
    ),
    (
        "Which questions does the benchmark ask about §7.5 (retreat)?",
        "SELECT text, answer FROM question WHERE section = section:`7.5`;",
    ),
    (
        "How many records of each kind make up the archive?",
        "RETURN object::keys((INFO FOR DB).tables).map(|$t| { table: $t, records: count((SELECT id FROM type::table($t))) });",
    ),
];

#[component]
pub fn Notebook(initial: Option<String>) -> impl IntoView {
    let kb = expect_context::<Kb>();
    let src = RwSignal::new(initial.unwrap_or_else(|| GALLERY[0].1.to_string()));
    let result: RwSignal<Option<QueryResult>> = RwSignal::new(None);
    let busy = RwSignal::new(false);
    let run = move || {
        let q = src.get_untracked();
        replace(&Route::Notebook(Some(q.clone())));
        busy.set(true);
        spawn_local(async move {
            let r = kb.query(q).await;
            result.set(Some(r));
            busy.set(false);
        });
    };
    Effect::new(move |_| {
        if kb.ready.get() {
            run();
        }
    });
    view! {
        <section class="page-head">
            <h1>"The Notebook"</h1>
            <p class="lede">"Ask the archive anything in SurrealQL. It is read-only: writes change nothing. The schema is on the "
                <a href=href(&Route::About)>"about"</a>" page; the rule functions ("<code>"fn::can"</code>", "<code>"fn::stack_check"</code>", "
                <code>"fn::fire_drm"</code>", …) answer rulings."</p>
        </section>
        <div class="notebook">
            <div class="editor">
                <textarea spellcheck="false" prop:value=move || src.get() on:input=move |ev| src.set(event_target_value(&ev))
                    on:keydown=move |ev: web_sys::KeyboardEvent| { if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) { ev.prevent_default(); run(); } }></textarea>
                <div class="q-actions">
                    <button on:click=move |_| run() disabled=move || busy.get()>{move || if busy.get() { "running…" } else { "run (Ctrl+Enter)" }}</button>
                </div>
                <div class="results">{move || result.get().map(|r| {
                    let ms = r.ms;
                    let err = r.error.clone();
                    let n = r.statements.len();
                    view! {
                        <p class="small">{format!("{n} statement(s) · {ms:.1} ms")}</p>
                        {err.map(|e| view! { <p class="error">{e}</p> })}
                        {r.statements.into_iter().enumerate().map(|(i, s)| view! {
                            <div class="statement">
                                {(n > 1).then(|| view! { <p class="small">{format!("statement {}", i + 1)}</p> })}
                                {match s.error { Some(e) => view! { <p class="error">{e}</p> }.into_any(), None => value_view(&s.result) }}
                            </div>
                        }).collect_view()}
                    }
                })}</div>
            </div>
            <aside class="gallery">
                <h3>"Questions to ask"</h3>
                <ul>{GALLERY.iter().map(|(q, sql)| view! {
                    <li><button class="link" on:click=move |_| { src.set(sql.to_string()); run(); }>{*q}</button></li>
                }).collect_view()}</ul>
                <p class="small">"Hundreds more, each with its query, in "<a href=href(&Route::Inquiries(None))>"Inquiries"</a>"."</p>
            </aside>
        </div>
    }
}
