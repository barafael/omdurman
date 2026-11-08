#!/usr/bin/env python3
"""Load, check and benchmark the Remember Gordon! SurrealDB corpus.

    python3 rg.py load            # (re)build the database: sections, schema, data, questions
    python3 rg.py check           # consistency checks (checks.surql); each must return []
    python3 rg.py bench [-v] [ID...]  # run the question benchmark; writes bench/REPORT.md
    python3 rg.py all             # load + check + bench
    python3 rg.py sql 'SELECT ...'  # ad-hoc query

Server: SURREAL_URL (default http://127.0.0.1:8765), root/root, namespace/database rg.
Start one with ./start.sh (podman, in-memory).
"""
import base64
import glob
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
URL = os.environ.get("SURREAL_URL", "http://127.0.0.1:8765") + "/sql"
AUTH = "Basic " + base64.b64encode(b"root:root").decode()
MANUAL = os.path.join(HERE, "..", "RememberGordonManual.md")


def sql(text, ns="rg", db="rg"):
    req = urllib.request.Request(URL, data=text.encode("utf-8"), method="POST")
    req.add_header("Accept", "application/json")
    req.add_header("Authorization", AUTH)
    req.add_header("surreal-ns", ns)
    if db:
        req.add_header("surreal-db", db)
    try:
        with urllib.request.urlopen(req) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:  # parse errors reject the whole request
        body = e.read().decode("utf-8", "replace")
        return [{"status": "ERR", "result": f"HTTP {e.code}: {body[:1500]}"}]


def run_file(path, db="rg"):
    res = sql(open(path, encoding="utf-8").read(), db=db)
    errs = [(i, r["result"]) for i, r in enumerate(res) if r["status"] != "OK"]
    return len(res), errs


def sq(v):
    """SurrealQL literal for a JSON value (strings are JSON-quoted, which SurrealQL accepts)."""
    return json.dumps(v, ensure_ascii=False)


def load_questions():
    qs = []
    for path in sorted(glob.glob(os.path.join(HERE, "bench", "questions_*.json"))):
        for q in json.load(open(path, encoding="utf-8")):
            q["_file"] = os.path.basename(path)
            qs.append(q)
    ids = [q["id"] for q in qs]
    dup = {i for i in ids if ids.count(i) > 1}
    if dup:
        sys.exit(f"duplicate question ids: {sorted(dup)}")
    return qs


def questions_surql(qs):
    out = []
    for q in qs:
        rec = {k: q.get(k) for k in ("text", "kind", "query", "expect", "match", "answer", "note") if q.get(k) is not None}
        rec["section_num"] = q.get("section")
        fields = ", ".join(f"{k}: {sq(v)}" for k, v in rec.items())
        sec = f", section: section:`{q['section']}`" if q.get("section") else ""
        out.append(f"CREATE question:`{q['id']}` CONTENT {{ {fields}{sec} }};")
        for a in q.get("about", []):
            out.append(f"RELATE question:`{q['id']}`->about->{a};")
        if q.get("section"):
            out.append(f"RELATE question:`{q['id']}`->about->section:`{q['section']}`;")
    return "\n".join(out) + "\n"


def cmd_load():
    subprocess.run([sys.executable, os.path.join(HERE, "tools", "gen_sections.py"), MANUAL,
                    os.path.join(HERE, "data", "00_sections.surql")], check=True)
    sql("REMOVE DATABASE IF EXISTS rg; DEFINE DATABASE rg;", db=None)
    total_err = 0
    files = [os.path.join(HERE, "schema.surql")] + sorted(glob.glob(os.path.join(HERE, "data", "*.surql")))
    qs = load_questions()
    qpath = os.path.join(HERE, "bench", "questions.generated.surql")
    open(qpath, "w", encoding="utf-8").write(questions_surql(qs))
    for f in files + [qpath]:
        n, errs = run_file(f)
        total_err += len(errs)
        print(f"{os.path.relpath(f, HERE):40s} {n:5d} statements, {len(errs)} errors")
        for i, e in errs[:8]:
            print(f"    stmt {i}: {str(e)[:300]}")
    counts = sql("RETURN { " + ", ".join(f"{t}: count(SELECT id FROM {t})" for t in TABLES) + " };")[-1]["result"]
    print("records:", json.dumps(counts))
    return total_err


TABLES = ["section", "cites", "faction", "unit_class", "unit", "tribe", "formation", "command_group", "weapon", "range_table",
          "range_line", "terrain", "location", "map", "component", "marker", "state", "combat_result", "constant", "phase",
          "turn", "scenario", "action", "rule", "modifier", "threshold", "die_outcome", "zoc_boundary", "setup",
          "reinforcement", "vp_award", "victory_level", "victory_condition", "open_question", "person", "historical_event",
          "question", "is_a", "belongs_to", "leads", "member_of", "fires_on", "capability", "entry", "governs", "concerns",
          "exception_to", "allowed_in", "prohibits", "next", "about", "raises", "appears_in"]


def cmd_check():
    text = open(os.path.join(HERE, "checks.surql"), encoding="utf-8").read()
    blocks = [b.strip() for b in text.split("\n-- CHECK ") if b.strip()]
    bad = 0
    for b in blocks:
        if b.startswith("--"):
            continue
        name, _, body = b.partition("\n")
        res = sql(body)
        r = res[-1]
        ok = r["status"] == "OK" and r["result"] in ([], None, 0)
        bad += not ok
        print(f"{'ok  ' if ok else 'FAIL'} {name.strip()}" + ("" if ok else f"\n      {json.dumps(r['result'], ensure_ascii=False)[:600]}"))
    print(f"{len(blocks) - bad}/{len(blocks)} checks passed")
    return bad


def norm(v):
    if isinstance(v, list):
        return [norm(x) for x in v]
    if isinstance(v, dict):
        return {k: norm(x) for k, x in sorted(v.items())}
    if isinstance(v, float) and v.is_integer():
        return int(v)
    return v


def matches(got, exp, mode):
    got, exp = norm(got), norm(exp)
    if mode == "set":
        key = lambda x: json.dumps(x, sort_keys=True)
        return isinstance(got, list) and sorted(map(key, got)) == sorted(map(key, exp))
    if mode == "contains":
        return isinstance(got, list) and all(e in got for e in exp)
    return got == exp


def cmd_bench(verbose=False, only=None):
    qs = load_questions()
    results, by_kind = [], {}
    for q in qs:
        if only and q["id"] not in only:
            continue
        k = q.get("kind", "basic")
        stat = by_kind.setdefault(k, [0, 0, 0])
        if not q.get("query"):
            stat[2] += 1
            results.append((q, "NOQUERY", None))
            continue
        try:
            res = sql(q["query"])
            last = res[-1]
            got = last["result"] if last["status"] == "OK" else {"ERROR": last["result"]}
        except Exception as e:  # noqa: BLE001
            got = {"ERROR": str(e)}
        ok = matches(got, q.get("expect"), q.get("match", "exact"))
        stat[0 if ok else 1] += 1
        results.append((q, "PASS" if ok else "FAIL", got))
        sql(f"UPDATE question:`{q['id']}` SET last_result = {sq(got)}, passed = {'true' if ok else 'false'};")
    p = sum(1 for _, s, _ in results if s == "PASS")
    f = sum(1 for _, s, _ in results if s == "FAIL")
    n = sum(1 for _, s, _ in results if s == "NOQUERY")
    for q, s, got in results:
        if s == "FAIL" or (verbose and s != "PASS"):
            print(f"{s} {q['id']}: {q['text']}\n   query:  {q.get('query')}\n   expect: {json.dumps(q.get('expect'), ensure_ascii=False)}\n   got:    {json.dumps(got, ensure_ascii=False)[:400]}")
        elif verbose:
            print(f"{s} {q['id']}")
    print(f"\n{p} passed, {f} failed, {n} without a query (of {len(results)})")
    for k, (a, b, c) in sorted(by_kind.items()):
        print(f"  {k:10s} pass {a:4d}  fail {b:3d}  no-query {c:3d}")
    if not only:
        write_report(results, by_kind)
    return f


def write_report(results, by_kind):
    lines = ["# Benchmark report", "", "Generated by `python3 rg.py bench`. Each question is answered by its SurrealQL",
             "query against the corpus; `expect` is the answer the manual supports.", "",
             "| kind | pass | fail | no query |", "|---|---|---|---|"]
    for k, (a, b, c) in sorted(by_kind.items()):
        lines.append(f"| {k} | {a} | {b} | {c} |")
    lines.append("")
    cur = None
    for q, s, got in results:
        sec = q.get("section") or "-"
        if q["_file"] != cur:
            cur = q["_file"]
            lines += ["", f"## {cur}", ""]
        mark = {"PASS": "✅", "FAIL": "❌", "NOQUERY": "➖"}[s]
        lines.append(f"### {mark} `{q['id']}` (§{sec}, {q.get('kind', 'basic')})")
        lines.append(f"**Q:** {q['text']}  ")
        if q.get("answer"):
            lines.append(f"**A:** {q['answer']}  ")
        if q.get("query"):
            lines += ["```sql", q["query"], "```", f"Result: `{json.dumps(got, ensure_ascii=False)[:300]}`"]
        if q.get("note"):
            lines.append(f"_Note:_ {q['note']}")
        lines.append("")
    open(os.path.join(HERE, "bench", "REPORT.md"), "w", encoding="utf-8").write("\n".join(lines) + "\n")


def main():
    args = sys.argv[1:] or ["all"]
    cmd = args[0]
    if cmd == "load":
        sys.exit(1 if cmd_load() else 0)
    if cmd == "check":
        sys.exit(1 if cmd_check() else 0)
    if cmd == "bench":
        v = "-v" in args
        only = [a for a in args[1:] if a != "-v"] or None
        sys.exit(1 if cmd_bench(v, only) else 0)
    if cmd == "sql":
        for r in sql(args[1]):
            print(r["status"], json.dumps(r["result"], ensure_ascii=False, indent=1))
        return
    if cmd == "all":
        e = cmd_load()
        c = cmd_check()
        b = cmd_bench()
        sys.exit(1 if (e or c or b) else 0)
    sys.exit(__doc__)


if __name__ == "__main__":
    main()
