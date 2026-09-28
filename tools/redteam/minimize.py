#!/usr/bin/env python3
"""Minimizza una sequenza che diverge tra oracolo e implementazione (fase 2).

Parte da un file di sequenza fallito (formato `gen.py sol`, oppure il seed di una
sequenza `evm`), toglie passi con delta debugging (ddmin) e, dopo ogni tentativo, ricalcola
gli esiti attesi con l'oracolo (`gen.replay`) e riesegue il consumatore. Si ferma sulla
sequenza più corta che fallisce ancora e la salva con il comando per riprodurla.

Uso:
  minimize.py sol  FILE.json            # consumatore Mollusk
  minimize.py evm  --seed S --ops M [--users U]   # consumatore Foundry
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
# BERNIE_ROOT: albero su cui eseguire i consumatori (per esempio una copia con un mutante).
ROOT = os.environ.get("BERNIE_ROOT") or os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import gen  # noqa: E402


def fails(chain, seq):
    """True se il consumatore fallisce sulla sequenza (dopo il ricalcolo dell'oracolo)."""
    seq = gen.replay(seq, seq["steps"])
    if chain == "sol":
        d = tempfile.mkdtemp(prefix="bernie-min-")
        with open(os.path.join(d, "sol_min.json"), "w") as f:
            json.dump(gen.stringify(seq), f)
        env = dict(os.environ, BERNIE_DIFF_DIR=d)
        p = subprocess.run(["cargo", "test", "-q", "-p", "bernie-program", "--release", "--test", "differential"],
                           cwd=os.path.join(ROOT, "solana"), env=env, capture_output=True, text=True)
        shutil.rmtree(d)
    else:
        d = os.path.join(ROOT, "evm", "diff")
        shutil.rmtree(d, ignore_errors=True)
        os.makedirs(d)
        with open(os.path.join(d, "evm_min.json"), "w") as f:
            json.dump(gen.evm_columns(seq), f)
        p = subprocess.run(["forge", "test", "--match-contract", "DifferentialTest"],
                           cwd=os.path.join(ROOT, "evm"), capture_output=True, text=True)
    return p.returncode != 0, p.stdout + p.stderr


def ddmin(chain, seq):
    steps = seq["steps"]
    n = 2
    while len(steps) >= 2:
        chunk = max(1, len(steps) // n)
        reduced = False
        for i in range(0, len(steps), chunk):
            cand = steps[:i] + steps[i + chunk:]
            if cand and fails(chain, {**seq, "steps": cand})[0]:
                steps, n, reduced = cand, max(n - 1, 2), True
                print(f"  {len(steps)} passi", file=sys.stderr)
                break
        if not reduced:
            if n >= len(steps):
                break
            n = min(len(steps), n * 2)
    return {**seq, "steps": steps}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("chain", choices=["sol", "evm"])
    ap.add_argument("file", nargs="?")
    ap.add_argument("--seed", type=int)
    ap.add_argument("--ops", type=int, default=200)
    ap.add_argument("--users", type=int, default=8)
    ap.add_argument("--out", default=os.path.join(ROOT, "docs", "testing", "repro"))
    a = ap.parse_args()
    if a.file:
        seq = gen.parse(json.load(open(a.file)))
    else:
        seq = gen.sequence(a.seed, a.chain, a.ops, a.users)
    ok, log = fails(a.chain, seq)
    if not ok:
        raise SystemExit("la sequenza non fallisce: niente da minimizzare")
    m = ddmin(a.chain, seq)
    small = gen.replay(m, m["steps"])
    os.makedirs(a.out, exist_ok=True)
    name = f"{a.chain}_seed{seq['seed']}_min{len(small['steps'])}.json"
    path = os.path.join(a.out, name)
    with open(path, "w") as f:
        json.dump(gen.stringify(small), f, indent=1)
    print(f"riproduzione minima: {path} ({len(small['steps'])} passi)")
    print(fails(a.chain, small)[1][-3000:])


if __name__ == "__main__":
    main()
