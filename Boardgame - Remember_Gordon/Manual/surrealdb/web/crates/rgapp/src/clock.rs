//! The clock: the Turn Record Track, laid out as the printed track runs (left to right, back, and
//! again), with the night boxes shaded; and the sequence of play for each scenario.

use leptos::prelude::*;
use serde_json::Value;

use crate::ui::{list, query_chip, sec_link, show, strings, text, use_query};

const TRACK: &str = "SELECT n, date, label, time, night, special FROM turn ORDER BY n;";
const NIGHT: &str = "SELECT section.num AS section, statement FROM rule WHERE concerns CONTAINS state:night ORDER BY section;";

fn sequence_sql(scenario: &str) -> String {
    format!("LET $s = {scenario};
LET $order = IF $s.moves_first = faction:anglo_egyptian {{ [phase:ae_player_turn, phase:d_player_turn] }} ELSE {{ [phase:d_player_turn, phase:ae_player_turn] }};
RETURN $order.map(|$pt| {{ turn: $pt.name, side: $pt.player_turn, steps: (SELECT name, step, actor.name AS actor, actor AS side, fire_mode, <-allowed_in<-action.name AS actions FROM phase WHERE player_turn_phase = $pt ORDER BY step) }});")
}

/// Rows of the printed track: the boxes snake left to right, right to left, left to right.
const ROWS: [&[i64]; 3] = [
    &[1, 2, 3, 4, 5, 6, 7, 8],
    &[16, 15, 14, 13, 12, 11, 10, 9],
    &[17, 18, 19, 20, 21, 22],
];

#[component]
pub fn Clock() -> impl IntoView {
    let track = use_query(|| TRACK.into());
    let night_rules = use_query(|| NIGHT.into());
    let selected = RwSignal::new(9i64);
    let scenario = RwSignal::new("scenario:campaign".to_string());
    let seq = use_query(move || sequence_sql(&scenario.get()));
    view! {
        <section class="page-head">
            <h1>"The Clock"</h1>
            <p class="lede">"Each day turn is two hours. The campaign runs twenty-two turns, from 6 a.m. on 1 September to 8 a.m.
               on the 3rd, through two nights; on the first, some of the Khalifa's men slip away."</p>
        </section>
        <section>
            <h2 class="rule">"The Turn Record Track"</h2>
            {show(track, move |v| {
                let turns = list(&v);
                let get = move |n: i64| turns.iter().find(|t| t["n"].as_i64() == Some(n)).cloned().unwrap_or(Value::Null);
                view! {
                    <div class="track">{ROWS.iter().enumerate().map(|(i, row)| {
                        let g = get.clone();
                        view! {
                            <div class="track-row" class:reverse=i == 1>{row.iter().map(|n| {
                                let t = g(*n);
                                let n = *n;
                                let night = t["night"] == Value::Bool(true);
                                view! {
                                    <button class="box" class:night=night class:sel=move || selected.get() == n on:click=move |_| selected.set(n)>
                                        <span class="n">{n}</span>
                                        <span class="lbl">{text(&t["label"])}</span>
                                    </button>
                                }
                            }).collect_view()}</div>
                        }
                    }).collect_view()}</div>
                    {move || { let t = get(selected.get()); let night = t["night"] == Value::Bool(true); view! {
                        <div class="turn-card" class:night=night>
                            <h3>{format!("Turn {} · {} {}", selected.get(), text(&t["date"]), if night { "· night".to_string() } else { text(&t["time"]) })}</h3>
                            {(!text(&t["special"]).is_empty()).then(|| view! { <p class="special">{format!("{} — once per campaign, in the Dervish movement phase: 1½ × a d10 Dervish units desert (§8.2).", text(&t["special"]))}</p> })}
                            {if night { view! { <p>"Night: Anglo-Egyptian movement halved, no howitzer fire, all ranges halved (§8.1)."</p> }.into_any() }
                             else { view! { <p>"Day turn: two hours of real time (§1.2)."</p> }.into_any() }}
                        </div>
                    } }}
                }
            })}
            <p class="small">"The printed track carries no turn numbers; they are numbered here in the order of its arrows. Rows of eight are this page's layout."</p>
            {query_chip(|| TRACK.into())}
            <details>
                <summary>"Every rule about the night"</summary>
                {show(night_rules, |v| view! { <ul class="rule-list">{list(&v).into_iter().map(|r| view! { <li>{sec_link(&text(&r["section"]))}" "{text(&r["statement"])}</li> }).collect_view()}</ul> })}
            </details>
        </section>
        <section>
            <h2 class="rule">"The sequence of play"</h2>
            <div class="toggle">
                {[("scenario:campaign", "Campaign game"), ("scenario:historical", "Historical scenario"), ("scenario:fall_of_khartoum", "Fall of Khartoum")].into_iter().map(|(id, l)| view! {
                    <button class:active=move || scenario.get() == id on:click=move |_| scenario.set(id.into())>{l}</button>
                }).collect_view()}
            </div>
            {show(seq, |v| view! {
                <div class="sequence">{list(&v).into_iter().map(|pt| {
                    let side = if text(&pt["side"]).contains("anglo") { "ae" } else { "dv" };
                    view! {
                        <div class=format!("player-turn {side}")>
                            <h3>{text(&pt["turn"])}</h3>
                            <ol>{list(&pt["steps"]).into_iter().map(|s| {
                                let actor = if text(&s["side"]).contains("anglo") { "ae" } else { "dv" };
                                view! {
                                    <li class=format!("step {actor}")>
                                        <span class="step-name">{text(&s["name"])}</span>
                                        <span class="small">{format!("{}{}", text(&s["actor"]), if text(&s["fire_mode"]).is_empty() { String::new() } else { format!(" · {} fire", text(&s["fire_mode"])) })}</span>
                                        <span class="acts">{strings(&s["actions"]).join(" · ")}</span>
                                    </li>
                                }
                            }).collect_view()}</ol>
                        </div>
                    }
                }).collect_view()}</div>
            })}
            <p class="small">"Defensive fire comes before the moving player's offensive fire; only the Anglo-Egyptian side has the
               second subphase of Maxim and howitzer fire, on defence too (§4, §6.4). Then the Game Turn marker advances."</p>
            {query_chip(move || sequence_sql(&scenario.get()))}
        </section>
    }
}
