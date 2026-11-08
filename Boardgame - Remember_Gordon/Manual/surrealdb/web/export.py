#!/usr/bin/env python3
"""Stage the corpus for the browser archive: crates/rgapp/public/data/.

The worker loads exactly the files `rg.py load` runs, in the same order (schema, data, questions),
listed in manifest.json with their sizes and a content version.

    python3 web/export.py          # write the dataset
    python3 web/export.py --check  # exit 1 if the staged dataset is stale
"""
import glob
import hashlib
import json
import os
import shutil
import subprocess
import sys

WEB = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.dirname(WEB)
OUT = os.path.join(WEB, "crates", "rgapp", "public", "data")
sys.path.insert(0, CORPUS)
import rg  # noqa: E402


def sources():
    subprocess.run([sys.executable, os.path.join(CORPUS, "tools", "gen_sections.py"), rg.MANUAL,
                    os.path.join(CORPUS, "data", "00_sections.surql")], check=True, capture_output=True)
    qpath = os.path.join(CORPUS, "bench", "questions.generated.surql")
    open(qpath, "w", encoding="utf-8").write(rg.questions_surql(rg.load_questions()))
    files = [os.path.join(CORPUS, "schema.surql")] + sorted(glob.glob(os.path.join(CORPUS, "data", "*.surql"))) + [qpath]
    return [(os.path.relpath(f, CORPUS).replace(os.sep, "/"), f) for f in files]


def manifest(files):
    h = hashlib.sha256()
    entries = []
    for rel, path in files:
        data = open(path, "rb").read()
        h.update(rel.encode() + b"\0" + data)
        entries.append({"path": rel, "bytes": len(data)})
    return {"format": 1, "version": h.hexdigest()[:16], "load": [e["path"] for e in entries], "files": entries,
            "attribution": "Rules text: REMEMBER GORDON! — The Battle of Omdurman, Phoenix Enterprises, Ltd., 1982 (game design Peter Bertram)."}


def main():
    files = sources()
    m = manifest(files)
    mpath = os.path.join(OUT, "manifest.json")
    if "--check" in sys.argv:
        try:
            cur = json.load(open(mpath))
        except FileNotFoundError:
            sys.exit("no staged dataset")
        sys.exit(0 if cur.get("version") == m["version"] else "staged dataset is stale")
    if os.path.isdir(OUT):
        shutil.rmtree(OUT)
    for rel, path in files:
        dst = os.path.join(OUT, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copyfile(path, dst)
    json.dump(m, open(mpath, "w"), indent=1)
    print(f"{len(files)} files, {sum(f['bytes'] for f in m['files']) // 1024} KiB, version {m['version']} -> {os.path.relpath(OUT)}")


if __name__ == "__main__":
    main()
