//! The War Office: the rule functions as instruments. Build a stack and have it checked, total a fire
//! attack's modifiers, lay the howitzers and roll for the scatter, roll for the mines and the deserters.
//! Every instrument shows the query it sent.

use leptos::prelude::*;
use serde_json::Value;

use crate::ui::{counter, d10, kind_of, list, sec_link, show, strings, text, use_query};

/// One instrument: its inputs, the query they make, and how its answer reads.
fn instrument(
    title: &'static str,
    sec: &'static str,
    blurb: &'static str,
    inputs: AnyView,
    sql: Memo<String>,
    render: fn(&Value) -> AnyView,
) -> impl IntoView {
    let res = use_query(move || sql.get());
    view! {
        <div class="instrument">
            <h3>{title}" "{sec_link(sec)}</h3>
            <p class="small">{blurb}</p>
            <div class="inputs">{inputs}</div>
            <div class="answer">{move || match res.get() {
                Some(Ok(v)) => render(&v),
                Some(Err(e)) if !e.is_empty() => view! { <p class="error">{e}</p> }.into_any(),
                _ => view! { <p class="loading">"…"</p> }.into_any(),
            }}</div>
            <pre class="sql">{move || sql.get()}</pre>
        </div>
    }
}

fn check(label: &'static str, sig: RwSignal<bool>) -> impl IntoView {
    view! { <label class="check"><input type="checkbox" prop:checked=move || sig.get() on:change=move |ev| sig.set(event_target_checked(&ev))/>{label}</label> }
}

fn slider(label: &'static str, sig: RwSignal<i64>, min: i64, max: i64) -> impl IntoView {
    view! {
        <label class="num-in">{label}
            <input type="range" min=min max=max prop:value=move || sig.get().to_string() on:input=move |ev| sig.set(event_target_value(&ev).parse().unwrap_or(min))/>
            <output>{move || sig.get()}</output>
        </label>
    }
}

fn die(label: &'static str, sig: RwSignal<i64>) -> impl IntoView {
    view! {
        <div class="die-in">
            <span>{label}</span>
            <span class="die" title="ten-sided die">{move || sig.get()}</span>
            <button on:click=move |_| sig.set(d10())>"roll the d10"</button>
        </div>
    }
}

fn select(
    label: &'static str,
    sig: RwSignal<String>,
    options: &'static [(&'static str, &'static str)],
) -> impl IntoView {
    view! {
        <label class="num-in">{label}
            <select on:change=move |ev| sig.set(event_target_value(&ev))>
                {options.iter().map(|(v, l)| view! { <option value=*v selected=move || sig.get() == *v>{*l}</option> }).collect_view()}
            </select>
        </label>
    }
}

fn big(v: impl Into<String>, label: &'static str) -> AnyView {
    let v = v.into();
    view! { <div class="big"><span class="big-n">{v}</span><span class="small">{label}</span></div> }.into_any()
}

fn signed(v: &Value) -> String {
    match v.as_i64() {
        Some(n) if n > 0 => format!("+{n}"),
        Some(n) => n.to_string(),
        None => text(v),
    }
}

const PALETTE: &str = "RETURN {
  dervish: array::flatten([(SELECT id, name, class AS kind FROM unit WHERE class = unit_class:d_leader ORDER BY name), (SELECT id, name, unit_class AS kind, color FROM tribe ORDER BY name),
    (SELECT id, name, class AS kind FROM unit WHERE id IN [unit:isa_zachneih, unit:dervish_artillery, unit:dervish_gunboat, unit:fort])]),
  anglo_egyptian: array::flatten([(SELECT id, name, class AS kind FROM unit WHERE class = unit_class:ae_leader ORDER BY name),
    (SELECT id, name, id AS kind FROM unit_class WHERE id IN [unit_class:british_infantry, unit_class:egyptian_infantry, unit_class:sudanese_infantry, unit_class:friendlies]),
    (SELECT id, name, class AS kind FROM unit WHERE id IN [unit:lancers_21, unit:camel_corps, unit:battery_32, unit:maxim_battery, unit:royal_engineers, unit:sultan, unit:old_gunboat])])
};";

#[component]
pub fn Office() -> impl IntoView {
    view! {
        <section class="page-head">
            <h1>"The War Office"</h1>
            <p class="lede">"The rules as instruments. Each one sends its question to the archive's rule functions and shows the
               query; the dice are ten-sided, as in the box."</p>
        </section>
        <Stacking />
        <div class="instruments">
            <FireControl />
            <Melee />
            <Howitzer />
            <Targets />
            <Mine />
            <Desertion />
            <Disruption />
            <Night />
        </div>
    }
}

#[component]
fn Stacking() -> impl IntoView {
    let palette = use_query(|| PALETTE.into());
    let hex: RwSignal<Vec<(String, String, String, String)>> = RwSignal::new(vec![
        (
            "unit:osman_digna".into(),
            "Osman Digna".into(),
            "unit_class:d_leader".into(),
            "dv".into(),
        ),
        (
            "tribe:baggara".into(),
            "Baggara".into(),
            "unit_class:d_infantry".into(),
            "dv".into(),
        ),
    ]);
    let sql = Memo::new(move |_| {
        format!(
            "RETURN fn::stack_check([{}]);",
            hex.get()
                .iter()
                .map(|h| h.0.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
    let res = use_query(move || sql.get());
    let chip = move |side: &'static str, u: Value| {
        let (id, name, kind) = (text(&u["id"]), text(&u["name"]), text(&u["kind"]));
        view! {
            <button class="pal" on:click=move |_| hex.update(|h| h.push((id.clone(), name.clone(), kind.clone(), side.into())))>
                {counter(text(&u["name"]), side, kind_of(&text(&u["kind"])), String::new(), None, true)}
            </button>
        }
    };
    view! {
        <section class="instrument wide">
            <h3>"Stacking: may these share a hex?"" "{sec_link("5.51")}" "{sec_link("5.52")}" "{sec_link("5.53")}</h3>
            <p class="small">"Four units at most, leaders free; gunboats alone; never two Dervish tribes; a Dervish leader only with
               his own command. Click counters to add them to the hex, click a counter in the hex to take it out. A tribe stands for one of its units."</p>
            <div class="stack-grid">
                <div class="palette">{show(palette, move |p| view! {
                    <h4>"Dervish"</h4><div class="counters">{list(&p["dervish"]).into_iter().map(|u| chip("dv", u)).collect_view()}</div>
                    <h4>"Anglo-Egyptian"</h4><div class="counters">{list(&p["anglo_egyptian"]).into_iter().map(|u| chip("ae", u)).collect_view()}</div>
                })}</div>
                <div class="hex-panel">
                    <div class="hex">
                        {move || hex.get().into_iter().enumerate().map(|(i, (_, name, kind, side))| view! {
                            <button class="pal" title="remove" on:click=move |_| hex.update(|h| { h.remove(i); })>{counter(name, &side, kind_of(&kind), String::new(), None, true)}</button>
                        }).collect_view()}
                        {move || hex.get().is_empty().then(|| view! { <p class="small">"An empty hex."</p> })}
                    </div>
                    <button class="link" on:click=move |_| hex.set(vec![])>"clear the hex"</button>
                    <div class="answer">{move || match res.get() {
                        Some(Ok(v)) => {
                            let legal = v["legal"].clone();
                            let (cls, word) = match legal { Value::Bool(true) => ("verdict-yes", "Legal"), Value::Bool(false) => ("verdict-no", "Not allowed"), _ => ("verdict-open", "The rules don't settle it") };
                            view! {
                                <p class=format!("headline {cls}")>{word}</p>
                                <p class="small">{format!("{} unit(s) count toward the limit of four", text(&v["counted_units"]))}</p>
                                <ul class="plain">{strings(&v["violations"]).into_iter().map(|s| view! { <li class="no">{format!("✗ {s}")}</li> }).collect_view()}
                                    {strings(&v["undetermined"]).into_iter().map(|s| view! { <li class="open">{format!("? {s}")}</li> }).collect_view()}</ul>
                            }.into_any()
                        }
                        Some(Err(e)) if !e.is_empty() => view! { <p class="error">{e}</p> }.into_any(),
                        _ => view! { <p class="loading">"…"</p> }.into_any(),
                    }}</div>
                    <pre class="sql">{move || sql.get()}</pre>
                </div>
            </div>
        </section>
    }
}

#[component]
fn FireControl() -> impl IntoView {
    let firer = RwSignal::new("faction:anglo_egyptian".to_string());
    let (direct, integrity, fort, hedge, entrenched) = (
        RwSignal::new(true),
        RwSignal::new(true),
        RwSignal::new(false),
        RwSignal::new(false),
        RwSignal::new(false),
    );
    let terrain = RwSignal::new(0i64);
    let roll = RwSignal::new(6i64);
    let sql = Memo::new(move |_| {
        format!(
            "LET $drm = fn::fire_drm({{ firer: {}, direct: {}, integrity: {}, target_in_fort: {}, terrain: {}, across_thorn_hedge: {}, target_entrenched: {} }});\nRETURN {{ modifier: $drm, net_roll: fn::net_roll({}, $drm) }};",
            firer.get(),
            direct.get(),
            integrity.get(),
            fort.get(),
            terrain.get(),
            hedge.get(),
            entrenched.get(),
            roll.get()
        )
    });
    let inputs = view! {
        {select("Firing side", firer, &[("faction:anglo_egyptian", "Anglo-Egyptian"), ("faction:dervish", "Dervish")])}
        {check("direct fire", direct)}{check("brigade integrity, all four at one hex", integrity)}
        {check("target stacked in a fort (−3)", fort)}{slider("terrain modifier (from the TEC)", terrain, -4, 0)}
        {check("across a thorn hedge (Dervish fire −2)", hedge)}{check("target entrenched (Dervish fire −4)", entrenched)}
        {die("the die", roll)}
    }.into_any();
    instrument(
        "Fire control: the die modifiers",
        "6.24",
        "Anglo-Egyptian direct fire +1 and brigade integrity +1; a fort −3; the Zariba's hedge and trench against Dervish fire; the net roll is held to 1–10. Cross-index it with the factors firing on the CRT (on the mapsheet).",
        inputs,
        sql,
        |v| {
            view! { <div class="bigs">{big(signed(&v["modifier"]), "die modifier")}{big(text(&v["net_roll"]), "net roll")}</div> }.into_any()
        },
    )
}

#[component]
fn Melee() -> impl IntoView {
    let attacker = RwSignal::new("faction:dervish".to_string());
    let entrenched = RwSignal::new(false);
    let sql = Memo::new(move |_| {
        let a = attacker.get();
        let defender = if a == "faction:dervish" {
            "faction:anglo_egyptian"
        } else {
            "faction:dervish"
        };
        format!(
            "RETURN {{ attacker: fn::melee_mod({a}, {}), defender: fn::melee_mod({defender}, false) }};",
            entrenched.get()
        )
    });
    let inputs = view! {
        {select("Attacker", attacker, &[("faction:dervish", "Dervish"), ("faction:anglo_egyptian", "Anglo-Egyptian"), ("unit_class:friendlies", "the Friendlies")])}
        {check("the defender is entrenched behind a trench", entrenched)}
    }.into_any();
    instrument(
        "Melee: hand to hand",
        "7.7",
        "Both sides roll on the CRT, simultaneously; the Dervish add 2, the Anglo-Egyptians 1 (the Friendlies use the Dervish modifier). Against an entrenched unit the Dervish modifier becomes −2. No other terrain counts.",
        inputs,
        sql,
        |v| {
            view! { <div class="bigs">{big(signed(&v["attacker"]), "attacker's modifier")}{big(signed(&v["defender"]), "defender's modifier")}</div> }.into_any()
        },
    )
}

#[component]
fn Howitzer() -> impl IntoView {
    let dist = RwSignal::new(6i64);
    let night = RwSignal::new(false);
    let crt = RwSignal::new(5i64);
    let impact = RwSignal::new(8i64);
    let sql = Memo::new(move |_| {
        format!(
            "RETURN {{ in_range: fn::howitzer_range_ok({}), night_forbids: {} AND (state:night->prohibits->action CONTAINS action:howitzer_fire), crt_roll: {}, impact: fn::howitzer_impact({}) }};",
            dist.get(),
            night.get(),
            crt.get(),
            impact.get()
        )
    });
    let inputs = view! {
        {slider("hexes from the gunboat to the target", dist, 1, 12)}{check("it is a night turn", night)}
        {die("first roll (CRT)", crt)}{die("second roll (impact)", impact)}
    }.into_any();
    instrument(
        "The named gunboats' howitzers",
        "6.64",
        "Pick a target 4 to 10 hexes away, ignoring line of sight, and roll twice: the CRT roll, then the impact roll. 7–10 hits the target; anything else lands per the Scattergram, even on your own men. Never at night.",
        inputs,
        sql,
        |v| {
            let blocked = v["night_forbids"] == Value::Bool(true);
            let out = v["in_range"] != Value::Bool(true);
            view! {
                {if blocked { view! { <p class="headline verdict-no">"No howitzer fire at night"</p> }.into_any() }
                 else if out { view! { <p class="headline verdict-no">"Out of the 4–10 hex band"</p> }.into_any() }
                 else { view! { <p class="headline" class:long={text(&v["impact"]).len() > 28}>{text(&v["impact"])}</p><p class="small">{format!("CRT roll {} — read against the howitzer factor", text(&v["crt_roll"]))}</p> }.into_any() }}
            }.into_any()
        },
    )
}

#[component]
fn Targets() -> impl IntoView {
    let target = RwSignal::new("gunboat".to_string());
    let result = RwSignal::new(2i64);
    let sql = Memo::new(move |_| {
        format!(
            "RETURN fn::special_hit({}, {});",
            serde_json::to_string(&target.get()).unwrap_or_default(),
            result.get()
        )
    });
    let inputs = view! {
        {select("Target", target, &[("gunboat", "a gunboat"), ("fort", "a fort"), ("wall hexside", "a wall hexside"), ("river chain", "the river chain")])}
        {slider("result on the CRT", result, 0, 6)}
    }.into_any();
    instrument(
        "Only artillery: gunboats, forts, walls",
        "6.61",
        "Gunboats sink on 3 or more; forts fall and walls breach on 2 or more (taking one enemy unit with them); the optional chain sinks on 3. Anything less is a miss.",
        inputs,
        sql,
        |v| {
            if v == &Value::Bool(true) {
                view! { <p class="headline verdict-yes">"Hit"</p> }.into_any()
            } else {
                view! { <p class="headline verdict-no">"Miss"</p> }.into_any()
            }
        },
    )
}

#[component]
fn Mine() -> impl IntoView {
    let roll = RwSignal::new(6i64);
    let sql = Memo::new(move |_| format!("RETURN fn::mine({});", roll.get()));
    instrument(
        "A British gunboat strikes a mine",
        "10.12",
        "Optional rule. The gunboat stops; the Dervish player rolls. Two mines in all, secretly recorded south of the Khor Shambat.",
        view! { {die("the blast", roll)} }.into_any(),
        sql,
        |v| view! { <p class="headline" class:long={text(v).len() > 28}>{text(v)}</p> }.into_any(),
    )
}

#[component]
fn Desertion() -> impl IntoView {
    let roll = RwSignal::new(7i64);
    let sql = Memo::new(move |_| format!("RETURN fn::desertion({});", roll.get()));
    instrument(
        "The first night: desertion",
        "8.2",
        "Once per campaign, on the first night turn: 1½ × the roll Dervish units slip away. Not the Khalifa, the gunboats, the artillery or the forts. No victory points for deserters.",
        view! { {die("the roll", roll)} }.into_any(),
        sql,
        |v| {
            view! {
            {big(text(&v["units"]), "units desert")}
            {(text(&v["rounding"]) == "unstated").then(|| view! { <p class="small open">"? Half a unit: the rules don't say how to round (see Dispatches)."</p> })}
        }.into_any()
        },
    )
}

#[component]
fn Disruption() -> impl IntoView {
    let n = RwSignal::new(3i64);
    let sql = Memo::new(move |_| format!("RETURN fn::disrupted_count({});", n.get()));
    instrument(
        "A “D” on the CRT",
        "CombatResults",
        "Half the units in the target hex, rounded up, are disrupted (inverted): no ZOC, no movement, no fire, no melee, until the end of their own player turn.",
        view! { {slider("units in the target hex", n, 1, 4)} }.into_any(),
        sql,
        |v| big(text(v), "units disrupted"),
    )
}

#[component]
fn Night() -> impl IntoView {
    let side = RwSignal::new("faction:anglo_egyptian".to_string());
    let ma = RwSignal::new(15i64);
    let range = RwSignal::new(5i64);
    let sql = Memo::new(move |_| {
        format!(
            "RETURN {{ movement: fn::night_ma({}, {}), range: fn::night_range({}) }};",
            side.get(),
            ma.get(),
            range.get()
        )
    });
    let inputs = view! {
        {select("Side", side, &[("faction:anglo_egyptian", "Anglo-Egyptian"), ("faction:dervish", "Dervish")])}
        {slider("movement allowance by day", ma, 1, 18)}{slider("fire range by day", range, 1, 10)}
    }.into_any();
    instrument(
        "By night",
        "8.1",
        "Anglo-Egyptian movement is halved, rounding down; every range is halved for both sides, but range 1 stays range 1.",
        inputs,
        sql,
        |v| {
            view! { <div class="bigs">{big(text(&v["movement"]), "movement at night")}{big(text(&v["range"]), "range at night")}</div> }.into_any()
        },
    )
}
