#!/usr/bin/env python3
"""Generatore di sequenze per il differential e lo state-machine fuzzing.

Ogni sequenza nasce da un seed (riproducibile) ed è eseguita sull'oracolo
(`oracle.py`): per ogni passo si registrano operazione, parametri, esito atteso e stato
atteso completo. I consumatori (Mollusk per Solana, Foundry per EVM, bernie-state in
nativo) rieseguono la stessa sequenza e confrontano dopo *ogni* passo.

Uso:
  gen.py sol  --seed S --seqs N --ops M [--users U] --out DIR
  gen.py evm  --seed S --seqs N --ops M [--users U] --out DIR
  gen.py state --seed S --seqs N --ops M            (JSONL su stdout, per il consumatore nativo)

Le operazioni includono di proposito casi che devono fallire (importi nulli, oltre il
saldo, oltre la supply, slippage stretto, deleghe mancanti o insufficienti, sweep e
claim senza nulla da ritirare, trasferimenti verso il contratto), sequenze improbabili
e utenti con pochi fondi.
"""
import argparse
import json
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from oracle import Market, TREASURY, RENT0  # noqa: E402

LIMITS = {
    "sol": dict(scale=10**9, min_price=10**6, max_price=10**15),
    "evm": dict(scale=10**18, min_price=10**12, max_price=10**24),
}


def log_uniform(r, lo, hi):
    """Intero con distribuzione log-uniforme in [lo, hi]."""
    if hi <= lo:
        return lo
    x = r.uniform(0, 1)
    import math
    return max(lo, min(hi, int(math.exp(math.log(lo) + x * (math.log(hi) - math.log(lo))))))


def params(r, chain):
    L = LIMITS[chain]
    P = r.choice([L["min_price"], L["max_price"], log_uniform(r, L["min_price"], L["max_price"])])
    p = r.choice([100, 1000, r.randint(100, 1000)])
    e = r.choice([0, p, r.randint(0, p)])
    return P, p, e


def wallets(r, chain, n):
    """Fondi degli utenti: la maggior parte ricchi, alcuni poveri (errori di saldo)."""
    rich = 10**18 if chain == "sol" else 10**27
    out = []
    for i in range(n):
        if i > 0 and r.random() < 0.2:
            out.append(r.randint(RENT0 + 1, 10**9) if chain == "sol" else r.randint(1, 10**16))
        else:
            out.append(rich)
    return out


def units_for(m, native):
    """Quante unità base l'utente può permettersi, a spanne (k cresce, è una stima)."""
    return max(1, native * m.SC // max(m.v.k, 1) // 2)


def amount(r, hi, cap=(1 << 64) - 1):
    return min(_amount(r, hi), cap)


def _amount(r, hi):
    k = r.random()
    if k < 0.25:
        return r.randint(1, 20)                       # polvere: arrotondamenti, Dust, ZeroPayout
    if k < 0.35:
        return r.choice([1, 2, 3, 10, 99, 100, 101, 999, 1000, 10000])
    return log_uniform(r, 1, max(1, hi))


def step(r, m, chain):
    """Sceglie e applica un'operazione; restituisce il record del passo."""
    ev = _step(r, m, chain)
    if chain == "sol":  # su Solana gli importi dell'istruzione sono u64
        for key in ("u", "pay", "min_out", "x", "approve"):
            if isinstance(ev.get(key), int):
                assert ev[key] <= (1 << 64) - 1, (key, ev[key])
    return ev


def _step(r, m, chain):
    n = m.n
    a = r.randrange(n)
    holders = [i for i in range(n) if m.tokens[i] > 0]
    x = r.random()
    ev = {}
    if x < 0.34 or m.v.S == 0 and x < 0.6:
        u = amount(r, units_for(m, m.native[a]))
        if r.random() < 0.02:
            u = 0
        # costo esatto dal modello, per costruire limiti stretti
        probe = m.__class__.__new__(m.__class__)
        probe.__dict__ = __import__("copy").deepcopy(m.__dict__)
        try:
            need = probe._math("mint", u)[1] if u > 0 else 1
        except Exception:
            need = 1
        mode = r.random()
        if chain == "sol":
            pay = need if mode < 0.5 else need * 2 + 7 if mode < 0.8 else max(0, need - 1) if mode < 0.9 else (1 << 64) - 1
            pay = min(pay, (1 << 64) - 1)
        else:
            pay = need if mode < 0.55 else need + r.randint(1, 10**18) if mode < 0.9 else max(0, need - 1)
            pay = min(pay, m.native[a])
        ev = {"op": "mint", "a": a, "u": u, "pay": pay, "t": r.randrange(4)}
        res = m.mint(a, u, pay)
    elif x < 0.66:
        if holders and r.random() < 0.9:
            a = r.choice(holders)
        bal = m.tokens[a]
        mode = r.random()
        u = (bal if mode < 0.25 else r.randint(1, bal) if bal and mode < 0.8 else bal + 1 if mode < 0.85
             else 0 if mode < 0.88 else m.v.S if mode < 0.93 else amount(r, max(bal, 1)))
        if chain == "sol":
            u = min(u, (1 << 64) - 1)
        # min_out: 0, esatto (dal modello) o esatto + 1
        probe = m.__class__.__new__(m.__class__)
        probe.__dict__ = __import__("copy").deepcopy(m.__dict__)
        try:
            out = probe._math("redeem", u, 0)[1]
        except Exception:
            out = 0
        mo = r.random()
        min_out = 0 if mo < 0.6 else out if mo < 0.85 else out + 1
        if chain == "sol":
            min_out = min(min_out, (1 << 64) - 1)
        ev = {"op": "redeem", "a": a, "u": u, "min_out": min_out, "t": r.randrange(4)}
        if chain == "sol":
            dm = r.random()
            approve = u if dm < 0.7 else u + r.randint(1, 100) if dm < 0.8 else max(0, u - 1) if dm < 0.9 else None
            if approve is not None:
                approve = min(approve, (1 << 64) - 1)
            ev["approve"] = approve
            res = m.redeem(a, u, min_out, approve)
        else:
            res = m.redeem(a, u, min_out)
    elif x < 0.74:
        xamt = amount(r, max(1, m.native[a] // 1000))
        if r.random() < 0.03:
            xamt = 0
        if chain == "evm":
            xamt = min(xamt, m.native[a])
        ev = {"op": "donate", "a": a, "x": xamt}
        res = m.donate(a, xamt)
    elif x < 0.81:
        ev = {"op": "sweep", "a": a}
        res = m.sweep()
    elif x < 0.90:
        if holders and r.random() < 0.9:
            a = r.choice(holders)
        b = r.randrange(n)
        bal = m.tokens[a]
        xamt = r.choice([bal, bal + 1, r.randint(0, bal) if bal else 0, 1])
        if chain == "sol":
            xamt = min(xamt, (1 << 64) - 1)
        if chain == "evm" and r.random() < 0.05:
            ev = {"op": "transfer_self", "a": a, "x": xamt}
            res = {"err": ["TransferToSelf"]}    # Bernie._update lo controlla prima del saldo
        else:
            ev = {"op": "transfer", "a": a, "b": b, "x": xamt}
            res = m.transfer(a, b, xamt)
    elif chain == "sol":
        ev = {"op": "revoke", "a": a}
        res = m.revoke(a)
    else:
        if r.random() < 0.5:
            who = r.choice([0, TREASURY, a])
            ev = {"op": "claim", "who": who}
            res = m.claim(who)
        else:
            who = r.choice([0, TREASURY])
            ev = {"op": "claim_all", "who": who, "caller": a}
            res = m.claim(who)   # claimAll su un solo token: stesso effetto di claimFeesFor
    ev["expect"] = res
    ev["state"] = m.snapshot()
    return ev


def sparsify(ev, i, every):
    """Marathon: per gli utenti si registrano solo quelli toccati dal passo (più il creator),
    e tutti ogni `every` passi. k, R, Q, S e i totali restano completi a ogni passo."""
    if not every or i % every == 0:
        return ev
    touched = {0, ev.get("a", 0), ev.get("b", 0)}
    if isinstance(ev.get("who"), int):
        touched.add(ev["who"])
    st = dict(ev["state"])
    for key in ("native", "tokens", "delegated"):
        st[key] = {str(t): st[key][t] for t in sorted(touched) if isinstance(t, int)}
    ev = dict(ev)
    ev["state"] = st
    return ev


def sequence(seed, chain, ops, users, sparse=0):
    r = random.Random(f"{chain}:{seed}")
    P, p, e = params(r, chain)
    w = wallets(r, chain, users)
    m = Market(chain, P, p, e, w)
    steps = [sparsify(step(r, m, chain), i, sparse) for i in range(ops)]
    return {"seed": seed, "chain": chain, "P": P, "p": p, "e": e, "wallets": w, "steps": steps}


def replay(seq, ops):
    """Riesegue sull'oracolo una lista di operazioni (già scelte) e ricalcola esiti e stati.
    Serve alla minimizzazione: togliendo passi, quelli successivi cambiano esito."""
    m = Market(seq["chain"], seq["P"], seq["p"], seq["e"], seq["wallets"])
    out = []
    for ev in ops:
        ev = {k: v for k, v in ev.items() if k not in ("expect", "state")}
        op = ev["op"]
        if op == "mint":
            res = m.mint(ev["a"], ev["u"], ev["pay"])
        elif op == "redeem":
            res = m.redeem(ev["a"], ev["u"], ev["min_out"], ev.get("approve")) if m.chain == "sol" \
                else m.redeem(ev["a"], ev["u"], ev["min_out"])
        elif op == "donate":
            res = m.donate(ev["a"], ev["x"])
        elif op == "sweep":
            res = m.sweep()
        elif op == "transfer":
            res = m.transfer(ev["a"], ev["b"], ev["x"])
        elif op == "transfer_self":
            res = {"err": ["TransferToSelf"]}
        elif op == "revoke":
            res = m.revoke(ev["a"])
        else:
            res = m.claim(ev["who"])
        ev["expect"] = res
        ev["state"] = m.snapshot()
        out.append(ev)
    return {**seq, "steps": out}


def parse(o):
    """Inverso di stringify per le sequenze salvate."""
    if isinstance(o, str) and (o.isdigit() or (o.startswith("-") and o[1:].isdigit())):
        return int(o)
    if isinstance(o, dict):
        return {k: parse(v) for k, v in o.items()}
    if isinstance(o, list):
        return [parse(v) for v in o]
    return o


def stringify(o):
    """Interi come stringhe decimali: u128 e uint256 non stanno nei numeri JSON."""
    if isinstance(o, bool) or o is None:
        return o
    if isinstance(o, int):
        return str(o)
    if isinstance(o, dict):
        return {k: stringify(v) for k, v in o.items()}
    if isinstance(o, list):
        return [stringify(v) for v in o]
    return o


# ── formato colonnare per Foundry (una riga per passo) ──

OPS = {"mint": 1, "redeem": 2, "donate": 3, "sweep": 4, "transfer": 5, "transfer_self": 6, "claim": 7, "claim_all": 8}
ERRS = ["ok", "ZeroAmount", "ExceedsSupply", "Dust", "ZeroPayout", "Slippage", "NoHolders", "NothingToClaim",
        "TransferToSelf", "InsufficientTokens", "InsufficientNative", "Overflow"]


def err_mask(res):
    if res.get("ok"):
        return 1
    mask = 0
    for e in res["err"]:
        mask |= 1 << ERRS.index(e)
    return mask


def evm_columns(seq):
    n = len(seq["wallets"])
    who = lambda w: n if w == TREASURY else w  # noqa: E731
    cols = {k: [] for k in ["op", "a", "b", "x", "y", "expect", "k", "R", "Q", "S", "avail", "tfo", "trin"]}
    for i in range(n):
        cols[f"native{i}"] = []
        cols[f"tok{i}"] = []
    for i in range(n + 1):
        cols[f"owed{i}"] = []
    for s in seq["steps"]:
        op = s["op"]
        cols["op"].append(OPS[op])
        a = s.get("a", 0)
        b = s.get("b", 0)
        xv = s.get("u", s.get("x", 0))
        yv = s.get("pay", s.get("min_out", 0))
        if op in ("claim", "claim_all"):
            a = who(s["who"])
            b = s.get("caller", 0)
        cols["a"].append(a)
        cols["b"].append(b)
        cols["x"].append(xv)
        cols["y"].append(yv)
        cols["expect"].append(err_mask(s["expect"]))
        st = s["state"]
        for k, key in [("k", "k"), ("R", "R"), ("Q", "Q"), ("S", "S"), ("avail", "available"),
                       ("tfo", "total_fees_owed"), ("trin", "treasury_in")]:
            cols[k].append(st[key])
        for i in range(n):
            cols[f"native{i}"].append(st["native"][i])
            cols[f"tok{i}"].append(st["tokens"][i])
        for i in range(n + 1):
            cols[f"owed{i}"].append(st["fees_owed"][i])
    head = {"seed": seq["seed"], "P": seq["P"], "p": seq["p"], "e": seq["e"], "users": n, "steps": len(seq["steps"]),
            "wallets": seq["wallets"]}
    return stringify({**head, **cols})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("target", choices=["sol", "evm"])
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--seqs", type=int, default=10)
    ap.add_argument("--ops", type=int, default=200)
    ap.add_argument("--users", type=int, default=8)
    ap.add_argument("--out", required=True)
    ap.add_argument("--sparse", type=int, default=0, help="stato utenti completo solo ogni N passi (solo sol)")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    total = 0
    for i in range(a.seqs):
        seed = a.seed + i
        seq = sequence(seed, a.target, a.ops, a.users, a.sparse if a.target == "sol" else 0)
        total += len(seq["steps"])
        path = os.path.join(a.out, f"{a.target}_{seed:08d}.json")
        with open(path, "w") as f:
            json.dump(stringify(seq) if a.target == "sol" else evm_columns(seq), f, separators=(",", ":"))
    print(f"{a.seqs} sequenze, {total} passi in {a.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
