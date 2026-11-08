//! Inquiries: the 748 questions of the benchmark, as a player at the table would ask them. Each is
//! answered by its SurrealQL query, run here and now against the archive, and checked against the
//! answer the manual supports.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::Value;

use crate::kb::Kb;
use crate::router::{Route, go, href};
use crate::ui::{list, sec_link, show, text, use_query, value_view};

const QUESTIONS: &str = "SELECT record::id(id) AS key, section_num AS section, kind, text, answer, query, expect, match, note FROM question ORDER BY section, key;";
const PAGE: usize = 30;

const KINDS: [(&str, &str, &str); 5] = [
    ("", "all", ""),
    (
        "player",
        "from the table",
        "the questions first drawn from the rulebook, as a player mid-game would ask them",
    ),
    ("basic", "plain", "ordinary lookups and rulings"),
    (
        "graph",
        "relations",
        "questions that walk the graph: exceptions, citations, commands, chains",
    ),
    (
        "open",
        "unsettled",
        "where the manual is silent; the query shows the archive knows it",
    ),
];

#[component]
pub fn Inquiries(selected: Option<String>) -> impl IntoView {
    let res = use_query(|| QUESTIONS.into());
    let kind = RwSignal::new(String::new());
    let search = RwSignal::new(String::new());
    let shown = RwSignal::new(PAGE);
    let sel = selected.clone();
    view! {
        <section class="page-head">
            <h1>"Inquiries"</h1>
            <p class="lede">"Seven hundred and forty-eight questions, from “how far is a hex?” to “can Osman Digna stack with the
               Baggara?”. Each one is a SurrealQL query with the answer the manual supports; open one and run it against the archive."</p>
        </section>
        {show(res, move |v| {
            let all = list(&v);
            let total = all.len();
            let pinned = sel.clone().and_then(|k| all.iter().find(|q| text(&q["key"]) == k).cloned());
            let all2 = all.clone();
            view! {
                {pinned.map(|q| view! { <div class="pinned">{card(q, true)}</div> })}
                <div class="filters">
                    <div class="toggle">{KINDS.iter().map(|(k, l, t)| view! {
                        <button class:active=move || kind.get() == *k title=*t on:click=move |_| { kind.set(k.to_string()); shown.set(PAGE); }>{*l}</button>
                    }).collect_view()}</div>
                    <input class="search" type="search" placeholder="search questions and answers…" prop:value=move || search.get()
                        on:input=move |ev| { search.set(event_target_value(&ev)); shown.set(PAGE); }/>
                    <button on:click=move |_| {
                        let i = (js_sys::Math::random() * all2.len() as f64) as usize;
                        if let Some(q) = all2.get(i) { go(Route::Inquiries(Some(text(&q["key"])))); }
                    }>"ask one at random"</button>
                </div>
                {move || {
                    let k = kind.get();
                    let s = search.get().to_lowercase();
                    let hits: Vec<Value> = all.iter().filter(|q| (k.is_empty() || text(&q["kind"]) == k)
                        && (s.is_empty() || text(&q["text"]).to_lowercase().contains(&s) || text(&q["answer"]).to_lowercase().contains(&s) || text(&q["section"]) == s))
                        .cloned().collect();
                    let n = hits.len();
                    let more = n > shown.get();
                    view! {
                        <p class="small">{format!("{n} of {total} questions")}</p>
                        <div class="questions">{hits.into_iter().take(shown.get()).map(|q| card(q, false)).collect_view()}</div>
                        {more.then(|| view! { <button class="more" on:click=move |_| shown.update(|s| *s += PAGE)>"more questions"</button> })}
                    }
                }}
            }
        })}
    }
}

/// One question: folded to its text, unfolded to its answer, its query and a live run.
fn card(q: Value, open: bool) -> impl IntoView {
    let kb = expect_context::<Kb>();
    let unfolded = RwSignal::new(open);
    let run: RwSignal<Option<Result<Value, String>>> = RwSignal::new(None);
    let query = text(&q["query"]);
    let expect = q["expect"].clone();
    let mode = {
        let m = text(&q["match"]);
        if m.is_empty() { "exact".to_string() } else { m }
    };
    let fire = {
        let query = query.clone();
        move || {
            let query = query.clone();
            spawn_local(async move {
                run.set(Some(kb.last(query).await));
            });
        }
    };
    if open {
        let f = fire.clone();
        Effect::new(move |_| {
            if kb.ready.get() {
                f();
            }
        });
    }
    let key = text(&q["key"]);
    let kind = text(&q["kind"]);
    view! {
        <div class="question" class:open=move || unfolded.get()>
            <button class="q-head" on:click=move |_| unfolded.update(|u| *u = !*u)>
                <span class=format!("kind k-{kind}")>{kind.clone()}</span>
                <span class="q-text">{text(&q["text"])}</span>
            </button>
            <Show when=move || unfolded.get()>
                {let (q, query, expect, mode, fire, key) = (q.clone(), query.clone(), expect.clone(), mode.clone(), fire.clone(), key.clone());
                 move || { let fire = fire.clone(); let expect = expect.clone(); let mode = mode.clone(); view! {
                    <div class="q-body">
                        <p class="answer-text">{text(&q["answer"])}" "{sec_link(&text(&q["section"]))}</p>
                        {(!text(&q["note"]).is_empty()).then(|| view! { <p class="small note">{text(&q["note"])}</p> })}
                        <pre class="sql">{query.clone()}</pre>
                        <div class="q-actions">
                            <button on:click=move |_| fire()>"run it"</button>
                            <a class="small" href=href(&Route::Notebook(Some(query.clone())))>"edit in the notebook"</a>
                            <a class="small" href=href(&Route::Inquiries(Some(key.clone())))>"link to this question"</a>
                        </div>
                        {move || run.get().map(|r| match r {
                            Ok(v) => {
                                let ok = rgcore::matches(&v, &expect, &mode);
                                view! {
                                    <div class="run">
                                        <p class=if ok { "verdict-yes" } else { "verdict-no" }>{if ok { "✓ the archive answers as the manual does" } else { "✗ the archive's answer differs from the expected one" }}</p>
                                        <div class="got">{value_view(&v)}</div>
                                        {(!ok).then(|| view! { <p class="small">"expected:"</p><div class="got">{value_view(&expect)}</div> })}
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! { <p class="error">{e}</p> }.into_any(),
                        })}
                    </div>
                 } }}
            </Show>
        </div>
    }
}
