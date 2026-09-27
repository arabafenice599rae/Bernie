#!/usr/bin/env python3
"""Mutation testing di Bernie (pre-audit, fasi 16–17).

Ogni mutante è applicato a una copia usa-e-getta del repository (mai all'albero di
lavoro), ricompilato e sottoposto alla suite indicata. Classi:

  KILLED         almeno un test fallisce
  SURVIVED       tutti i test passano: lacuna da analizzare
  COMPILE_ERROR  il mutante non compila (non conta come test forte)
  TIMEOUT        la suite supera il tempo limite (conta come rilevato, ma va letto)

Sorgenti dei mutanti:
  solana   cargo-mutants su bernie-program + catalogo mirato (solana_targeted.json);
           ogni mutante richiede `cargo build-sbf` perché Mollusk carica il .so.
  state    catalogo mirato su bernie-state (cargo-mutants copre il resto, vedi README).
  evm      operatori sintattici su evm/src/*.sol + catalogo mirato (evm_targeted.json).

Uso:
  python3 tools/mutation/mutate.py solana [--only REGEX] [--limit N]
  python3 tools/mutation/mutate.py evm    [--only REGEX] [--limit N]
  python3 tools/mutation/mutate.py state  [--only REGEX]

Risultati: docs/testing/mutation/<target>.json e <target>.md.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(ROOT, "docs", "testing", "mutation")
WORK = os.environ.get("BERNIE_MUT_WORK", "/tmp/bernie-mutation")


def sh(cmd, cwd, timeout=None):
    t = time.time()
    try:
        p = subprocess.run(cmd, cwd=cwd, shell=True, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout + p.stderr, time.time() - t
    except subprocess.TimeoutExpired as e:
        out = (e.stdout or b"").decode(errors="replace") if isinstance(e.stdout, bytes) else (e.stdout or "")
        return None, out, time.time() - t


def make_copy():
    """Copia dei file tracciati da git, più le dipendenze non tracciate che servono ai test."""
    if os.path.exists(WORK):
        shutil.rmtree(WORK)
    files = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, check=True).stdout.split(b"\0")
    for f in filter(None, files):
        f = f.decode()
        src, dst = os.path.join(ROOT, f), os.path.join(WORK, f)
        if not os.path.exists(src):
            continue  # file cancellato ma non ancora committato
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copy2(src, dst)
    # file nuovi non ancora tracciati sotto i percorsi dei test
    for extra in ["solana/state/tests", "solana/program/tests", "evm/test", "tools/mutation", "tools/redteam"]:
        for dirpath, _, names in os.walk(os.path.join(ROOT, extra)):
            for n in names:
                src = os.path.join(dirpath, n)
                dst = os.path.join(WORK, os.path.relpath(src, ROOT))
                if not os.path.exists(dst):
                    os.makedirs(os.path.dirname(dst), exist_ok=True)
                    shutil.copy2(src, dst)
    nm = os.path.join(ROOT, "evm", "node_modules")
    if os.path.isdir(nm):
        os.symlink(nm, os.path.join(WORK, "evm", "node_modules"))


# ── mutanti ────────────────────────────────────────────────────────────────


def cargo_mutants(package, exclude=None):
    """Mutanti di cargo-mutants come (file, riga, colonna inizio/fine, sostituzione)."""
    cmd = f"cargo mutants --list --json -p {package}"
    if exclude:
        cmd += f" --exclude-re '{exclude}'"
    rc, out, _ = sh(cmd, os.path.join(WORK, "solana"))
    if rc != 0:
        raise SystemExit("cargo mutants --list fallito:\n" + out[-2000:])
    res = []
    for m in json.loads(out[out.index("["):]):
        sp = m["span"]
        res.append({
            "id": "cm:" + m["name"],
            "file": "solana/" + m["file"],
            "kind": "span",
            "span": (sp["start"]["line"], sp["start"]["column"], sp["end"]["line"], sp["end"]["column"]),
            "replacement": m["replacement"],
            "category": m["genre"],
            "desc": m["name"],
        })
    return res


def targeted(name):
    with open(os.path.join(HERE, name)) as f:
        items = json.load(f)
    for it in items:
        it["kind"] = "text"
        it["id"] = "t:" + it["name"]
    return items


# Operatori sintattici per Solidity: su righe di codice di src/, un'occorrenza alla volta.
SOL_OPS = [
    ("arith", r"(?<![+\-*/=<>!&|])\+(?![+=])", ["-"]),
    ("arith", r"(?<![+\-*/=<>!&|])-(?![-=>])", ["+"]),
    ("arith", r"(?<![*/])\*(?![*=/])", ["/"]),
    ("arith", r"(?<![*/])/(?![*/=])", ["*"]),
    ("arith-assign", r"\+=", ["-="]),
    ("arith-assign", r"-=", ["+="]),
    ("cmp", r"(?<![<>=!])<(?![<=])", ["<="]),
    ("cmp", r"<=", ["<"]),
    ("cmp", r"(?<![<>=!-])>(?![>=])", [">="]),
    ("cmp", r">=", [">"]),
    ("cmp", r"==", ["!="]),
    ("cmp", r"!=", ["=="]),
    ("logic", r"&&", ["||"]),
    ("logic", r"\|\|", ["&&"]),
    ("const", r"\b1e18\b", ["1e17"]),
    ("const", r"\b10_000\b", ["9_999"]),
    ("const", r"\b20\b", ["21"]),
]


def sol_operator_mutants():
    res = []
    for fn in sorted(os.listdir(os.path.join(WORK, "evm", "src"))):
        if not fn.endswith(".sol"):
            continue
        rel = f"evm/src/{fn}"
        lines = open(os.path.join(WORK, rel)).read().split("\n")
        for i, line in enumerate(lines, 1):
            code = line.split("//")[0]
            s = code.strip()
            if (not s or s.startswith(("*", "/*", "import", "pragma", "event", "error", "function", "contract",
                                        "library", "interface", "}", "{", "emit", "mapping"))
                    or "SPDX" in line):
                continue
            for cat, pat, reps in SOL_OPS:
                for m in re.finditer(pat, code):
                    for r in reps:
                        res.append({
                            "id": f"op:{rel}:{i}:{m.start() + 1}:{m.group(0)}->{r}",
                            "file": rel,
                            "kind": "span",
                            "span": (i, m.start() + 1, i, m.end() + 1),
                            "replacement": r,
                            "category": cat,
                            "desc": f"{rel}:{i}: `{m.group(0)}` → `{r}` in `{s[:80]}`",
                        })
            # cancellazione di revert/require condizionali e chiamate di controllo
            if re.match(r"(if \(.*\) )?revert \w+\(\);$", s) or re.match(r"_checkSolvency\(|BernieMath\.checkAt\(|checkAt\(", s):
                res.append({
                    "id": f"del:{rel}:{i}",
                    "file": rel,
                    "kind": "span",
                    "span": (i, 1, i, len(line) + 1),
                    "replacement": "",
                    "category": "delete-check",
                    "desc": f"{rel}:{i}: riga cancellata `{s[:80]}`",
                })
    return res


def apply(m):
    path = os.path.join(WORK, m["file"])
    src = open(path).read()
    if m["kind"] == "text":
        if src.count(m["find"]) != 1:
            return None, f"testo da mutare trovato {src.count(m['find'])} volte"
        return src, src.replace(m["find"], m["replace"])
    l1, c1, l2, c2 = m["span"]
    lines = src.split("\n")
    # colonne 1-based in caratteri, fine esclusa
    start = sum(len(x) + 1 for x in lines[: l1 - 1]) + (c1 - 1)
    end = sum(len(x) + 1 for x in lines[: l2 - 1]) + (c2 - 1)
    return src, src[:start] + m["replacement"] + src[end:]


# ── esecuzione ─────────────────────────────────────────────────────────────

TARGETS = {
    "solana": {
        "build": "cargo build-sbf --manifest-path program/Cargo.toml",
        "test": "cargo test -p bernie-program --release",
        "cwd": "solana",
        "timeout": 300,
    },
    "state": {
        "build": "true",
        "test": "cargo test -p bernie-state --release",
        "cwd": "solana",
        "timeout": 120,
    },
    "evm": {
        "build": "forge build",
        "test": "forge test",
        "cwd": "evm",
        "timeout": 600,
    },
}


def run(target, only=None, limit=None, keep_going=True):
    cfg = TARGETS[target]
    make_copy()
    cwd = os.path.join(WORK, cfg["cwd"])
    if target == "solana":
        muts = targeted("solana_targeted.json") + cargo_mutants("bernie-program")
    elif target == "state":
        muts = targeted("state_targeted.json") + cargo_mutants("bernie-state", exclude="verification")
    else:
        muts = targeted("evm_targeted.json") + sol_operator_mutants()
    if only:
        muts = [m for m in muts if re.search(only, m["id"])]
    if limit:
        muts = muts[:limit]

    rc, out, dt = sh(cfg["build"] + " && " + cfg["test"], cwd, timeout=3600)
    if rc != 0:
        raise SystemExit(f"la suite non passa sull'albero non mutato:\n{out[-3000:]}")
    print(f"baseline verde in {dt:.0f}s; {len(muts)} mutanti", flush=True)

    results = []
    for n, m in enumerate(muts, 1):
        orig, mutated = apply(m)
        path = os.path.join(WORK, m["file"])
        if orig is None:
            results.append({**m, "status": "NOT_APPLIED", "note": mutated})
            print(f"[{n}/{len(muts)}] NOT_APPLIED {m['id']}: {mutated}", flush=True)
            continue
        if orig == mutated:
            continue
        open(path, "w").write(mutated)
        try:
            rc, out, bt = sh(cfg["build"], cwd, timeout=900)
            if rc != 0:
                status, detail, tt = "COMPILE_ERROR", out[-400:], 0
            else:
                rc, out, tt = sh(cfg["test"], cwd, timeout=cfg["timeout"])
                if rc is None:
                    status, detail = "TIMEOUT", ""
                elif rc == 0:
                    status, detail = "SURVIVED", ""
                else:
                    status, detail = "KILLED", killer(out)
        finally:
            open(path, "w").write(orig)
        results.append({**m, "status": status, "killed_by": detail, "secs": round(bt + tt, 1)})
        print(f"[{n}/{len(muts)}] {status:13} {m['id']}  {detail[:100]}", flush=True)
    # ripristino del build all'albero non mutato
    sh(cfg["build"], cwd, timeout=900)
    return results


def killer(out):
    """Nome del primo test fallito, per il rapporto."""
    for pat in (r"\[FAIL[^\]]*\] (\w+)", r"^test (\S+) \.\.\. FAILED", r"---- (\S+) stdout ----", r"panicked at ([^\n]+)"):
        m = re.search(pat, out, re.M)
        if m:
            return m.group(1)
    return "test failed"


def classify_equivalent(results):
    """Sopravvissuti già analisi a mano come equivalenti (equivalent.json, con motivazione)."""
    with open(os.path.join(HERE, "equivalent.json")) as f:
        eq = {e["id"]: e["reason"] for e in json.load(f)}
    for r in results:
        if r["status"] == "SURVIVED" and r["id"] in eq:
            r["status"] = "EQUIVALENT"
            r["killed_by"] = eq[r["id"]]
    return results


def report(target, results):
    os.makedirs(OUT, exist_ok=True)
    for r in results:
        r.pop("span", None)
    with open(os.path.join(OUT, f"{target}.json"), "w") as f:
        json.dump(results, f, indent=1, ensure_ascii=False)
    counts = {}
    for r in results:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    lines = [f"# Mutation testing: {target}", "",
             "Generato da `tools/mutation/mutate.py " + target + "`. " +
             ", ".join(f"{k}: {v}" for k, v in sorted(counts.items())), "",
             "| Esito | Mutante | Rilevato da (o motivo dell'equivalenza) |", "|---|---|---|"]
    order = {"SURVIVED": 0, "NOT_APPLIED": 1, "TIMEOUT": 2, "EQUIVALENT": 3, "COMPILE_ERROR": 4, "KILLED": 5}
    for r in sorted(results, key=lambda r: (order.get(r["status"], 9), r["id"])):
        desc = (r.get("desc") or r.get("name")).replace("|", "\\|")
        by = (r.get("killed_by") or "").replace("|", "\\|").replace("\n", " ")
        by = by if r["status"] == "EQUIVALENT" else by[:80]
        lines.append(f"| {r['status']} | {desc} | {by} |")
    with open(os.path.join(OUT, f"{target}.md"), "w") as f:
        f.write("\n".join(lines) + "\n")
    print(json.dumps(counts))


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("target", choices=TARGETS)
    ap.add_argument("--only")
    ap.add_argument("--limit", type=int)
    a = ap.parse_args()
    res = classify_equivalent(run(a.target, a.only, a.limit))
    report(a.target, res)
    sys.exit(1 if any(r["status"] == "SURVIVED" for r in res) else 0)
