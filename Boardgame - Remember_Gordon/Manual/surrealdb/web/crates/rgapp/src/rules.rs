//! The rulebook, section by section: the manual's own words, what each section cites and is cited by,
//! the atomic rules the archive draws from it, what it leaves open, and what players have asked about
//! it. And the dispatches: everything the manual leaves unsettled.

use leptos::prelude::*;
use serde_json::Value;

use crate::router::{Route, href, key_of};
use crate::ui::{list, manual_text, query_chip, sec_link, show, strings, text, use_query};

const TOC: &str = "SELECT num, title, depth, kind, text FROM section ORDER BY ord;";

fn section_sql(num: &str) -> String {
    format!("LET $x = section:`{num}`;
RETURN {{
  num: $x.num, title: $x.title, text: $x.text, kind: $x.kind, parent: $x.parent.num, parent_title: $x.parent.title,
  children: (SELECT num, title FROM section WHERE parent = $x ORDER BY ord),
  cites: $x->cites->section.num, cited_by: $x<-cites<-section.num,
  rules: (SELECT kind, statement, scenarios.name AS scenarios, ->governs->action.name AS actions,
          ->exception_to->rule.section.num AS exception_to, <-exception_to<-rule.section.num AS excepted_by FROM rule WHERE section = $x),
  open: (SELECT kind, question FROM open_question WHERE refs CONTAINS $x),
  asked: (SELECT id, text, kind FROM question WHERE section = $x ORDER BY kind DESC)
}};")
}

#[component]
pub fn Rulebook(selected: Option<String>) -> impl IntoView {
    let toc = use_query(|| TOC.into());
    let filter = RwSignal::new(String::new());
    let num = selected.clone().unwrap_or_else(|| "1.1".into());
    view! {
        <section class="page-head">
            <h1>"The Rulebook"</h1>
            <p class="lede">"The 1982 rules, as printed. Beside each section, the rules the archive draws out of it, the sections it
               points to and that point to it, and what it leaves unsaid."</p>
        </section>
        <div class="rulebook">
            <aside class="toc">
                <input class="search" type="search" placeholder="search the manual…" prop:value=move || filter.get() on:input=move |ev| filter.set(event_target_value(&ev))/>
                {let current = num.clone(); show(toc, move |v| {
                    let rows = list(&v);
                    let current = current.clone();
                    view! { <ul>{move || {
                        let f = filter.get().to_lowercase();
                        rows.iter().filter(|r| f.is_empty() || text(&r["text"]).to_lowercase().contains(&f) || text(&r["title"]).to_lowercase().contains(&f))
                            .map(|r| {
                                let n = text(&r["num"]);
                                let depth = r["depth"].as_i64().unwrap_or(0);
                                let label = format!("{} {}", n, title_or_opening(&r));
                                view! { <li class=format!("d{depth}") class:active=n == current><a href=href(&Route::Rules(Some(n.clone())))>{label}</a></li> }
                            }).collect_view()
                    }}</ul> }
                })}
            </aside>
            <Section num=num />
        </div>
    }
}

#[component]
fn Section(num: String) -> impl IntoView {
    let sql = section_sql(&num);
    let s2 = sql.clone();
    let res = use_query(move || sql.clone());
    view! {
        <article class="section-view">
            {show(res, |s| {
                if s["num"].is_null() {
                    return view! { <p class="error">"No such section."</p> }.into_any();
                }
                let chips = |label: &'static str, v: &Value| {
                    let nums = strings(v);
                    (!nums.is_empty()).then(|| view! { <p class="refs"><span class="small">{label}</span>{nums.into_iter().map(|n| view! { " "{sec_link(&n)} }).collect_view()}</p> })
                };
                view! {
                    <p class="kicker">{(!text(&s["parent"]).is_empty()).then(|| view! { {sec_link(&text(&s["parent"]))}{format!(" {}", text(&s["parent_title"]))} })}</p>
                    <h2>{format!("§{} {}", text(&s["num"]), title_or_opening(&s))}</h2>
                    <div class="page">{manual_text(&text(&s["text"]))}
                        {(!list(&s["children"]).is_empty()).then(|| view! {
                            <ul class="plain">{list(&s["children"]).into_iter().map(|c| view! { <li>{sec_link(&text(&c["num"]))}{format!(" {}", text(&c["title"]))}</li> }).collect_view()}</ul>
                        })}
                    </div>
                    {chips("cites", &s["cites"])}
                    {chips("cited by", &s["cited_by"])}
                    {(!list(&s["rules"]).is_empty()).then(|| view! {
                        <h3>"Rules drawn from it"</h3>
                        <ul class="rules">{list(&s["rules"]).into_iter().map(|r| view! {
                            <li>
                                <span class=format!("kind k-{}", text(&r["kind"]))>{text(&r["kind"])}</span>
                                {text(&r["statement"])}
                                <div class="small">
                                    {(!strings(&r["actions"]).is_empty()).then(|| format!("governs: {} ", strings(&r["actions"]).join(", ")))}
                                    {(!strings(&r["scenarios"]).is_empty()).then(|| format!("· only in: {} ", strings(&r["scenarios"]).join(", ")))}
                                    {(!strings(&r["exception_to"]).is_empty()).then(|| view! { "· an exception to "{strings(&r["exception_to"]).into_iter().map(|n| view! { {sec_link(&n)}" " }).collect_view()} })}
                                    {(!strings(&r["excepted_by"]).is_empty()).then(|| view! { "· with exceptions in "{strings(&r["excepted_by"]).into_iter().map(|n| view! { {sec_link(&n)}" " }).collect_view()} })}
                                </div>
                            </li>
                        }).collect_view()}</ul>
                    })}
                    {(!list(&s["open"]).is_empty()).then(|| view! {
                        <h3>"Left open"</h3>
                        <ul class="dispatch-list">{list(&s["open"]).into_iter().map(|o| view! { <li><span class=format!("kind k-{}", text(&o["kind"]))>{text(&o["kind"])}</span>{text(&o["question"])}</li> }).collect_view()}</ul>
                    })}
                    {(!list(&s["asked"]).is_empty()).then(|| view! {
                        <h3>"Asked at the table"</h3>
                        <ul class="asked">{list(&s["asked"]).into_iter().map(|q| view! {
                            <li><a href=href(&Route::Inquiries(Some(key_of(&text(&q["id"])))))>{text(&q["text"])}</a></li>
                        }).collect_view()}</ul>
                    })}
                }.into_any()
            })}
            {query_chip(move || s2.clone())}
        </article>
    }
}

/// A section's title, or for the untitled numbered paragraphs the first words of their text.
fn title_or_opening(s: &Value) -> String {
    let t = text(&s["title"]);
    if !t.is_empty() {
        return t;
    }
    let body = text(&s["text"]);
    let words: Vec<&str> = body.split_whitespace().take(6).collect();
    if words.is_empty() {
        String::new()
    } else {
        format!(
            "{}…",
            words.join(" ").trim_end_matches([',', '.', ';', ':'])
        )
    }
}

const DISPATCHES: &str = "SELECT id, kind, question, reading, ->raises->section.num AS sections FROM open_question ORDER BY kind, id;";

#[component]
pub fn Dispatches() -> impl IntoView {
    let res = use_query(|| DISPATCHES.into());
    let kind = RwSignal::new(String::new());
    view! {
        <section class="page-head">
            <h1>"Dispatches"</h1>
            <p class="lede">"What the rulebook does not settle. Rather than guess, the archive records each of these, and its rule
               functions answer “undetermined” where one applies. A "<em>"gap"</em>" is a question the rules never answer, an "
               <em>"ambiguity"</em>" one they answer two ways, an "<em>"inconsistency"</em>" a place where the manual contradicts itself."</p>
            <p class="small">"In 1982 such questions went to Phoenix Enterprises, phrased for a yes or no, with a stamped, self-addressed envelope."</p>
        </section>
        <div class="toggle">
            {[("", "all"), ("gap", "gaps"), ("ambiguity", "ambiguities"), ("inconsistency", "inconsistencies")].into_iter().map(|(k, l)| view! {
                <button class:active=move || kind.get() == k on:click=move |_| kind.set(k.into())>{l}</button>
            }).collect_view()}
        </div>
        {show(res, move |v| {
            let rows = list(&v);
            view! { <div class="dispatches">{move || {
                let k = kind.get();
                rows.iter().filter(|r| k.is_empty() || text(&r["kind"]) == k).map(|r| view! {
                    <div class="dispatch">
                        <span class=format!("kind k-{}", text(&r["kind"]))>{text(&r["kind"])}</span>
                        <p class="q">{text(&r["question"])}</p>
                        {(!text(&r["reading"]).is_empty()).then(|| view! { <p class="small">{text(&r["reading"])}</p> })}
                        <p class="refs">{strings(&r["sections"]).into_iter().map(|n| view! { {sec_link(&n)}" " }).collect_view()}</p>
                    </div>
                }).collect_view()
            }}</div> }
        })}
        {query_chip(|| DISPATCHES.into())}
    }
}
