# Remember Gordon! — the browser archive

A static website about the Battle of Omdurman as the 1982 rulebook tells it, built on the SurrealDB corpus
one directory up. The whole database runs **in the visitor's browser**: SurrealDB 4.0 compiled to
WebAssembly, in memory, in a Web Worker. It loads the corpus on every visit (about 3 s) and is then
sealed read-only. There is no server.

The architecture follows the bavovniatko knowledge-base explorer (`db/web` in that repository), scaled
down to one corpus.

```
../schema.surql, ../data/*.surql, ../bench/questions_*.json
        └─ export.py ─▶ crates/rgapp/public/data/   the files rg.py loads, in its order, + manifest.json
crates/rgcore    open in-memory SurrealDB, load the manifest as root, seal to a guest session, query → JSON
crates/rgworker  Web Worker hosting rgcore; JSON messages to and from the app
crates/rgapp     Leptos UI (client-side, hash routes)
crates/rgcheck   native: load the dataset exactly as the worker does, run all 748 questions as the guest, probe writes
smoke.py         headless Chromium over CDP: every view, console errors, two interactions, screenshots
```

## Running it

```sh
python3 web/export.py                                # after any change to the corpus
cargo run --release -p rgcheck                       # in web/: 748 questions as a guest + write probes
(cd web/crates/rgapp && trunk serve --release)       # http://127.0.0.1:8771
web/build.sh                                         # production build → web/crates/rgapp/dist (static files)
python3 web/smoke.py --shots /tmp/shots [--dark]     # after a build; needs the `websockets` package
```

`web/` is its own Cargo workspace (it is not a member of the game's workspace), so the game's CI and
`cargo test --workspace` never build it. It uses the repository's pinned toolchain and needs the
`wasm32-unknown-unknown` target, `trunk`, and optionally `wasm-opt`.

## The views

Every view is a hash route, and every panel shows the SurrealQL it ran ("▸ the query", with a link
into the notebook).

| route | what it is |
|---|---|
| `#/` **Prologue** | the battle, Queen Victoria's two telegrams, the two seats at the table, the road to Omdurman (1881–1898, the three playable days marked), the people, ways in with example questions |
| `#/armies[/<id>]` **Armies** | both orders of battle as cardboard counters with unit-type glyphs; the Dervish commands by leader. A **dossier** for any unit, tribe, class or command: armament, what it may and may not do (each with its rule, section and basis), where it may go, scenarios, open questions |
| `#/field` **Field** | a schematic of the theatre (placed only from the set-up rules and historical notes, and labelled as such), the zone-of-control boundary table, every named place of both maps, the terrain |
| `#/clock` **Clock** | the Turn Record Track as printed (serpentine, night boxes shaded, the desertion roll), and the sequence of play per scenario |
| `#/scenarios[/<id>]` **Scenarios** | the three scenarios: set-up, arrivals, special rules, victory; a live "who has won?" calculator for each |
| `#/office` **War Office** | the rule functions as instruments: a stacking hex, fire control, melee, the howitzers' two rolls, special targets, river mines, desertion, the D result, night |
| `#/rules[/<n>]` **Rulebook** | every section verbatim, what it cites and is cited by, the atomic rules drawn from it, what it leaves open, and the questions asked about it |
| `#/dispatches` **Dispatches** | the open questions: gaps, ambiguities, inconsistencies of the manual |
| `#/inquiries[/<id>]` **Inquiries** | the 748 benchmark questions; filter, search, open one, run its query, see it checked against the manual's answer |
| `#/notebook?q=…` **Notebook** | free SurrealQL (read-only) with a gallery of example questions |
| `#/record/<id>` | any record: its fields and every edge in and out |

The ☾ button switches to a night-turn theme.

## How it stays read-only

`Kb::open` starts SurrealDB with a random root password that never leaves the worker, and with guest
access. The corpus loads as root; `Kb::seal` then drops to an anonymous guest session for good. Every
table is defined `PERMISSIONS FOR select FULL` (in `../schema.surql`), so a guest's writes are no-ops and
`DEFINE`/`REMOVE` are refused. `rgcheck` asserts this, and in any case a visitor only ever touches their
own in-memory copy.

## Licences

SurrealDB is under the Business Source License 1.1; the footer links it. The rules text quoted is from
*REMEMBER GORDON! — The Battle of Omdurman*, Phoenix Enterprises, Ltd., 1982. The fonts come from
Google Fonts (the only third-party request the site makes).
