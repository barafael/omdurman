//! The armies: both orders of battle as counters, and a dossier for any unit, tribe, class or command
//! — what it is, what it fires with, what it may and may not do (with the rule and its basis), where it
//! appears, and what the manual leaves open about it.

use leptos::prelude::*;
use serde_json::Value;

use crate::router::{Route, href};
use crate::ui::{
    counter, kind_of, list, query_chip, rid_link, sec_link, show, side_of, strings, text,
    tribe_color, use_query, value_view,
};

const OOB: &str = "RETURN {
  units: (SELECT id, name, faction, class AS kind, class.name AS kind_name, printed, count, generic, title FROM unit ORDER BY name),
  groups: (SELECT id, name, color, basis, individual_assignment, <-leads<-unit.{id, name} AS leaders, <-member_of<-tribe.{id, name, color, unit_class} AS tribes FROM command_group ORDER BY name),
  formations: (SELECT id, name, designation, kind, nationality FROM formation ORDER BY name)
};";

fn dossier_sql(rid: &str) -> String {
    format!(
        r#"LET $s = {rid};
LET $p = fn::precedence($s);
RETURN {{
  id: $s, table: record::tb($s), name: $s.name, faction: $s.faction.name ?? $s.faction, kind: $s.class ?? $s.unit_class ?? $s,
  printed: $s.printed, count: $s.count, title: $s.title, role: $s.role, history: $s.history, color: $s.color, aka: $s.aka,
  description: $s.description, note: $s.note, basis: $s.basis,
  classes: $p.filter(|$x| record::tb($x) = "unit_class").map(|$x| $x.name),
  lines: fn::lines($s).map(|$l| string::concat($l.name, " · ", $l.table.name)),
  abilities: (SELECT VALUE id FROM action ORDER BY name).map(|$a| {{
    LET $c = fn::can($s, $a);
    RETURN IF $c = NONE {{ NONE }} ELSE {{ {{ action: $a.name, allowed: $c.allowed, condition: $c.condition, basis: $c.basis, section: $c.section, note: $c.note }} }};
  }}).filter(|$x| $x != NONE),
  enters: (SELECT out.name AS place, allowed, condition, section.num AS section FROM entry WHERE in IN $p),
  command: IF record::tb($s) = "tribe" {{ $s->member_of->command_group<-leads<-unit.{{id, name}} }}
           ELSE IF record::tb($s) = "unit" {{ array::flatten($s->leads->command_group<-member_of<-tribe.{{id, name}}) }} ELSE {{ [] }},
  members: IF record::tb($s) = "command_group" {{ $s<-member_of<-tribe.{{id, name}} }} ELSE {{ [] }},
  scenarios: array::distinct($s->appears_in->scenario.{{id, name}}),
  rules: (SELECT section.num AS section, statement FROM rule WHERE concerns CONTAINSANY $p ORDER BY section),
  open: (SELECT id, question FROM open_question WHERE refs CONTAINSANY $p)
}};"#
    )
}

#[component]
pub fn Armies(selected: Option<String>) -> impl IntoView {
    let oob = use_query(|| OOB.into());
    let sel = selected.clone();
    view! {
        <section class="page-head">
            <h1>"The Armies"</h1>
            <p class="lede">"Kitchener's Anglo-Egyptian army arrives from the north along the Nile; the Khalifa's tribes gather on the
               plain west of it. Every counter the rules name is here. Choose one for its dossier."</p>
        </section>
        {sel.map(|rid| view! { <Dossier rid=rid /> })}
        {show(oob, |v| {
            let units = list(&v["units"]);
            let side = |s: &str| units.iter().filter(|u| side_of(&text(&u["faction"])) == s).cloned().collect::<Vec<_>>();
            let (ae, dv) = (side("ae"), side("dv"));
            let groups = list(&v["groups"]);
            let formations = list(&v["formations"]);
            view! {
                <div class="armies">
                    <div class="army ae">
                        <h2>"Anglo-Egyptian"</h2>
                        <h3>"Leaders"</h3>
                        <div class="counters">{ae.iter().filter(|u| kind_of(&text(&u["kind"])) == "leader").map(unit_counter).collect_view()}</div>
                        <h3>"Combat units"</h3>
                        <div class="counters">{ae.iter().filter(|u| kind_of(&text(&u["kind"])) != "leader").map(unit_counter).collect_view()}</div>
                        <h3>"Formations"</h3>
                        <ul class="plain">{formations.iter().map(|f| view! {
                            <li>{rid_link(&text(&f["id"]), Some(text(&f["name"])))}<span class="small">{format!(" {} · {}", text(&f["kind"]), text(&f["nationality"]))}</span></li>
                        }).collect_view()}</ul>
                        <p class="small">"Infantry fights in brigades of four battalions; all four in one hex have "
                            <a href=href(&Route::Rules(Some("5.54".into())))>"brigade integrity"</a>"."</p>
                    </div>
                    <div class="army dv">
                        <h2>"Dervish"</h2>
                        <h3>"Leaders and their commands"</h3>
                        {groups.iter().map(|g| {
                            let tribes = list(&g["tribes"]);
                            let leaders = list(&g["leaders"]);
                            let unstated = text(&g["individual_assignment"]) == "unstated";
                            view! {
                                <div class="command">
                                    <div class="command-head">
                                        {rid_link(&text(&g["id"]), Some(text(&g["name"])))}
                                        {(!text(&g["color"]).is_empty()).then(|| view! { <span class="swatch" title=text(&g["color"])></span> })}
                                    </div>
                                    <div class="counters">
                                        {leaders.iter().map(|l| view! { <a href=href(&Route::Armies(Some(text(&l["id"]))))>{counter(text(&l["name"]), "dv", "leader", String::new(), tribe_color(&text(&g["color"])), true)}</a> }).collect_view()}
                                        {tribes.iter().map(|t| view! { <a href=href(&Route::Armies(Some(text(&t["id"]))))>{counter(text(&t["name"]), "dv", kind_of(&text(&t["unit_class"])), String::new(), tribe_color(&text(&t["color"])), true)}</a> }).collect_view()}
                                    </div>
                                    {unstated.then(|| view! { <p class="small note">"The rules name these leaders and tribes together; which leader commands which tribe is not stated."</p> })}
                                </div>
                            }
                        }).collect_view()}
                        <h3>"Other units"</h3>
                        <div class="counters">{dv.iter().filter(|u| kind_of(&text(&u["kind"])) != "leader").map(unit_counter).collect_view()}</div>
                    </div>
                </div>
            }
        })}
        {query_chip(|| OOB.into())}
    }
}

fn unit_counter(u: &Value) -> impl IntoView + use<> {
    let name = match u["count"].as_i64() {
        Some(n) if n > 1 => format!("{} ×{n}", text(&u["name"])),
        _ => text(&u["name"]),
    };
    let rid = text(&u["id"]);
    view! {
        <a href=href(&Route::Armies(Some(rid))) title=text(&u["kind_name"])>
            {counter(name, side_of(&text(&u["faction"])), kind_of(&text(&u["kind"])), text(&u["printed"]), None, false)}
        </a>
    }
}

#[component]
fn Dossier(rid: String) -> impl IntoView {
    let sql = dossier_sql(&rid);
    let s2 = sql.clone();
    let res = use_query(move || sql.clone());
    view! {
        <section class="dossier">
            {show(res, |d| {
                let abilities = list(&d["abilities"]);
                let (may, maynot): (Vec<Value>, Vec<Value>) = abilities.into_iter().partition(|a| a["allowed"] == Value::Bool(true));
                let side = side_of(&text(&d["faction"]).to_lowercase());
                let kind = kind_of(&text(&d["kind"]));
                let table = text(&d["table"]);
                let name = text(&d["name"]);
                let color = tribe_color(&text(&d["color"]));
                view! {
                    <div class="dossier-head">
                        {(table != "command_group").then(|| counter(name.clone(), side, kind, text(&d["printed"]), color, false))}
                        <div>
                            <p class="kicker">{format!("dossier · {}", text(&d["faction"]))}</p>
                            <h2>{name.clone()}{d["count"].as_i64().filter(|n| *n > 1).map(|n| format!(" (×{n})"))}</h2>
                            <p class="classes">{strings(&d["classes"]).join(" › ")}</p>
                            {[&d["title"], &d["role"], &d["description"], &d["history"], &d["note"]].into_iter().filter(|x| !text(x).is_empty())
                                .map(|x| view! { <p>{text(x)}</p> }).collect_view()}
                            {(!strings(&d["aka"]).is_empty()).then(|| view! { <p class="small">"Also: "{strings(&d["aka"]).join(", ")}</p> })}
                        </div>
                    </div>
                    <div class="dossier-grid">
                        <div>
                            <h3>"Armament"</h3>
                            {if strings(&d["lines"]).is_empty() { view! { <p class="small">"No range line (movement only, or set per tribe)."</p> }.into_any() }
                             else { view! { <ul class="plain">{strings(&d["lines"]).into_iter().map(|l| view! { <li>{l}</li> }).collect_view()}</ul> }.into_any() }}
                            {(!list(&d["command"]).is_empty()).then(|| view! { <h3>"Command"</h3><p>{value_view(&d["command"])}</p> })}
                            {(!list(&d["members"]).is_empty()).then(|| view! { <h3>"Tribes"</h3><p>{value_view(&d["members"])}</p> })}
                            {(!list(&d["scenarios"]).is_empty()).then(|| view! { <h3>"Appears in"</h3><p>{value_view(&d["scenarios"])}</p> })}
                            {(!list(&d["enters"]).is_empty()).then(|| view! {
                                <h3>"Where it may go"</h3>
                                <ul class="plain">{list(&d["enters"]).into_iter().map(|e| view! {
                                    <li class=if e["allowed"] == Value::Bool(true) { "yes" } else { "no" }>
                                        {if e["allowed"] == Value::Bool(true) { "✓ " } else { "✗ " }}{text(&e["place"])}" "{sec_link(&text(&e["section"]))}
                                        {(!text(&e["condition"]).is_empty()).then(|| view! { <span class="small">{format!(" — {}", text(&e["condition"]))}</span> })}
                                    </li>
                                }).collect_view()}</ul>
                            })}
                        </div>
                        <div>
                            <h3>"May"</h3>
                            {ability_list(may, true)}
                            <h3>"May not"</h3>
                            {ability_list(maynot, false)}
                        </div>
                    </div>
                    {(!list(&d["open"]).is_empty()).then(|| view! {
                        <h3>"What the manual leaves open"</h3>
                        <ul class="dispatch-list">{list(&d["open"]).into_iter().map(|o| view! { <li>{text(&o["question"])}</li> }).collect_view()}</ul>
                    })}
                    <details>
                        <summary>{format!("Rules that concern it ({})", list(&d["rules"]).len())}</summary>
                        <ul class="rule-list">{list(&d["rules"]).into_iter().map(|r| view! { <li>{sec_link(&text(&r["section"]))}" "{text(&r["statement"])}</li> }).collect_view()}</ul>
                    </details>
                }
            })}
            {query_chip(move || s2.clone())}
            <p><a class="small" href=href(&Route::Armies(None))>"close the dossier ×"</a></p>
        </section>
    }
}

fn ability_list(items: Vec<Value>, yes: bool) -> impl IntoView + use<> {
    if items.is_empty() {
        return view! { <p class="small">"—"</p> }.into_any();
    }
    view! {
        <ul class="abilities">{items.into_iter().map(|a| {
            let implied = text(&a["basis"]) == "implied";
            view! {
                <li class:yes=yes class:no=!yes>
                    <span class="ability">{text(&a["action"])}</span>
                    {implied.then(|| view! { <span class="basis" title=text(&a["note"])>"implied"</span> })}
                    " "{sec_link(&text(&a["section"]))}
                    {(!text(&a["condition"]).is_empty()).then(|| view! { <span class="cond">{text(&a["condition"])}</span> })}
                </li>
            }
        }).collect_view()}</ul>
    }.into_any()
}
