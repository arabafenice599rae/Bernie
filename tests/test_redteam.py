"""Strumenti del pre-audit (tools/redteam): determinismo del generatore e replay dopo la
minimizzazione. Il replay deve dare un esito per ogni sottosequenza, mai un'eccezione."""
import os
import random
import sys
import unittest

from common import ROOT

sys.path.insert(0, os.path.join(ROOT, "tools", "redteam"))
import gen  # noqa: E402
from oracle import NATIVE, Market  # noqa: E402


class RedTeamTools(unittest.TestCase):
    def test_same_seed_same_sequence(self):
        for chain in ("sol", "evm"):
            a = gen.sequence(7, chain, 120, 5)
            b = gen.sequence(7, chain, 120, 5)
            self.assertEqual(gen.stringify(a), gen.stringify(b), chain)
            self.assertNotEqual(gen.stringify(a), gen.stringify(gen.sequence(8, chain, 120, 5)), chain)

    def test_replay_matches_generation(self):
        for chain in ("sol", "evm"):
            seq = gen.sequence(3, chain, 150, 4)
            self.assertEqual(gen.stringify(gen.replay(seq, seq["steps"])), gen.stringify(seq), chain)

    def test_replay_of_any_subsequence(self):
        # regressione: togliendo passi, un mint EVM poteva inviare msg.value oltre il saldo
        # e l'oracolo finiva con un saldo negativo invece di InsufficientNative
        r = random.Random(11)
        for chain in ("sol", "evm"):
            for seed in range(4):
                seq = gen.sequence(seed, chain, 80, 4)
                for _ in range(25):
                    keep = [s for s in seq["steps"] if r.random() < 0.5]
                    gen.replay(seq, keep)

    def test_evm_value_above_balance(self):
        m = Market("evm", 10**15, 10**15, 5000, [10**18, 10**18])
        self.assertEqual(m.mint(1, 1, 10**18 + 1), {"err": [NATIVE]})
        self.assertEqual(m.donate(1, 10**18 + 1), {"err": [NATIVE]})
        self.assertEqual(m.native[1], 10**18)


if __name__ == "__main__":
    unittest.main()
