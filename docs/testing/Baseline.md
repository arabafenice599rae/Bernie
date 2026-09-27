# Baseline (fase 0)

Stato di partenza della tranche red-team, prima di qualunque test nuovo. Commit di
riferimento: `4583bf5` (merge della PR #6, prima di ogni test del red team). Macchina: container Linux
4 vCPU, 15 GB RAM. Nessuna modifica alla logica di Bernie tra baseline e report.

## Versioni

| Strumento | Versione |
|---|---|
| rustc / cargo | 1.98.1 |
| Agave (solana-cli) | 4.2.2 |
| cargo-build-sbf / platform-tools | 4.1.0 / v1.54 |
| pinocchio | 0.11.2 |
| Mollusk (mollusk-svm) | 0.15.1 |
| Foundry (forge) | 1.8.3-dev |
| solc | 0.8.30 |
| Node | 22.22.2 |
| Python | 3.11.15 |
| Kani / CBMC | 0.68.0 / 6.11.0 |
| Halmos | 0.3.3 |
| cargo-mutants | 27.1.0 |

## Suite esistenti

| Comando | Esito | Test | Durata |
|---|---|---|---|
| `cargo fmt --all --check` (solana) | ok | — | < 1 s |
| `cargo clippy --all-targets -- -D warnings` (solana) | ok, 0 warning | — | 1 s (cache) |
| `cargo test -p bernie-state --release` | ok | 8 (exhaustive 2, params 4, vectors 2) | < 1 s |
| `cargo build-sbf` | ok | — | < 1 s (cache) |
| `cargo test -p bernie-program --release` (Mollusk) | ok | 12 (lib 1, frontend 2, mollusk 9) | 1,1 s |
| `forge test` (evm) | ok | 17 (Bernie 16, Vectors 1) | 0,3 s |
| `python -m unittest discover -s tests` | ok | 17 + 10 subtest | 3,2 s |
| `python tools/gen_vectors.py --check` | ok, vettori allineati al modello | — | < 5 s |
| `SOLC=./solc python tools/abi_check.py` | ok (21 selettori, 5 topic, 5 ERC-20; frontend 25 + 5) | — | < 5 s |
| `node emit.mjs --check` (frontend/test) | ok, fixture aggiornate | — | < 5 s |
| `python tools/keccak.py` | ok | — | < 1 s |
| `gitleaks git` | 24 commit, nessun segreto | — | 1 s |

## Verifica formale

| Harness | Esito | Durata |
|---|---|---|
| Kani `absorb_conserves_and_canonicalizes` | VERIFIED | 115 s |
| Kani `mint_canonical_or_error` | VERIFIED | 2 191 s |
| Kani `redeem_canonical_or_error` | VERIFIED | 34 s |
| Kani `donate_canonical_or_error` | VERIFIED | 5 s |
| Kani `fees_step_and_split` | VERIFIED | 164 s |
| Halmos (9 `check_*` su `BernieMathHalmos`) | vedi sotto | |

HALMOS_TABLE

## Warning e note

- `forge build` segnala lint informativi su `BernieFactory`/`Bernie` (missing-zero-check sul
  costruttore della factory, calls-loop in `claimAll`, eventi dopo chiamate esterne). Sono
  scelte documentate (treasury fissata al deploy, `claimAll` limitato dal chiamante, guard
  di reentrancy); nessuno cambia il comportamento. Restano come nota nel report.
- Node: DeprecationWarning `punycode` da una dipendenza, non dal codice del repo.
- `gitleaks dir` (non `git`) segnala `keys/` e `target/`: entrambi ignorati da git, mai committati.
- Nessun test saltato o ignorato nella baseline.
- Primo tentativo Halmos locale fallito in 1–8 s: errore dello script locale (regex
  `^…$` passata a `--function`, che Halmos ancora da sé → "No tests"). Rilanciato con gli
  stessi argomenti della CI (`formal.yml`).
