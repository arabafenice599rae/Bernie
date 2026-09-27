"""Fuzz multi-attore (§16): I1–I6, P3, excess ≥ 0, P2, sweep su entrambe le chain."""
import unittest

from common import SCALES, Err, Ledger, Vault, n, random_params, rng

ACTORS = ("a", "b", "c", "d")
PASSIVE = "passivo"
CREATOR = "creator"


class Chain:
    """Ledger del modello più lo sweep v1.6.

    Il creator è un wallet del Ledger; `v.fc` conta le fee del creator non ancora
    uscite dal contratto, così il controllo P3 dell'Appendice A resta valido.
    Solana: la fee del creator resta nel vault; lamport del vault (senza rent) =
    bal + fc; sweep = lamport − (R + Q)/SCALE.
    EVM: le fee sono in feesOwed; sweep = saldo − (R + Q)/SCALE − totalFeesOwed = excess().
    """

    def __init__(self, chain, P, p, e, SCALE, wallet):
        self.chain = chain
        wallets = {a: wallet for a in ACTORS + (PASSIVE,)}
        wallets[CREATOR] = 0
        self.led = Ledger(Vault(P, p, e, SCALE), wallets)

    @property
    def v(self):
        return self.led.v

    def vault_native(self):
        v = self.v
        return v.bal + (v.fc if self.chain == "solana" else 0)

    def sweep(self):
        v = self.v
        amt = v.excess() + (v.fc if self.chain == "solana" else 0)
        if amt <= 0:
            raise Err("NothingToClaim")
        k0, state0 = v.k, (v.k, v.R, v.Q, v.S)
        v.bal -= v.excess()
        if self.chain == "solana":
            v.fc = 0
        self.led.w[CREATOR] += amt
        v.inv(k0)
        assert (v.k, v.R, v.Q, v.S) == state0, "sweep non deve toccare k, R, Q, S"
        assert v.excess() == 0

    def claim_fees(self):
        # Solo EVM: il creator ritira feesOwed[creator].
        amt = self.v.fc
        if amt <= 0:
            raise Err("NothingToClaim")
        self.v.fc = 0
        self.led.w[CREATOR] += amt

    def check(self):
        v = self.v
        self.led.check()  # P3 ed excess ≥ 0 come nell'Appendice A
        # I2 in unità native reali: il vault copre sempre (R + Q)/SCALE.
        assert self.vault_native() * v.SC >= v.R + v.Q


def run(chain, SCALE, r, ops):
    P, p, e = random_params(r, SCALE)
    wallet = P * 10**7
    c = Chain(chain, P, p, e, SCALE, wallet)
    led = c.led
    c.led.mint(PASSIVE, r.randint(1, 10**6))
    passive_units = led.tok[PASSIVE]
    last_value = led.value(PASSIVE)
    stats = {"ok": 0, "err": 0}
    for _ in range(ops):
        a = r.choice(ACTORS)
        op = r.random()
        try:
            if op < 0.40:
                u = r.choice([1, r.randint(1, 1000), r.randint(1, 10**6)])
                led.mint(a, u)
            elif op < 0.80:
                if led.tok[a] == 0:
                    continue
                u = r.choice([led.tok[a], r.randint(1, led.tok[a])])
                led.redeem(a, u)
            elif op < 0.90:
                led.donate(a, r.randint(1, max(1, P // 10)))
            elif op < 0.96:
                c.sweep()
            elif chain == "evm":
                c.claim_fees()
            else:
                continue
            stats["ok"] += 1
        except Err:
            stats["err"] += 1
        c.check()
        # P2: l'holder passivo non vede mai scendere il valore di backing.
        assert led.tok[PASSIVE] == passive_units
        value = led.value(PASSIVE)
        assert value >= last_value, "P2 violata"
        last_value = value
    return stats


class Fuzz(unittest.TestCase):
    def _run(self, chain):
        for SCALE in SCALES:
            r = rng(f"fuzz:{chain}:{SCALE}")
            total = {"ok": 0, "err": 0}
            for _ in range(n(60)):
                s = run(chain, SCALE, r, n(300))
                total["ok"] += s["ok"]
                total["err"] += s["err"]
            with self.subTest(SCALE=SCALE):
                self.assertGreater(total["ok"], total["err"], "troppe operazioni rifiutate")

    def test_solana(self):
        self._run("solana")

    def test_evm(self):
        self._run("evm")

    def test_last_exit_reopen(self):
        """Ultimo uscente: penalità in excess, sweep, rientro al k raggiunto (I4, §7)."""
        for SCALE in SCALES:
            r = rng(f"reopen:{SCALE}")
            for _ in range(n(200)):
                P, p, e = random_params(r, SCALE)
                c = Chain("solana", P, p, e, SCALE, P * 10**7)
                led = c.led
                led.mint("a", r.randint(1, 10**5))
                led.mint("b", r.randint(1, 10**5))
                try:
                    led.redeem("a", led.tok["a"])
                    led.redeem("b", led.tok["b"])
                except Err:  # Dust o ZeroPayout con prezzi minuscoli (SCALE 10)
                    continue
                v = led.v
                self.assertEqual((v.S, v.Q, v.R), (0, 0, 0))
                k_end = v.k
                if c.vault_native() > 0:
                    c.sweep()
                self.assertEqual(c.vault_native(), 0)
                led.mint("c", r.randint(1, 10**5))
                self.assertGreaterEqual(led.v.k, k_end)
                c.check()


if __name__ == "__main__":
    unittest.main()
