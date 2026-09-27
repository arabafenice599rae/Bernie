"""Supporto condiviso dai test: import del modello e generatori di stati casuali."""
import os
import random
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "model"))
sys.path.insert(0, os.path.join(ROOT, "tools"))

import bernie  # noqa: E402
from bernie import BPS, FEE_C, FEE_P, Err, Ledger, Vault, cdiv, split_fees  # noqa: E402,F401

SCALES = (10, 10**9, 10**18)

# Intervalli di prezzo per SCALE: host (SCALE 10), Solana, EVM (§20).
PRICE_RANGE = {10: (1, 10**4), 10**9: (10**6, 10**15), 10**18: (10**12, 10**24)}

# Moltiplicatore del numero di casi: BERNIE_FUZZ=10 per le esecuzioni lunghe.
FUZZ = float(os.environ.get("BERNIE_FUZZ", "1"))


def n(base):
    return max(1, int(base * FUZZ))


def rng(tag):
    seed = os.environ.get("BERNIE_SEED", "1")
    return random.Random(f"{seed}:{tag}")


def log_uniform(r, lo, hi):
    """Intero tra lo e hi con distribuzione log-uniforme (copre tutti gli ordini di grandezza)."""
    if lo == hi:
        return lo
    a, b = lo.bit_length(), hi.bit_length()
    while True:
        x = r.getrandbits(r.randint(a, b)) | 1
        if lo <= x <= hi:
            return x


def random_params(r, SCALE):
    lo, hi = PRICE_RANGE[SCALE]
    P = log_uniform(r, lo, hi)
    p = r.randint(100, 1000)
    e = r.choice([0, p, r.randint(0, p)])
    return P, p, e


def random_state(r, SCALE, k=None, S=None, p=None, e=None, full_residual=False):
    """Stato canonico (I1–I6) con saldo esatto: k, S, Q < S arbitrari.

    full_residual: Q il più vicino possibile a S (caso peggiore per P6c e P6d).
    """
    P, p0, e0 = random_params(r, SCALE)
    p = p0 if p is None else p
    e = e0 if e is None else e
    v = Vault(P, p, e, SCALE)
    v.k = P if k is None else k
    v.S = r.randint(1, 10**6) if S is None else S
    v.R = v.S * v.k
    # Residuo che rende (R + Q) multiplo di SCALE (I5), più qualche SCALE intero;
    # l'absorb lo riporta sotto S (I6) spostando k di poco.
    base = (-v.R) % SCALE
    if full_residual and base < v.S:
        v.Q = base + (v.S - 1 - base) // SCALE * SCALE
    else:
        v.Q = base + r.randint(0, 3) * SCALE
    v.absorb()
    v.bal = (v.R + v.Q) // SCALE
    v.inv(v.k)
    return v
