#!/usr/bin/env python3
"""Vettori di confine (pre-audit, fasi 5–9): stati e importi ai limiti, con esito atteso.

Per ogni stato di partenza costruito direttamente (S = 0, 1, 2, grande; Q = 0, 1, S − 1;
k al prezzo minimo e massimo e oltre) e per ogni operazione con argomenti ai bordi
(0, 1, 2, S − 1, S, S + 1, MAX − 1, MAX; slippage esatto − 1, esatto, esatto + 1),
l'oracolo (Appendice A più i domini di chain) calcola esito e stato finale.

  vectors/boundary_sol.txt   SCALE 10⁹, importi u64, k/R/Q u128  → solana/state/tests/boundary.rs
  vectors/boundary_evm.json  SCALE 10¹⁸, uint256                  → evm/test/Boundary.t.sol

Formato sol: righe del flusso di `stream.py`, precedute da `Z k R Q S p e` (stato iniziale).
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(ROOT, "model"))
from bernie import Vault, Err  # noqa: E402

U64 = (1 << 64) - 1
U128 = (1 << 128) - 1
U256 = (1 << 256) - 1


class Big(int):
    """Intero che registra il massimo valore assoluto di ogni risultato intermedio: il
    modello, eseguito su questi interi, dice se una piattaforma a 256 bit traboccherebbe."""
    peak = 0

    def _w(self, x):
        Big.peak = max(Big.peak, abs(int(x)))
        return Big(x)

    def __add__(self, o): return self._w(int(self) + int(o))
    def __radd__(self, o): return self._w(int(o) + int(self))
    def __sub__(self, o): return self._w(int(self) - int(o))
    def __rsub__(self, o): return self._w(int(o) - int(self))
    def __mul__(self, o): return self._w(int(self) * int(o))
    def __rmul__(self, o): return self._w(int(o) * int(self))
    def __floordiv__(self, o): return self._w(int(self) // int(o))
    def __rfloordiv__(self, o): return self._w(int(o) // int(self))
    def __mod__(self, o): return self._w(int(self) % int(o))
    def __neg__(self): return self._w(-int(self))


def peak_of(v, op, u, lim):
    """Massimo intermedio dell'operazione sul modello (None se il modello fallisce prima)."""
    w = Vault.__new__(Vault)
    w.__dict__ = {k: (Big(x) if isinstance(x, int) else x) for k, x in v.__dict__.items()}
    Big.peak = 0
    try:
        if op == "M":
            w.mint(Big(u), Big(lim))
        elif op == "R":
            w.redeem(Big(u), Big(lim))
        else:
            w.donate(Big(u))
    except Err:
        pass
    return Big.peak


def vault(k, S, Q, p, e, SC):
    v = Vault.__new__(Vault)
    v.k, v.R, v.Q, v.S, v.SC, v.p, v.e = k, S * k, Q, S, SC, p, e
    v.bal, v.fc, v.fp = (v.R + v.Q) // SC, 0, 0
    return v


def canonical_states(SC, prices):
    out = []
    for k in prices:
        for S in (0, 1, 2, 3, 10**9, 10**15, U64 - 1):
            for Q in sorted({0, 1, S - 1} if S else {0}):
                if Q < 0 or (S and Q >= S):
                    continue
                if (S * k + Q) % SC:
                    # correggi Q al multiplo più vicino che rispetta I5 e I6
                    Q = (SC - (S * k) % SC) % SC
                    if S and Q >= S:
                        continue
                out.append((k, S, Q))
    return sorted(set(out))


def op_cases(v, chain):
    S = v.S
    amounts = sorted({0, 1, 2, max(S - 1, 0), S, S + 1, U64 - 1, U64})
    for u in amounts:
        yield ("M", u, None)
        yield ("R", u, None)
    for a in (0, 1, 2, U64 - 1, U64):
        yield ("D", a, None)
    # slippage esatto − 1, esatto, esatto + 1 per un mint e un redeem piccoli
    for u in (1, 7):
        yield ("Mx", u, None)
        if S >= u:
            yield ("Rx", u, None)


def run_sol(v, op, u, SC):
    """Esito sull'oracolo con i domini di Solana. Restituisce lista di righe del flusso."""
    lines = []

    def attempt(kind, u, lim):
        w = Vault.__new__(Vault)
        w.__dict__ = dict(v.__dict__)
        fc0, fp0, b0 = w.fc, w.fp, w.bal
        try:
            if kind == "M":
                if u > 0:
                    probe = Vault.__new__(Vault)
                    probe.__dict__ = dict(v.__dict__)
                    total = probe.mint(u)
                    if total > U64 or probe.S > U64 or max(probe.k, probe.R + probe.Q) > U128:
                        raise Err("Overflow|Slippage" if total > lim else "Overflow")
                    # prodotti intermedi di state.rs in u128: u·k·e, u·k
                    if u * v.k * v.e > U128 or u * probe.k > U128:
                        raise Err("Overflow")
                res = w.mint(u, lim)
                tail = f"ok {res} {w.fc - fc0} {w.fp - fp0}"
            elif kind == "R":
                if 0 < u <= v.S:
                    if u * v.k * v.p > U128:
                        raise Err("Overflow")
                    probe = Vault.__new__(Vault)
                    probe.__dict__ = dict(v.__dict__)
                    try:
                        probe.redeem(u, 0)
                        if b0 - probe.bal > U64:
                            raise Err("Overflow")
                    except Err as ex:
                        if ex.args[0] == "Overflow":
                            raise
                res = w.redeem(u, lim)
                tail = f"ok {res} {w.fc - fc0} {w.fp - fp0}"
            else:
                if u * SC + w.Q > U128:
                    raise Err("Overflow")
                w.donate(u)
                if w.k > U128 or w.R > U128:
                    raise Err("Overflow")
                tail = "ok"
            fin = w
        except Err as ex:
            tail = f"err {ex.args[0]}"
            fin = v
        return f"{kind} {u} {lim} {tail} {fin.k} {fin.R} {fin.Q} {fin.S}" if kind != "D" else \
            f"D {u} {tail} {fin.k} {fin.R} {fin.Q} {fin.S}"

    if op in ("M", "R", "D"):
        lines.append(attempt(op, u, U64 if op == "M" else 0))
    elif op == "Mx":
        probe = Vault.__new__(Vault)
        probe.__dict__ = dict(v.__dict__)
        try:
            need = probe.mint(u)
        except Err:
            return lines
        if need > U64:
            return lines
        for lim in (need - 1, need, need + 1):
            if 0 <= lim <= U64:
                lines.append(attempt("M", u, lim))
    else:
        probe = Vault.__new__(Vault)
        probe.__dict__ = dict(v.__dict__)
        try:
            out = probe.redeem(u)
        except Err:
            return lines
        for lim in (max(out - 1, 0), out, out + 1):
            lines.append(attempt("R", u, lim))
    return lines


def sol():
    SC = 10**9
    rows = []
    prices = (10**6, 10**6 + 1, 10**15, 10**15 - 1, 10**20, 1 << 100)
    for (k, S, Q) in canonical_states(SC, prices):
        if S * k + Q > U128 or S > U64:
            continue  # stato non rappresentabile nel vault Solana (R, Q in u128)
        for p, e in ((100, 0), (1000, 1000), (200, 100)):
            v = vault(k, S, Q, p, e, SC)
            for op, u, _ in op_cases(v, "sol"):
                for line in run_sol(v, op, u, SC):
                    rows.append(f"Z {k} {S * k} {Q} {S} {p} {e}")
                    rows.append(line)
    path = os.path.join(ROOT, "vectors", "boundary_sol.txt")
    with open(path, "w") as f:
        f.write("# generato da tools/redteam/boundary.py: non modificare a mano\n")
        f.write("\n".join(rows) + "\n")
    return len(rows) // 2


def evm():
    SC = 10**18
    cols = {k: [] for k in ["k0", "R0", "Q0", "S0", "p", "e", "op", "u", "lim", "ok", "err", "res", "fc", "fp",
                            "k", "R", "Q", "S"]}
    prices = (10**12, 10**12 + 1, 10**24, 10**24 - 1, 1 << 200)
    n = 0
    for (k, S, Q) in canonical_states(SC, prices):
        if S * k + Q > U256:
            continue  # stato non rappresentabile in uint256
        for p, e in ((100, 0), (1000, 1000), (200, 100)):
            v = vault(k, S, Q, p, e, SC)
            amounts = sorted({0, 1, 2, max(S - 1, 0), S, S + 1, 10**30, 1 << 200, U256})
            cases = [("M", u, U256) for u in amounts] + [("R", u, 0) for u in amounts] + \
                    [("D", a, 0) for a in (0, 1, 2, 10**30, 1 << 200)]
            for op, u, lim in cases:
                w = Vault.__new__(Vault)
                w.__dict__ = dict(v.__dict__)
                fc0, fp0 = w.fc, w.fp
                try:
                    if op == "M":
                        res = w.mint(u, lim)
                    elif op == "R":
                        res = w.redeem(u, lim)
                    else:
                        w.donate(u)
                        res = 0
                    # uint256: qualunque intermedio oltre 2²⁵⁶ − 1 è Panic(0x11) (§13)
                    if peak_of(v, op, u, lim) > U256:
                        raise Err("Overflow")
                    ok, err, fin = 1, "", w
                except Err as ex:
                    ok, err, fin, res = 0, ex.args[0], v, 0
                    # un errore del modello con intermedi oltre 256 bit: sull'EVM può arrivare
                    # prima il Panic dell'aritmetica checked; entrambi ammessi
                    if err != "Overflow" and peak_of(v, op, u, lim) > U256:
                        err = "Overflow|" + err
                    fc0, fp0 = w.fc, w.fp = v.fc, v.fp
                for key, val in (("k0", k), ("R0", S * k), ("Q0", Q), ("S0", S), ("p", p), ("e", e),
                                 ("op", {"M": 1, "R": 2, "D": 3}[op]), ("u", u), ("lim", lim), ("ok", ok),
                                 ("err", err), ("res", res), ("fc", fin.fc - fc0 if ok else 0),
                                 ("fp", fin.fp - fp0 if ok else 0), ("k", fin.k), ("R", fin.R), ("Q", fin.Q),
                                 ("S", fin.S)):
                    cols[key].append(val if key == "err" else str(val))
                n += 1
    with open(os.path.join(ROOT, "vectors", "boundary_evm.json"), "w") as f:
        json.dump(cols, f, separators=(",", ":"))
    return n


if __name__ == "__main__":
    files = [os.path.join(ROOT, "vectors", f) for f in ("boundary_sol.txt", "boundary_evm.json")]
    if "--check" in sys.argv:
        old = [open(f).read() for f in files]
    print(f"sol: {sol()} casi; evm: {evm()} casi", file=sys.stderr)
    if "--check" in sys.argv:
        new = [open(f).read() for f in files]
        for f, o, n in zip(files, old, new):
            if o != n:
                open(f, "w").write(o)
                raise SystemExit(f"{f} non aggiornato: eseguire python3 tools/redteam/boundary.py")
        print("vettori di confine aggiornati", file=sys.stderr)
