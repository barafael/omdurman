//! The prologue: the battle, the two seats at the table, the road to Omdurman, the people, and the
//! ways into the archive.

use leptos::prelude::*;
use serde_json::Value;

use crate::router::{Route, href};
use crate::ui::{list, manual_text, query_chip, rid_link, show, sql_block, text, use_query};

const TIMELINE: &str = "SELECT id, date, text, scenario, scenario.name AS scenario_name FROM historical_event ORDER BY date;";
const PEOPLE: &str = "SELECT id, name, known_as, role, counter FROM person WHERE role != NONE AND id != person:peter_bertram ORDER BY name;";
const SEATS: &str = "SELECT id, name, player_role, seat, (SELECT VALUE name FROM unit WHERE faction = $parent.id AND class.name CONTAINS 'leader') AS leaders FROM faction ORDER BY name DESC;";

/// Ways in: each a view, a question a player might ask there, and the query that answers it.
const WAYS: [(&str, Route, &str, &str, &str); 6] = [
    (
        "The Armies",
        Route::Armies(None),
        "Every counter of both armies, and what each may do.",
        "Can Osman Digna stack with the Baggara?",
        "RETURN fn::stack_check([unit:osman_digna, tribe:baggara]);",
    ),
    (
        "The Clock",
        Route::Clock,
        "Twenty-two turns from dawn on 1 September, nights included.",
        "Which turns are fought at night?",
        "SELECT VALUE n FROM turn WHERE night ORDER BY n;",
    ),
    (
        "The Scenarios",
        Route::Scenarios(None),
        "The campaign, the historical morning, and the fall of Khartoum.",
        "Gordon falls on turn 5, at the cost of 24 Dervish units. Who has won?",
        "RETURN fn::fok_result(5, 24).result;",
    ),
    (
        "The War Office",
        Route::Office,
        "Roll the howitzers, check a stack, total a fire attack.",
        "What does a British brigade in integrity add to its die?",
        "RETURN fn::fire_drm({ firer: faction:anglo_egyptian, direct: true, integrity: true });",
    ),
    (
        "The Rulebook",
        Route::Rules(None),
        "The manual, section by section, with the rules drawn out of it.",
        "Which rules are exceptions to the stacking limit?",
        "SELECT section.num AS section, statement FROM rule:r5_51_limit<-exception_to<-rule;",
    ),
    (
        "Inquiries",
        Route::Inquiries(None),
        "748 questions from the table, each answered by a query.",
        "What may a disrupted unit not do?",
        "RETURN state:disrupted->prohibits->action.name;",
    ),
];

#[component]
pub fn Home() -> impl IntoView {
    let intro = use_query(|| "RETURN section:`1.1`.text;".into());
    let quotes = use_query(|| "RETURN person:victoria.quotes;".into());
    let timeline = use_query(|| TIMELINE.into());
    let people = use_query(|| PEOPLE.into());
    let seats = use_query(|| SEATS.into());
    view! {
        <section class="hero">
            <p class="kicker">"Sudan · 1 – 3 September 1898"</p>
            <h1>"Remember Gordon!"</h1>
            <p class="dek">"The Battle of Omdurman, as the 1982 board game tells it: the Khalifa's sixty thousand on the
               plain of Kerreri, Kitchener's army behind its zariba with its back to the Nile, the gunboats, the night,
               the Mahdi's Tomb."</p>
            <div class="telegrams">
                {show(quotes, |v| {
                    let q = list(&v);
                    view! {
                        <div class="telegram"><span class="stamp">"1885"</span>{format!("“{}”", text(q.first().unwrap_or(&Value::Null)).to_uppercase())}<span class="sig">"— Queen Victoria, on the fall of Khartoum"</span></div>
                        <div class="telegram"><span class="stamp">"1898"</span>{format!("“{}”", text(q.get(1).unwrap_or(&Value::Null)).to_uppercase())}<span class="sig">"— Queen Victoria, after Omdurman"</span></div>
                    }
                })}
            </div>
        </section>

        <section class="band">
            <div class="manual-quote">
                <h2>"From the rulebook"</h2>
                {show(intro, |v| { let t = text(&v); let first = t.split("\n\n").next().unwrap_or_default().to_string(); manual_text(&first) })}
                <p class="cite"><a href=href(&Route::Rules(Some("1.1".into())))>"§1.1 General Comments →"</a></p>
            </div>
        </section>

        <section>
            <h2 class="rule">"Two seats at the table"</h2>
            {show(seats, |v| view! {
                <div class="seats">{list(&v).into_iter().map(|f| {
                    let id = text(&f["id"]);
                    let side = if id.contains("anglo") { "ae" } else { "dv" };
                    view! {
                        <a class=format!("seat {side}") href=href(&Route::Armies(None))>
                            <h3>{text(&f["name"])}</h3>
                            <p class="role">{text(&f["player_role"])}</p>
                            <p class="small">{format!("Sits at the {} of the map.", text(&f["seat"]))}</p>
                            <p class="small">"Leaders: "{list(&f["leaders"]).iter().map(text).collect::<Vec<_>>().join(", ")}</p>
                        </a>
                    }
                }).collect_view()}</div>
            })}
            {query_chip(|| SEATS.into())}
        </section>

        <section>
            <h2 class="rule">"The road to Omdurman"</h2>
            <p class="lede">"Seventeen years from the Mahdi's rising to Kitchener's memorial service, from the historical notes
               at the back of the rulebook. Three of these days are games."</p>
            {show(timeline, |v| view! {
                <ol class="timeline">{list(&v).into_iter().map(|e| {
                    let scen = text(&e["scenario"]);
                    let played = !scen.is_empty();
                    view! {
                        <li class:played=played>
                            <span class="date">{text(&e["date"])}</span>
                            <span class="what">{text(&e["text"])}</span>
                            {played.then(|| view! { <a class="play" href=href(&Route::Scenarios(Some(scen.clone())))>{format!("play it: {}", text(&e["scenario_name"]))}</a> })}
                        </li>
                    }
                }).collect_view()}</ol>
            })}
            {query_chip(|| TIMELINE.into())}
        </section>

        <section>
            <h2 class="rule">"Dramatis personae"</h2>
            {show(people, |v| view! {
                <div class="people">{list(&v).into_iter().map(|p| {
                    let counter = text(&p["counter"]);
                    view! {
                        <div class="person">
                            <h4>{text(&p["name"])}</h4>
                            {(!text(&p["known_as"]).is_empty()).then(|| view! { <p class="aka">{text(&p["known_as"])}</p> })}
                            <p>{text(&p["role"])}</p>
                            {(!counter.is_empty()).then(|| view! { <p class="small">"In the game: "{rid_link(&counter, Some("his counter →".into()))}</p> })}
                        </div>
                    }
                }).collect_view()}</div>
            })}
            {query_chip(|| PEOPLE.into())}
        </section>

        <section>
            <h2 class="rule">"Ways into the archive"</h2>
            <p class="lede">"Every page here is built from queries against the rulebook graph, and each shows its query.
               A few to start with:"</p>
            <div class="ways">{WAYS.iter().map(|(title, route, blurb, question, sql)| view! {
                <div class="way">
                    <a class="way-title" href=href(route)>{*title}</a>
                    <p>{*blurb}</p>
                    <p class="q">{format!("“{question}”")}</p>
                    {sql_block(sql)}
                    <a class="small" href=href(&Route::Notebook(Some(sql.to_string())))>"ask it in the notebook →"</a>
                </div>
            }).collect_view()}</div>
        </section>
    }
}
