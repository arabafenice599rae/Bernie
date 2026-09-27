#!/usr/bin/env python3
"""Genera i vettori di test condivisi (§17, "Differenziale") dal modello dell'Appendice A.

Un file per SCALE in vectors/: 10 (host, esaustivo), 10⁹ (Solana), 10¹⁸ (EVM).
Ogni caso parte da create(P, p, e) e applica passi mint/redeem/donate; per ogni passo
registra l'esito (valore restituito o errore) e lo stato completo dopo il passo.
Tutti gli interi sono stringhe decimali. L'uscita è deterministica: la CI rigenera
i file e fallisce se differiscono da quelli committati.

Uso: python3 tools/gen_vectors.py [--check]
"""
import copy
import json
import os
import random
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "model"))
from bernie import Err, Vault  # noqa: E402

U64 = 2**64 - 1
U128 = 2**128 - 1

TARGETS = {
    # nome file: (SCALE, prezzo min, prezzo max, tetto di R per restare realistici, tipi Solana)
    "scale_10.json": (10, 1, 10**4, 10**19, True),
    "scale_1e9.json": (10**9, 10**6, 10**15, 6 * 10**26, True),
    "scale_1e18.json": (10**18, 10**12, 10**24, 10**54, False),
}


def state(v):
    return {"k": str(v.k), "R": str(v.R), "Q": str(v.Q), "S": str(v.S),
            "bal": str(v.bal), "fc": str(v.fc), "fp": str(v.fp)}


class Case:
    def __init__(self, name, P, p, e, SCALE, solana):
        self.v = Vault(P, p, e, SCALE)
        self.solana = solana
        self.out = {"name": name,
                    "params": {"price": str(P), "penalty_bps": p, "entry_bps": e},
                    "steps": []}

    def step(self, op, **args):
        v = copy.deepcopy(self.v)
        try:
            if op == "mint":
                res = v.mint(args["u"], args.get("max_cost"))
            elif op == "redeem":
                res = v.redeem(args["u"], args.get("min_out", 0))
            elif op == "donate":
                res = v.donate(args["a"])
            else:
                raise ValueError(op)
            expect = {"ok": True, "result": None if res is None else str(res)}
            if self.solana:
                self.check_solana_types(v, args, res)
            self.v = v
        except Err as err:
            expect = {"ok": False, "error": str(err)}
        rec = {"op": op}
        rec.update({key: (None if val is None else str(val)) for key, val in args.items()})
        rec["expect"] = expect
        rec["state"] = state(self.v)
        self.out["steps"].append(rec)
        return expect["ok"]

    @staticmethod
    def check_solana_types(v, args, res):
        assert v.S <= U64 and max(v.k, v.R, v.Q) <= U128
        assert (res or 0) <= U64 and v.bal <= U64
        assert all(x is None or x <= U64 for x in args.values())


def fit(v, u, cap):
    """Il più grande u' ≤ u (dimezzando) il cui mint lascia R ≤ cap e il saldo in u64."""
    while u >= 1:
        w = copy.deepcopy(v)
        try:
            cost = w.mint(u)
            if w.R <= cap and w.S <= U64 and w.bal <= U64 and cost <= U64:
                return u
        except Err:
            return u
        u //= 2
    return 0


def random_case(r, idx, SCALE, lo, hi, cap, solana):
    P = r.choice([lo, hi, r.randint(lo, hi), int(10 ** r.uniform(len(str(lo)) - 1, len(str(hi)) - 1)) or lo])
    P = min(max(P, lo), hi)
    p = r.choice([100, 200, 1000, r.randint(100, 1000)])
    e = r.choice([0, p, p // 2, r.randint(0, p)])
    c = Case(f"random_{idx}", P, p, e, SCALE, solana)
    holders = {}
    max_u = max(1, min(U64 if solana else 10**30, cap // max(P, 1) // 4))
    for _ in range(r.randint(5, 40)):
        x = r.random()
        if x < 0.45:
            u = r.choice([1, r.randint(1, 1000), r.randint(1, max_u)])
            # Resta sotto il tetto di R (su Solana: lamport totali realistici, tutto in u64).
            # Con S piccola l'epen assorbita alza molto k', quindi si prova sul modello.
            u = fit(c.v, u, cap)
            if u < 1:
                continue
            who = r.randint(0, 3)
            if c.step("mint", u=u, max_cost=None):
                holders[who] = holders.get(who, 0) + u
        elif x < 0.85 and holders:
            who = r.choice(list(holders))
            u = r.choice([holders[who], r.randint(1, holders[who])])
            if c.step("redeem", u=u, min_out=0):
                holders[who] -= u
                if not holders[who]:
                    del holders[who]
        else:
            c.step("donate", a=r.choice([1, r.randint(1, max(1, P))]))
    return c.out


def epen_witness(SCALE, lo, hi):
    """P e u₁ per il caso epen_rounding (vedi edge_cases)."""
    for n in range(1, 10**6):
        r = n * SCALE % 101
        if 1 <= r <= 99:
            P = 100 * ((n * SCALE - r) // 101) + r   # P + ⌈P/100⌉ = n·SCALE + 1
            if lo <= P <= hi:
                break
    epen = -(-P * 100 // 10_000)
    u1 = epen + 1
    while -(-u1 * P // SCALE) * SCALE - u1 * P + epen >= u1:   # resto del primo mint + epen < u₁
        u1 *= 2
    return P, u1


def edge_cases(SCALE, lo, hi, cap, solana):
    out = []

    c = Case("errors", lo, 200, 100, SCALE, solana)
    c.step("mint", u=0)                      # ZeroAmount
    c.step("donate", a=5)                    # NoHolders
    c.step("redeem", u=1, min_out=0)         # ExceedsSupply (S == 0)
    # 10¹² unità: abbastanza perché il redeem paghi qualcosa su ogni SCALE (Slippage raggiungibile).
    c.step("mint", u=10**12)
    c.step("redeem", u=10**12 + 1, min_out=0)  # ExceedsSupply
    c.step("redeem", u=0, min_out=0)           # ZeroAmount
    c.step("donate", a=0)                      # ZeroAmount
    c.step("mint", u=10, max_cost=0)           # Slippage
    c.step("redeem", u=10**11, min_out=U64)    # Slippage
    c.step("redeem", u=1, min_out=0)           # Dust o ZeroPayout con prezzi bassi
    c.step("donate", a=7)
    c.step("redeem", u=10**12 - 1, min_out=0)
    c.step("redeem", u=1, min_out=0)          # ultimo uscente: S → 0, Q → 0
    c.step("mint", u=5)                      # rientro al k raggiunto
    out.append(c.out)

    c = Case("dust_and_zero_payout", lo, 1000, 0, SCALE, solana)
    c.step("mint", u=1)
    for u in (1, 1):
        c.step("redeem", u=u, min_out=0)
    out.append(c.out)

    c = Case("max_price_large_supply", hi, 1000, 1000, SCALE, solana)
    u_big = max(1, min(U64 if solana else 10**40, cap // hi // 2))
    c.step("mint", u=u_big)
    c.step("mint", u=u_big // 3 + 1)
    c.step("redeem", u=u_big // 2, min_out=0)
    c.step("donate", a=min(U64, 10**18) if solana else 10**30)
    c.step("redeem", u=u_big // 3, min_out=0)
    out.append(c.out)

    c = Case("min_price_max_units", lo, 100, 0, SCALE, solana)
    u_max = min(U64 // 2, cap // lo // 2) if solana else 10**40
    c.step("mint", u=u_max)
    c.step("redeem", u=u_max // 7, min_out=0)
    c.step("mint", u=u_max // 5)
    c.step("redeem", u=1, min_out=0)
    out.append(c.out)

    c = Case("k_growth_many_exits", lo * 7 + 3 if lo * 7 + 3 <= hi else lo, 1000, 1000, SCALE, solana)
    c.step("mint", u=10**6)
    c.step("mint", u=10**6)
    for _ in range(30):
        c.step("redeem", u=31_337, min_out=0)
        c.step("mint", u=12_345)
    out.append(c.out)

    # Arrotondamento per eccesso dell'epen (§6): un caso in cui ⌈·⌉ e ⌊·⌋ danno stati diversi.
    # Lo stato canonico dipende solo da R + Q e da S, quindi il verso dell'arrotondamento
    # dell'epen conta solo se sposta c = ⌈(full + epen)/SCALE⌉. Con e = 1% ed x = P non
    # multiplo di 100 si sceglie x + ⌈x/100⌉ = N·SCALE + 1; un primo mint grande (u₁)
    # lascia k = P, poi il mint di 1 unità paga 1 unità nativa in più che col floor.
    P, u1 = epen_witness(SCALE, lo, hi)
    c = Case("epen_rounding", P, 200, 100, SCALE, solana)
    c.step("mint", u=u1)
    c.step("mint", u=1)
    c.step("mint", u=3)
    c.step("redeem", u=1, min_out=0)
    out.append(c.out)

    # Forma dell'esempio di §8 (p = 500, e = 460): holder, attaccante, redeem della vittima.
    if SCALE >= 10**9:
        c = Case("section_8_shape", lo, 500, 460, SCALE, solana)
        c.step("mint", u=25 * 10**6)
        c.step("mint", u=5 * 10**6)
        c.step("redeem", u=20 * 10**6, min_out=0)
        out.append(c.out)
    return out


def generate():
    files = {}
    for fname, (SCALE, lo, hi, cap, solana) in TARGETS.items():
        r = random.Random(f"bernie-vectors:{SCALE}")
        cases = edge_cases(SCALE, lo, hi, cap, solana)
        cases += [random_case(r, i, SCALE, lo, hi, cap, solana) for i in range(60)]
        doc = {"spec": "Bernie v1.6", "generator": "tools/gen_vectors.py",
               "scale": str(SCALE), "decimals": len(str(SCALE)) - 1,
               "fees_bps": {"creator": 20, "protocol": 20},
               "cases": cases}
        files[fname] = json.dumps(doc, indent=1, sort_keys=True, ensure_ascii=False) + "\n"
    return files


def flat_evm(text):
    """Vettori SCALE 10¹⁸ come array paralleli, un elemento per passo (per vm.parseJson* di Foundry).

    op: 1 mint, 2 redeem, 3 donate. arg: u oppure a. limit: max_cost (mint) oppure min_out
    (redeem), valido se has_limit = 1. first: 1 al primo passo di ogni caso. err: nome
    dell'errore atteso, "" se il passo riesce. Interi come stringhe decimali.
    """
    doc = json.loads(text)
    cols = {c: [] for c in ("first", "price", "p", "e", "op", "arg", "limit", "has_limit", "ok", "err", "result",
                            "k", "R", "Q", "S", "bal", "fc", "fp")}
    for case in doc["cases"]:
        prm = case["params"]
        for i, st in enumerate(case["steps"]):
            cols["first"].append("1" if i == 0 else "0")
            cols["price"].append(prm["price"])
            cols["p"].append(str(prm["penalty_bps"]))
            cols["e"].append(str(prm["entry_bps"]))
            cols["op"].append({"mint": "1", "redeem": "2", "donate": "3"}[st["op"]])
            cols["arg"].append(st.get("u") or st.get("a") or "0")
            lim = st.get("max_cost") if st["op"] == "mint" else st.get("min_out")
            cols["limit"].append("0" if lim is None else lim)
            cols["has_limit"].append("0" if lim is None else "1")
            ok = st["expect"]["ok"]
            cols["ok"].append("1" if ok else "0")
            cols["err"].append("" if ok else st["expect"]["error"])
            cols["result"].append((st["expect"].get("result") or "0") if ok else "0")
            for key in ("k", "R", "Q", "S", "bal", "fc", "fp"):
                cols[key].append(st["state"][key])
    cols["n"] = str(len(cols["op"]))
    return json.dumps(cols, sort_keys=True, separators=(",", ":")) + "\n"


def main():
    check = "--check" in sys.argv
    outdir = os.path.join(ROOT, "vectors")
    os.makedirs(outdir, exist_ok=True)
    stale = []
    files = generate()
    files["scale_1e18.flat.json"] = flat_evm(files["scale_1e18.json"])
    for fname, text in files.items():
        path = os.path.join(outdir, fname)
        if check:
            try:
                with open(path, encoding="utf-8") as f:
                    if f.read() != text:
                        stale.append(fname)
            except FileNotFoundError:
                stale.append(fname)
        else:
            with open(path, "w", encoding="utf-8") as f:
                f.write(text)
            steps = json.loads(text)["n"] if fname.endswith(".flat.json") else text.count(chr(34) + "op" + chr(34))
            print(f"{path}: {steps} passi")
    if stale:
        print("vettori non aggiornati, eseguire tools/gen_vectors.py:", ", ".join(stale))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
