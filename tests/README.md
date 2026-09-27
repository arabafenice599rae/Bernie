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

## Tolleranze e scostamenti noti

- **P6c con SCALE 10.** Con k < SCALE il profitto mark-to-k dell'attaccante può superare
  la penalità della vittima fino a 3 unità native (ceil del proprio mint più resto del
  redeem della vittima). Con SCALE 10⁹ e 10¹⁸ lo sforamento resta sotto 1 unità, quindi
  entro `⌈pen/SCALE⌉`. Il test usa `P6C_EPS_HOST = 3` solo per SCALE 10.
- **§9, pareggio.** Il modello dà +3,885% (il README riporta +3,89%, derivato dal 3,74% arrotondato).
- **§9, tabella.** Per p = 2%, e = 1%, τ = 1%/g la formula `(e + p)·τ/2` composta dà +5,63%/anno;
  il README riporta +5,7%. Il test accetta uno scarto del 2%.
