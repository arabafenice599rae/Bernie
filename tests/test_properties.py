"""Proprietà economiche e aritmetiche di §5, §8 e §17 sul modello di riferimento."""
import copy
import unittest

from common import BPS, FEE_C, FEE_P, SCALES, Err, Vault, cdiv, n, random_state, rng, split_fees

FEE = FEE_C + FEE_P

# Massimo osservato: 3,0 unità native su ~36.000 scenari con SCALE 10 (0 sforamenti con 10⁹ e 10¹⁸).
P6C_EPS_HOST = 3


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

    def test_p6_capture(self):
        """P6a vittima indenne, P6b nessuna perdita di backing, P6c cattura ≤ penalità, P6d soglia."""
        checked = {"a": 0, "d": 0}
        for SCALE in SCALES:
            r = rng(f"p6:{SCALE}")
            for _ in range(n(3000)):
                O = r.choice([1, r.randint(1, 50), r.randint(1, 10**6)])
                V = r.choice([1, r.randint(1, 50), r.randint(1, 10**6)])
                A = r.choice([1, r.randint(1, 50), r.randint(1, 10**7)])
                if SCALE > 10 and r.random() < 0.3:
                    # k piccoli anche sulle SCALE di produzione: arrotondamenti più pesanti.
                    v = random_state(r, SCALE, k=r.randint(1, 100 * SCALE), S=O + V)
                else:
                    v = random_state(r, SCALE, S=O + V)
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
                # P6c. Con SCALE 10 e k < SCALE i resti di arrotondamento (ceil del mint
                # dell'attaccante, resto del redeem della vittima) pesano fino a 3 unità
                # native oltre la penalità: tolleranza P6C_EPS_HOST, vedi tests/README.md.
                eps = P6C_EPS_HOST if SCALE == 10 else 0
                self.assertLessEqual(profit, cdiv(pen_v, SCALE) + eps, "P6c")
                if v.p * V <= (v.e + FEE) * O:
                    checked["d"] += 1
                    self.assertLessEqual(profit, 1, "P6d")
        self.assertGreater(checked["d"], 100)

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
        # Il modello dà +3,885%: il README riporta +3,89%, ricavato dal 3,74% già arrotondato.
        self.assertAlmostEqual((1 / (1 - rt) - 1) * 100, 3.89, delta=0.006)
        self.assertAlmostEqual(self.v_star(200, 100) * 100, 41, delta=0.5)

    def test_split_matters(self):
        self.assertAlmostEqual(self.v_star(300, 0) * 100, 12, delta=0.5)
        self.assertAlmostEqual(self.v_star(200, 100) * 100, 41, delta=0.5)
        self.assertAlmostEqual(self.v_star(150, 150) * 100, 56, delta=0.5)

    def test_simulation_table(self):
        rows = [  # p, e, round-trip %, v* %, k/anno τ=1%, k/anno τ=5%
            (100, 50, 2.28, 47, 2.8, 14.6),
            (200, 100, 3.74, 41, 5.7, 31),
            (300, 150, 5.19, 39, 8.6, 51),
            (500, 250, 8.05, 37, 14.7, 98),
        ]
        for p, e, rt, vs, k1, k5 in rows:
            with self.subTest(p=p, e=e):
                self.assertAlmostEqual(self.round_trip(p, e) * 100, rt, delta=0.005)
                self.assertAlmostEqual(self.v_star(p, e) * 100, vs, delta=0.5)
                # Crescita di k ≈ (e + p)·τ/2 al giorno, composta su 365 giorni (limite ottimistico).
                for tau, want in ((0.01, k1), (0.05, k5)):
                    g = ((1 + (e + p) / BPS * tau / 2) ** 365 - 1) * 100
                    # Tabella di simulazione: tolleranza 2% relativo (min 0,1 punti).
                    # Nota: per p = 2%, e = 1%, τ = 1% la formula dà +5,63%, il README +5,7%.
                    self.assertAlmostEqual(g, want, delta=max(0.1, want * 0.02))


if __name__ == "__main__":
    unittest.main()
