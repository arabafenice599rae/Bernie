# Test e verifica di Bernie

Tutto parte dal modello di riferimento dell'Appendice A, copiato in `model/bernie.py`
(un test fallisce se le due copie divergono).

| Cosa | Comando | Dove gira |
|---|---|---|
| Modello: fuzz multi-attore, P1–P7, §8, §9 | `python3 -m unittest discover -s tests` | locale, CI |
| Fuzz lungo | `BERNIE_FUZZ=20 BERNIE_SEED=42 python3 -m unittest discover -s tests` | CI notturna |
| Vettori condivisi | `python3 tools/gen_vectors.py` (`--check` in CI) | locale, CI |
| Appendice B | `python3 tools/abi_check.py` (usa solc se presente, o `$SOLC`) | locale, CI |
| `state.rs`: vettori, esaustivo SCALE 10, parametri | `cd solana && cargo test` | locale, CI |
| Kani su `state.rs` | `cd solana/state && cargo kani` | CI |
| Interfaccia EVM | `cd evm && forge build` | CI |

Solo libreria standard Python; `tools/keccak.py` è un Keccak-256 in Python puro.

## Vettori

`vectors/scale_10.json` (host), `scale_1e9.json` (Solana), `scale_1e18.json` (EVM).
Ogni caso parte da `create(P, p, e)`; ogni passo registra esito e stato completo
(`k`, `R`, `Q`, `S`, saldo del vault senza fee, fee cumulative). Interi come stringhe.
`state.rs` li rigioca per SCALE 10 e 10⁹; i vettori 10¹⁸ servono all'EVM.

## Limiti di cattura

P6c e P6d usano gli ε di §8: `ε_c = ⌊V/SCALE⌋ + 2` e `ε_d = ⌊A·(O+V)/((O+A)·SCALE)⌋ + 1`.
Il test genera supply piccole, medie e fino a 10¹⁸, residuo `Q₀` massimo, condizione di P6d
al limite e k anche sotto `MIN_PRICE`. Con ε = 1 al posto di ε_d, o senza il termine `⌊V/SCALE⌋`
in ε_c, il test fallisce.
