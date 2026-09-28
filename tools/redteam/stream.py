#!/usr/bin/env python3
"""Flusso di operazioni per il differential ad alto volume su `bernie-state` (fase 2).

Oracolo: il modello dell'Appendice A (`model/bernie.py`). Scrive su stdout una riga per
operazione, letta da `solana/state/examples/diff_stream.rs`, che applica la stessa
operazione a `Vault<SCALE>` e confronta esito e k, R, Q, S dopo ogni passo.

  H scale price p e              nuova sequenza (seed nel commento '#')
  M u max  ok paid fc fp  k R Q S
  R u min  ok out fc fp   k R Q S
  D a      ok             k R Q S
  <op> ... err Nome       k R Q S   (stato invariato)

Uso: stream.py --seed S --seqs N --ops M --scale 10|1000000000 | cargo run --release --example diff_stream
Gli importi restano entro u64 e i prodotti entro u128, come su Solana. Ogni errore è unico:
il primo nell'ordine di §7 (`ordered.py`), con Overflow nel punto in cui il calcolo esce dal
dominio; i limiti di overflow hanno anche vettori dedicati (`boundary.py`).
"""
import argparse
import math
import os
import random
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "model"))
from bernie import Vault, Err  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ordered  # noqa: E402

U64 = (1 << 64) - 1


def lu(r, lo, hi):
    if hi <= lo:
        return lo
    return max(lo, min(hi, int(math.exp(math.log(lo) + r.random() * (math.log(hi) - math.log(lo))))))


def fits(v):
    return v.k * (v.S + 1) < (1 << 120) and v.S < (1 << 62)


def run(seed, ops, scale, out):
    r = random.Random(f"stream:{scale}:{seed}")
    lo, hi = (1, 10**6) if scale == 10 else (10**6, 10**15)
    P = r.choice([lo, hi, lu(r, lo, hi)])
    p = r.choice([100, 1000, r.randint(100, 1000)])
    e = r.choice([0, p, r.randint(0, p)])
    v = Vault(P, p, e, scale)
    w = out.write
    w(f"# seed {seed}\nH {scale} {P} {p} {e}\n")
    for _ in range(ops):
        x = r.random()
        save = (v.k, v.R, v.Q, v.S, v.bal, v.fc, v.fp)
        fc0, fp0 = v.fc, v.fp
        try:
            if x < 0.45 or v.S == 0 and x < 0.7:
                u = r.randint(1, 30) if r.random() < 0.3 else lu(r, 1, max(1, 10**14 * scale // max(v.k, 1)))
                if r.random() < 0.01:
                    u = 0
                u = min(u, U64)
                mx = U64 if r.random() < 0.8 else r.randint(0, U64 if r.random() < 0.1 else 10**12)
                line = f"M {u} {mx}"
                # primo errore nell'ordine di §7, con i domini di Solana (u64, u128)
                err = ordered.check("sol", v, "M", u, mx)
                ordered.cross(v, "M", u, mx, err)
                if err:
                    raise Err(err)
                res = v.mint(u, mx)
            elif x < 0.9:
                u = r.choice([v.S, r.randint(1, max(v.S, 1)), r.randint(1, 30), v.S + 1, 0])
                u = min(u, U64)
                mo = 0 if r.random() < 0.8 else r.randint(0, 10**12)
                line = f"R {u} {mo}"
                err = ordered.check("sol", v, "R", u, mo)
                ordered.cross(v, "R", u, mo, err)
                if err:
                    raise Err(err)
                res = v.redeem(u, mo)
            else:
                a = r.randint(1, 30) if r.random() < 0.5 else lu(r, 1, 10**12)
                if r.random() < 0.02:
                    a = 0
                line = f"D {a}"
                err = ordered.check("sol", v, "D", a)
                ordered.cross(v, "D", a, 0, err)
                if err:
                    raise Err(err)
                v.donate(a)
                res = None
            if not fits(v):
                raise OverflowError
            tail = f"ok" if res is None else f"ok {res} {v.fc - fc0} {v.fp - fp0}"
        except Err as ex:
            v.k, v.R, v.Q, v.S, v.bal, v.fc, v.fp = save
            tail = f"err {ex.args[0]}"
        except OverflowError:
            # fuori dal dominio di prova: si annulla e si passa oltre
            v.k, v.R, v.Q, v.S, v.bal, v.fc, v.fp = save
            continue
        w(f"{line} {tail} {v.k} {v.R} {v.Q} {v.S}\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--seqs", type=int, default=10)
    ap.add_argument("--ops", type=int, default=1000)
    ap.add_argument("--scale", type=int, default=10**9)
    a = ap.parse_args()
    out = sys.stdout
    for i in range(a.seqs):
        run(a.seed + i, a.ops, a.scale, out)


if __name__ == "__main__":
    main()
