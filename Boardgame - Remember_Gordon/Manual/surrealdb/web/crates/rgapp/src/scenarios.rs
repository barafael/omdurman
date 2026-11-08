//! The scenarios: the campaign game, the historical scenario and the fall of Khartoum — set-up,
//! arrivals, special rules, victory — each with a calculator that asks the rule functions.

use leptos::prelude::*;
use serde_json::Value;

use crate::router::{Route, href};
use crate::ui::{list, query_chip, sec_link, show, strings, text, use_query, value_view};

const LIST: &str = "SELECT id, name, start, end, length_turns, length_text, sets_up_first.name AS sets_up_first, moves_first.name AS moves_first,
  maps.name AS maps, character, (SELECT VALUE [date, text] FROM historical_event WHERE scenario = $parent.id)[0] AS event,
  section.num AS section FROM scenario ORDER BY section;";

fn detail_sql(s: &str) -> String {
    format!("LET $s = {s};
RETURN {{
  setups: (SELECT faction.name AS side, not_in_play, subjects.{{id, name}} AS who, quantity, quantity_text, subject_text, represents, constraint, section.num AS section FROM setup WHERE scenario = $s ORDER BY side, section),
  arrivals: (SELECT faction.name AS side, turn, turn_window, text, entry_text, counts.{{n, represents}} AS counts, section.num AS section FROM reinforcement WHERE scenario = $s ORDER BY side, turn),
  rules: (SELECT section.num AS section, statement FROM rule WHERE scenarios CONTAINS $s ORDER BY section),
  awards: (SELECT (beneficiary.name ?? beneficiary) AS to, points, event, section.num AS section FROM vp_award WHERE scenario = $s ORDER BY points DESC),
  levels: (SELECT faction.name AS side, level, rank, scale, min, max, condition, basis FROM victory_level WHERE scenario = $s ORDER BY side, rank DESC, scale),
  conditions: (SELECT faction.name AS side, kind, result, condition, award_word, section.num AS section FROM victory_condition WHERE scenario = $s)
}};")
}

#[component]
pub fn Scenarios(selected: Option<String>) -> impl IntoView {
    let all = use_query(|| LIST.into());
    let sel = selected
        .clone()
        .unwrap_or_else(|| "scenario:campaign".into());
    let s2 = sel.clone();
    let detail = use_query(move || detail_sql(&s2));
    let s3 = sel.clone();
    view! {
        <section class="page-head">
            <h1>"The Scenarios"</h1>
            <p class="lede">"Three ways to fight it. The rulebook suggests starting with Khartoum, then the historical morning,
               and only then the full campaign (§1.1)."</p>
        </section>
        {let sel = sel.clone(); show(all, move |v| view! {
            <div class="folders">{list(&v).into_iter().map(|s| {
                let id = text(&s["id"]);
                let active = id == sel;
                let ev = list(&s["event"]);
                view! {
                    <a class="folder" class:active=active href=href(&Route::Scenarios(Some(id.clone())))>
                        <p class="kicker">{text(ev.first().unwrap_or(&Value::Null))}</p>
                        <h3>{text(&s["name"])}</h3>
                        <p class="small">{if text(&s["length_turns"]).is_empty() { text(&s["length_text"]) } else { format!("{} turns · {} – {}", text(&s["length_turns"]), text(&s["start"]), text(&s["end"])) }}</p>
                        <p class="small">{format!("{} sets up first; {} moves first · {}", text(&s["sets_up_first"]), text(&s["moves_first"]), strings(&s["maps"]).join(", "))}</p>
                        {(!text(&s["character"]).is_empty()).then(|| view! { <p class="quote">{text(&s["character"])}</p> })}
                    </a>
                }
            }).collect_view()}</div>
        })}
        <Calculator scenario=s3.clone() />
        {show(detail, |d| view! {
            <div class="scenario-detail">
                <div>
                    <h2 class="rule">"Set-up"</h2>
                    {list(&d["setups"]).into_iter().map(|s| {
                        let out = s["not_in_play"] == Value::Bool(true);
                        let qty = s["quantity"].as_i64().map(|n| format!("{n} × ")).unwrap_or_default();
                        view! {
                            <div class="setup" class:out=out>
                                <span class="side-tag">{text(&s["side"])}</span>
                                {if out { "Not in play: " } else { "" }}
                                {qty}{(!text(&s["subject_text"]).is_empty()).then(|| format!("{}: ", text(&s["subject_text"])))}
                                {value_view(&s["who"])}
                                {(!text(&s["quantity_text"]).is_empty()).then(|| view! { <span class="small">{format!(" ({})", text(&s["quantity_text"]))}</span> })}
                                {(!text(&s["represents"]).is_empty()).then(|| view! { <span class="small">{format!(" — represents {}", text(&s["represents"]))}</span> })}
                                {(!text(&s["constraint"]).is_empty()).then(|| view! { <p class="where">{text(&s["constraint"])}</p> })}
                                " "{sec_link(&text(&s["section"]))}
                            </div>
                        }
                    }).collect_view()}
                    <h2 class="rule">"Arrivals"</h2>
                    {if list(&d["arrivals"]).is_empty() { view! { <p class="small">"No reinforcements: everything starts on the map."</p> }.into_any() } else { view! {
                        <ul class="arrivals">{list(&d["arrivals"]).into_iter().map(|a| {
                            let when = if !text(&a["turn"]).is_empty() { format!("Turn {}", text(&a["turn"])) }
                                else if !list(&a["turn_window"]).is_empty() { format!("Turns {}", strings(&a["turn_window"]).join(", ")) } else { "Entry".into() };
                            view! { <li><span class="side-tag">{text(&a["side"])}</span><strong>{when}</strong>" "{text(&a["text"])}
                                {(!text(&a["entry_text"]).is_empty()).then(|| view! { <span class="small">{format!(" · enters {}", text(&a["entry_text"]))}</span> })}
                                " "{sec_link(&text(&a["section"]))}</li> }
                        }).collect_view()}</ul>
                    }.into_any() }}
                </div>
                <div>
                    <h2 class="rule">"Special rules"</h2>
                    <ul class="rule-list">{list(&d["rules"]).into_iter().map(|r| view! { <li>{sec_link(&text(&r["section"]))}" "{text(&r["statement"])}</li> }).collect_view()}</ul>
                    <h2 class="rule">"Victory"</h2>
                    {(!list(&d["awards"]).is_empty()).then(|| view! {
                        <table class="result"><thead><tr><th>"points"</th><th>"to"</th><th>"for"</th></tr></thead>
                        <tbody>{list(&d["awards"]).into_iter().map(|a| view! { <tr><td class="num">{text(&a["points"])}</td><td>{text(&a["to"])}</td><td>{text(&a["event"])}</td></tr> }).collect_view()}</tbody></table>
                    })}
                    <table class="result levels"><thead><tr><th>"side"</th><th>"level"</th><th>"needs"</th></tr></thead>
                    <tbody>{list(&d["levels"]).into_iter().map(|l| {
                        let needs = if !text(&l["condition"]).is_empty() { text(&l["condition"]) }
                            else if l["max"].is_null() { format!("{}+ {}", text(&l["min"]), text(&l["basis"])) }
                            else { format!("{}–{} {}", text(&l["min"]), text(&l["max"]), text(&l["basis"])) };
                        view! { <tr><td>{text(&l["side"])}</td><td>{text(&l["level"])}</td><td>{needs}</td></tr> }
                    }).collect_view()}</tbody></table>
                    <ul class="rule-list">{list(&d["conditions"]).into_iter().map(|c| view! {
                        <li>{sec_link(&text(&c["section"]))}" "<strong>{[text(&c["side"]), text(&c["result"])].join(" ")}</strong>" "{text(&c["condition"])}
                            {(!text(&c["award_word"]).is_empty()).then(|| view! { <span class="small">{format!(" (the rules say: \"{}\")", text(&c["award_word"]))}</span> })}</li>
                    }).collect_view()}</ul>
                </div>
            </div>
        })}
        {query_chip(move || detail_sql(&sel))}
    }
}

/// The scenario's victory arithmetic, live from the rule functions.
#[component]
fn Calculator(scenario: String) -> impl IntoView {
    let a = RwSignal::new(40i64);
    let b = RwSignal::new(12i64);
    let gordon = RwSignal::new(5i64); // 0 = survives turn 8
    let sc = scenario.clone();
    let sql = Memo::new(move |_| match sc.as_str() {
        "scenario:historical" => format!("RETURN fn::historical_result({}, {});", a.get(), b.get()),
        "scenario:fall_of_khartoum" => format!(
            "RETURN fn::fok_result({}, {});",
            if gordon.get() == 0 {
                "NONE".into()
            } else {
                gordon.get().to_string()
            },
            b.get()
        ),
        _ => format!("RETURN fn::campaign_result({}, {});", a.get(), b.get()),
    });
    let res = use_query(move || sql.get());
    let num = move |label: &'static str, sig: RwSignal<i64>, max: i64| {
        view! {
            <label class="num-in">{label}
                <input type="range" min="0" max=max prop:value=move || sig.get().to_string() on:input=move |ev| sig.set(event_target_value(&ev).parse().unwrap_or(0))/>
                <output>{move || sig.get()}</output>
            </label>
        }
    };
    let inputs = match scenario.as_str() {
        "scenario:historical" => view! { {num("Dervish units eliminated", a, 140)}{num("Anglo-Egyptian units eliminated", b, 40)} }.into_any(),
        "scenario:fall_of_khartoum" => view! {
            <label class="num-in">"Gordon eliminated on turn"
                <select on:change=move |ev| gordon.set(event_target_value(&ev).parse().unwrap_or(0))>
                    {(1..=8).map(|t| view! { <option value=t.to_string() selected=t == 5>{t}</option> }).collect_view()}
                    <option value="0">"— survives turn 8"</option>
                </select>
            </label>
            {num("Dervish units lost", b, 48)}
        }.into_any(),
        _ => view! { {num("Anglo-Egyptian victory points", a, 120)}{num("Dervish victory points", b, 120)} }.into_any(),
    };
    view! {
        <section class="calculator">
            <h2 class="rule">"Who has won?"</h2>
            <div class="calc-grid">
                <div class="calc-inputs">{inputs}</div>
                <div class="verdict">{move || match res.get() {
                    Some(Ok(v)) => verdict(&v),
                    Some(Err(e)) if !e.is_empty() => view! { <p class="error">{e}</p> }.into_any(),
                    _ => view! { <p class="loading">"…"</p> }.into_any(),
                }}</div>
            </div>
            <pre class="sql">{move || sql.get()}</pre>
        </section>
    }
}

fn verdict(v: &Value) -> AnyView {
    let headline = if let Some(r) = v.get("result") {
        text(r)
    } else if let Some(l) = v.get("net_level") {
        format!("{} {}", text(&v["winner"]), text(l))
            .trim()
            .to_string()
    } else {
        format!("{} {}", text(&v["winner"]), text(&v["level"]))
            .trim()
            .to_string()
    };
    let headline = if headline.is_empty() {
        "No band of the victory table covers this".to_string()
    } else {
        headline
    };
    view! { <p class="headline">{headline}</p>{value_view(v)} }.into_any()
}
