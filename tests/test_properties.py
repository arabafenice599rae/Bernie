"""Proprietà economiche e aritmetiche di §5, §8 e §17 sul modello di riferimento."""
import copy
import unittest

from common import (BPS, FEE_C, FEE_P, PRICE_RANGE, SCALES, Err, Vault, cdiv, log_uniform, n,
                    random_state, rng, split_fees)

FEE = FEE_C + FEE_P



def eps_c(V, SCALE):
    """P6c: quota del residuo preesistente (Q₀ < S₀) più due arrotondamenti, in unità native."""
    return V // SCALE + 2


def eps_d(O, V, A, SCALE):
    """P6d: quota pro-rata dell'attaccante di un residuo pieno (O + V sotto-unità), più 1.

    Vale 1 se A·(O + V) < (O + A)·SCALE; al massimo 1/k della posizione dell'attaccante.
    """
    return A * (O + V) // ((O + A) * SCALE) + 1


def clone(v):
    return copy.deepcopy(v)


class Arithmetic(unittest.TestCase):
    def test_fee_split(self):
        """§17: ft(base+1) − ft(base) ∈ {0, 1} e fc + fp == ft."""
        prev = split_fees(0)[0]
        for base in range(1, 200_000):
            ft, fc, fp = split_fees(base)
            self.assertIn(ft - prev, (0, 1))
            self.assertEqual(fc + fp, ft)
            self.assertGreaterEqual(fc, fp)
            prev = ft
        r = rng("fees")
        for _ in range(n(20_000)):
            base = r.getrandbits(r.randint(1, 200))
            ft, fc, fp = split_fees(base)
            self.assertEqual(ft, base * FEE // BPS)
            self.assertEqual(fc + fp, ft)
            self.assertIn(split_fees(base + 1)[0] - ft, (0, 1))

    def test_p7_out_monotone(self):
        """P7: out = g − ft(g) non decrescente, incrementi 0 o 1."""
        prev = 0
        for g in range(1, 10**6):
            out = g - split_fees(g)[0]
            self.assertIn(out - prev, (0, 1))
            prev = out

    def test_mint_bounds_and_monotone_cost(self):
        """§17: c·SCALE ≥ full + epen, b ≤ c; costo del mint non decrescente in u (P7)."""
        for SCALE in SCALES:
            r = rng(f"mintcost:{SCALE}")
            for _ in range(n(300)):
                v = random_state(r, SCALE)
                u0 = r.randint(1, 10**5)
                costs = []
                for u in range(u0, u0 + 20):
                    w = clone(v)
                    k_prev = w.k
                    epen = cdiv(u * w.k * w.e, BPS)
                    w.Q += epen
                    w.absorb()
                    full = u * w.k
                    c = cdiv(full + epen, SCALE)
                    b = cdiv(full, SCALE)
                    self.assertGreaterEqual(c * SCALE, full + epen)
                    self.assertLessEqual(b, c)
                    self.assertGreaterEqual(w.k, k_prev)
                    costs.append(clone(v).mint(u))
                self.assertEqual(costs, sorted(costs))

    def test_redeem_bounds(self):
        """§17: nel redeem mai out > g né g·SCALE > full − pen; stato canonico o errore."""
        for SCALE in SCALES:
            r = rng(f"redeem:{SCALE}")
            for _ in range(n(3000)):
                v = random_state(r, SCALE)
                u = r.randint(1, v.S)
                full = u * v.k
                pen = cdiv(full * v.p, BPS)
                w = clone(v)
                try:
                    out = w.redeem(u)
                except Err as err:
                    self.assertIn(str(err), ("Dust", "ZeroPayout"))
                    continue
                g = (full - pen) // SCALE
                self.assertLessEqual(out, g)
                self.assertLessEqual(g * SCALE, full - pen)
                self.assertTrue(w.S == 0 and w.Q == 0 and w.R == 0 or w.Q < w.S)

    def test_absorb(self):
        """§17: absorb conserva R + Q e I1, e lascia Q < S."""
        for SCALE in SCALES:
            r = rng(f"absorb:{SCALE}")
            for _ in range(n(3000)):
                v = random_state(r, SCALE)
                v.Q += r.getrandbits(r.randint(1, 90))
                before = v.R + v.Q
                v.absorb()
                self.assertEqual(v.R + v.Q, before)
                self.assertEqual(v.R, v.S * v.k)
                self.assertLess(v.Q, v.S)


class Economics(unittest.TestCase):
    def test_p1_round_trip_loses(self):
        """P1: sequenze di mint e redeem dello stesso attore restituiscono meno del versato."""
        for SCALE in SCALES:
            r = rng(f"p1:{SCALE}")
            for _ in range(n(1000)):
                v = random_state(r, SCALE)
                paid = got = held = 0
                for _ in range(r.randint(1, 6)):
                    try:
                        if held == 0 or r.random() < 0.5:
                            u = r.randint(1, 10**6)
                            paid += v.mint(u)
                            held += u
                        else:
                            u = r.randint(1, held)
                            got += v.redeem(u)
                            held -= u
                    except Err:
                        pass
                if held:
                    try:
                        got += v.redeem(held)
                    except Err:
                        pass  # posizione residua non riscattabile: perdita ancora più netta
                if paid:
                    self.assertLess(got, paid)

    def test_p5_no_entry_self_refund(self):
        """P5: il nuovo entrante non riceve nulla della propria epen."""
        for SCALE in SCALES:
            r = rng(f"p5:{SCALE}")
            for _ in range(n(1000)):
                v = random_state(r, SCALE)
                u = r.randint(1, 10**6)
                k_old, S_old = v.k, v.S
                epen = cdiv(u * k_old * v.e, BPS)
                # Ciò che gli holder esistenti ricevono se l'epen è assorbita solo da loro.
                ref = clone(v)
                ref.Q += epen
                ref.absorb()
                v.mint(u)
                # Gli holder esistenti prendono almeno quanto con l'epen assorbita solo da loro...
                self.assertGreaterEqual(v.k, ref.k)
                # ...e il nuovo entrante non supera il backing che ha pagato, al netto di epen.
                c = cdiv(u * ref.k + epen, SCALE)
                self.assertLessEqual(u * v.k, c * SCALE - epen)
                self.assertEqual(v.S, S_old + u)

    def _attack(self, v, V, A):
        """Scenario P6: stesso stato iniziale, redeem V con e senza mint A dell'attaccante."""
        base = clone(v)
        out_plain = base.redeem(V)
        k_plain = base.k
        att = clone(v)
        cost = att.mint(A)
        out_att = att.redeem(V)
        return out_plain, k_plain, out_att, att, cost

    @staticmethod
    def _sizes(r, mode):
        """O, V, A: piccoli, fino a 10⁶, fino a 10¹⁸ (supply reali) o con P6d al limite."""
        if mode == "small":
            return r.randint(1, 50), r.randint(1, 50), r.randint(1, 50)
        if mode == "mid":
            return r.randint(1, 10**6), r.randint(1, 10**6), r.randint(1, 10**7)
        V = log_uniform(r, 1, 10**15)
        A = log_uniform(r, 1, 10**17)
        if mode == "big":
            return log_uniform(r, 1, 10**18), V, A
        return None, V, A  # "tight": O calcolato da p ed e

    def test_p6_capture(self):
        """P6a vittima indenne, P6b nessuna perdita di backing, P6c e P6d con ε_c ed ε_d."""
        checked = {"a": 0, "d": 0}
        for SCALE in SCALES:
            r = rng(f"p6:{SCALE}")
            for i in range(n(4000)):
                mode = ("small", "mid", "big", "tight")[i % 4]
                O, V, A = self._sizes(r, mode)
                p = r.randint(100, 1000)
                e = r.choice([0, p, r.randint(0, p)])
                if O is None:
                    O = cdiv(p * V, e + FEE) + r.choice([0, r.randint(0, V)])
                lo, hi = PRICE_RANGE[SCALE]
                # Anche k sotto MIN_PRICE: i limiti valgono per ogni k.
                k = log_uniform(r, 1, hi) if r.random() < 0.3 else None
                v = random_state(r, SCALE, k=k, S=O + V, p=p, e=e,
                                 full_residual=r.random() < 0.7)
                k_before = v.k
                try:
                    out_plain, k_plain, out_att, att, cost = self._attack(v, V, A)
                except Err:
                    continue
                checked["a"] += 1
                self.assertGreaterEqual(out_att, out_plain, "P6a")
                self.assertGreaterEqual(O * att.k, O * k_before, "P6b")
                profit = A * att.k // SCALE - cost  # mark-to-k
                pen_v = cdiv(V * k_before * v.p, BPS)
                ctx = f"SCALE={SCALE} O={O} V={V} A={A} p={p} e={e} k={k_before}"
                self.assertLessEqual(profit * SCALE, pen_v + eps_c(V, SCALE) * SCALE, "P6c " + ctx)
                if v.p * V <= (v.e + FEE) * O:
                    checked["d"] += 1
                    self.assertLessEqual(profit, eps_d(O, V, A, SCALE), "P6d " + ctx)
        self.assertGreater(checked["d"], n(2000))

    def test_example_section_8(self):
        """Esempio verificato di §8 (v1.6): p = 500, e = 460, O = 5, V = 20, A = 5."""
        for SCALE in SCALES:
            def state():
                v = Vault(10**6, 500, 460, SCALE)
                v.k = 10**6 * SCALE
                v.S = 25
                v.R = 25 * v.k
                v.bal = v.R // SCALE
                return v
            plain = state()
            self.assertEqual(plain.value(5), 5_000_000)
            self.assertEqual(plain.redeem(20), 18_924_000)
            self.assertEqual(plain.value(5), 6_000_000)
            att = state()
            cost = att.mint(5)
            self.assertEqual(att.redeem(20), 19_098_101)
            self.assertEqual(att.value(5), 5_550_600)
            self.assertEqual(att.value(5) - cost, 254_416)


class Parameters(unittest.TestCase):
    """Tabelle di §9: round-trip, pareggio, v*."""

    @staticmethod
    def round_trip(p, e):
        # Mint e redeem immediati con molti altri holder (k praticamente fermo).
        v = Vault(10**15, p, e, 10**9)
        v.k = 10**15
        v.S = 10**12
        v.R = v.S * v.k
        v.bal = v.R // 10**9
        u = 10**6
        paid = v.mint(u)
        got = v.redeem(u)
        return 1 - got / paid

    @staticmethod
    def v_star(p, e):
        r = (e + FEE) / p
        return r / (1 + r)

    def test_reference(self):
        rt = self.round_trip(200, 100)
        self.assertAlmostEqual(rt * 100, 3.74, delta=0.005)
        self.assertEqual(round((1 / (1 - rt) - 1) * 100, 2), 3.88)
        self.assertAlmostEqual(self.v_star(200, 100) * 100, 41, delta=0.5)

    def test_split_matters(self):
        self.assertAlmostEqual(self.v_star(300, 0) * 100, 12, delta=0.5)
        self.assertAlmostEqual(self.v_star(200, 100) * 100, 41, delta=0.5)
        self.assertAlmostEqual(self.v_star(150, 150) * 100, 56, delta=0.5)

    def test_formula_table(self):
        rows = [  # p, e, round-trip %, v* %, k/anno τ=1%, k/anno τ=5%
            (100, 50, 2.28, 47, 2.78, 14.67),
            (200, 100, 3.74, 41, 5.63, 31.48),
            (300, 150, 5.19, 39, 8.56, 50.74),
            (500, 250, 8.05, 37, 14.67, 98.13),
        ]
        for p, e, rt, vs, k1, k5 in rows:
            with self.subTest(p=p, e=e):
                self.assertEqual(round(self.round_trip(p, e) * 100, 2), rt)
                self.assertEqual(round(self.v_star(p, e) * 100), vs)
                # Formula di §9: (1 + (e + p)·τ/2)^365 − 1.
                for tau, want in ((0.01, k1), (0.05, k5)):
                    g = ((1 + (e + p) / BPS * tau / 2) ** 365 - 1) * 100
                    self.assertEqual(round(g, 2), want)


if __name__ == "__main__":
    unittest.main()
