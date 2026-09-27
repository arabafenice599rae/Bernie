"""Il modello eseguibile deve coincidere con l'Appendice A del README."""
import os
import re
import unittest

from common import ROOT


def appendix_a(readme):
    m = re.search(r"## Appendice A[^\n]*\n+```python\n(.*?)```", readme, re.S)
    assert m, "Appendice A non trovata"
    return m.group(1)


class SpecSync(unittest.TestCase):
    def test_model_matches_appendix_a(self):
        with open(os.path.join(ROOT, "README.md"), encoding="utf-8") as f:
            readme = f.read()
        with open(os.path.join(ROOT, "model", "bernie.py"), encoding="utf-8") as f:
            model = f.read()
        self.assertEqual(model, appendix_a(readme),
                         "model/bernie.py diverge dall'Appendice A: aggiornare entrambi")

    def test_constants_match_section_20(self):
        from common import FEE_C, FEE_P, BPS
        self.assertEqual((FEE_C, FEE_P, BPS), (20, 20, 10_000))


if __name__ == "__main__":
    unittest.main()
