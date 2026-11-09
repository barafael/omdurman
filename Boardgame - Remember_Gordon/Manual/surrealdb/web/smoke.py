#!/usr/bin/env python3
"""Smoke-test the built archive in headless Chromium over its debugging protocol: wait for the corpus
to load, visit every view, report console errors and error panels, run a few interactions, and save a
screenshot of each view.

    python3 web/smoke.py [--shots DIR] [--dark] [--subpath /omdurman/archive/] [--live URL]   (needs `websockets`)

Serves crates/rgapp/dist itself, so build first (trunk build, or build.sh). After the reference
project's bench.py (bavovniatko db/web)."""
import asyncio
import base64
import json
import os
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

import websockets

DIST = Path(__file__).resolve().parent / "crates" / "rgapp" / "dist"
PORT, CDP = 8773, 9335
BASE = f"http://127.0.0.1:{PORT}/"

VIEWS = [
    ("home", "#/", ["Two seats at the table", "The road to Omdurman", "Kitchener"]),
    ("armies", "#/armies", ["Leaders and their commands", "21 Lancers"]),
    ("dossier", "#/armies/unit%3Asultan", ["Sultan", "May not", "Howitzer"]),
    ("dossier-tribe", "#/armies/tribe%3Abaggara", ["Baggara", "spears and swords"]),
    ("field", "#/field", ["Khor Shambat", "Zones of control", "Mahdi's Tomb"]),
    ("clock", "#/clock", ["Turn Record Track", "Dervish Desertion Roll", "Maxim Second Fire"]),
    ("scenarios", "#/scenarios", ["The Campaign Game", "Who has won?", "Victory"]),
    ("scenario-fok", "#/scenarios/scenario%3Afall_of_khartoum", ["GORDON", "British marginal"]),
    ("office", "#/office", ["Stacking", "Osman Digna may only stack", "howitzer"]),
    ("rules", "#/rules/5.24", ["upstream", "Rules drawn from it", "Asked at the table"]),
    ("dispatches", "#/dispatches", ["desertion", "inconsistency"]),
    ("inquiries", "#/inquiries", ["748 questions", "from the table"]),
    ("inquiry", "#/inquiries/s5_24_downstream_then_back", ["the archive answers as the manual does"]),
    ("notebook", "#/notebook", ["Questions to ask", "statement"]),
    ("record", "#/record/location%3Akerreri", ["Kerreri", "Out"]),
    ("about", "#/about", ["About this archive", "question"]),
]


def start(subpath):
    """Serve dist at `subpath` (as GitHub Pages serves a project site) and open Chromium."""
    root = DIST
    if subpath.strip("/"):
        root = Path(tempfile.mkdtemp(prefix="rgsite_"))
        (root / subpath.strip("/")).parent.mkdir(parents=True, exist_ok=True)
        (root / subpath.strip("/")).symlink_to(DIST)
    server = subprocess.Popen([sys.executable, "-m", "http.server", str(PORT), "-d", str(root), "--bind", "127.0.0.1"],
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    browser = subprocess.Popen(["chromium", "--headless=new", "--no-sandbox", "--disable-gpu", "--window-size=1280,900",
                                f"--remote-debugging-port={CDP}", f"--user-data-dir={tempfile.mkdtemp(prefix='rgsmoke_')}", "about:blank"],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(50):
        try:
            tabs = json.load(urllib.request.urlopen(f"http://127.0.0.1:{CDP}/json"))
            return server, browser, next(t["webSocketDebuggerUrl"] for t in tabs if t["type"] == "page")
        except (OSError, StopIteration):
            time.sleep(0.2)
    raise SystemExit("chromium did not open its debugging port")


async def session(ws_url, shots, dark):
    errors = []
    async with websockets.connect(ws_url, max_size=None) as ws:
        n = 0
        events = []

        async def call(method, **params):
            nonlocal n
            n += 1
            await ws.send(json.dumps({"id": n, "method": method, "params": params}))
            while True:
                m = json.loads(await ws.recv())
                if m.get("id") == n:
                    return m.get("result", {})
                events.append(m)

        async def ev(expr):
            r = await call("Runtime.evaluate", expression=expr, awaitPromise=True, returnByValue=True)
            return r.get("result", {}).get("value")

        await call("Runtime.enable")
        await call("Page.enable")
        await call("Emulation.setDeviceMetricsOverride", width=1280, height=900, deviceScaleFactor=1, mobile=False)
        t0 = time.time()
        await call("Page.navigate", url=BASE + "#/")
        for _ in range(240):
            st = await ev("(document.querySelector('.status')||{}).innerText || ''")
            if st and ("ready" in st or "failed" in st):
                break
            await asyncio.sleep(0.5)
        print(f"status: {st!r} after {time.time() - t0:.1f}s", flush=True)
        if dark:
            await ev("document.querySelector('button.night').click(); true")
        ok_all = True
        for name, route, needles in VIEWS:
            await ev(f"location.hash = {json.dumps(route)}; true")
            body = ""
            for _ in range(40):
                await asyncio.sleep(0.25)
                body = await ev("document.querySelector('main').innerText") or ""
                if all(x.lower() in body.lower() for x in needles) and "A rider is bringing" not in body:
                    break
            missing = [x for x in needles if x.lower() not in body.lower()]
            errs = await ev("[...document.querySelectorAll('main .error')].map(e => e.innerText)") or []
            status = "ok " if not missing and not errs else "BAD"
            ok_all &= status == "ok "
            print(f"{status} {name:14} {len(body):6d} chars" + (f"  missing {missing}" if missing else "") + (f"  errors {errs[:3]}" if errs else ""), flush=True)
            if shots:
                await asyncio.sleep(0.4)
                h = await ev("Math.min(document.documentElement.scrollHeight, 4200)")
                await call("Emulation.setDeviceMetricsOverride", width=1280, height=int(h or 900), deviceScaleFactor=1, mobile=False)
                img = await call("Page.captureScreenshot", format="png")
                await call("Emulation.setDeviceMetricsOverride", width=1280, height=900, deviceScaleFactor=1, mobile=False)
                Path(shots, f"{name}{'-dark' if dark else ''}.png").write_bytes(base64.b64decode(img["data"]))
        # interactions
        await ev("location.hash = '#/office'; true")
        await asyncio.sleep(1.5)
        await ev("[...document.querySelectorAll('.palette button.pal')].find(b => b.innerText.includes('HADENDOWA')).click(); true")
        await asyncio.sleep(0.8)
        verdict = await ev("document.querySelector('.stack-grid .headline').innerText")
        print(f"office: Osman Digna + Baggara + Hadendowa -> {verdict!r}", flush=True)
        await ev("location.hash = '#/inquiries'; true")
        await asyncio.sleep(1.5)
        await ev("(() => { const i = document.querySelector('.filters .search'); i.value = 'stack'; i.dispatchEvent(new Event('input', {bubbles: true})); return true; })()")
        await asyncio.sleep(0.6)
        print("inquiries 'stack':", await ev("document.querySelector('main .small').innerText"), flush=True)
        for m in events:
            if m.get("method") == "Runtime.exceptionThrown":
                errors.append(m["params"]["exceptionDetails"].get("text"))
            if m.get("method") == "Runtime.consoleAPICalled" and m["params"]["type"] in ("error", "warning"):
                errors.append(" ".join(str(a.get("value", a.get("description", ""))) for a in m["params"]["args"])[:300])
        print(f"console errors/warnings: {len(errors)}")
        for e in errors[:10]:
            print("  ", e)
        return ok_all and not errors


def main():
    shots = None
    if "--shots" in sys.argv:
        shots = sys.argv[sys.argv.index("--shots") + 1]
        os.makedirs(shots, exist_ok=True)
    sub = sys.argv[sys.argv.index("--subpath") + 1] if "--subpath" in sys.argv else "/"
    global BASE
    BASE = f"http://127.0.0.1:{PORT}/{sub.strip('/')}/".replace("//", "/").replace("http:/", "http://")
    if "--live" in sys.argv:  # test a deployed site instead of the local dist
        BASE = sys.argv[sys.argv.index("--live") + 1].rstrip("/") + "/"
    server, browser, ws_url = start(sub)
    try:
        ok = asyncio.run(session(ws_url, shots, "--dark" in sys.argv))
    finally:
        browser.terminate()
        server.terminate()
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
