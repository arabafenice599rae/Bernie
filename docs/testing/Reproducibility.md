# Riproducibilità (fase 24)

Ogni campagna del pre-audit è deterministica: stesso seed, stessa sequenza, stesso esito.
I generatori usano `random.Random` con seed testuale (`"{chain}:{seed}"` per `gen.py`,
`"stream:{scale}:{seed}"` per `stream.py`, `"{BERNIE_SEED}:{tag}"` per i test Python),
quindi non dipendono da `PYTHONHASHSEED`, dall'ora o dall'ordine dei file.
`tests/test_redteam.py` verifica che lo stesso seed dia la stessa sequenza e che seed
diversi diano sequenze diverse.

Versioni degli strumenti: [Baseline.md](Baseline.md#versioni).

## Setup

```sh
# Solana: toolchain Agave 4.2.2 (build-sbf) e Rust stable
sh -c "$(curl -sSfL https://release.anza.xyz/v4.2.2/install)"
# EVM: Foundry, dipendenze npm (OpenZeppelin)
(cd evm && npm ci)
# strumenti opzionali
cargo install --locked cargo-mutants   # mutation testing
pip install halmos==0.3.3              # verifica simbolica EVM
cargo install --locked kani-verifier && cargo kani setup
```

## Un comando per ciascuna campagna

| Campagna | Comando (dalla radice del repo) |
|---|---|
| Suite di CI completa | vedi `.github/workflows/ci.yml` (job model, rust, solana, frontend, evm) |
| Vettori di confine | `python tools/redteam/boundary.py --check` poi `cargo test -p bernie-state --release --test boundary` (in `solana/`) e `forge test --match-contract BoundaryTest` (in `evm/`) |
| Differential stato, 10⁶ passi | `cd solana && cargo build --release -p bernie-state --example diff_stream && python3 ../tools/redteam/stream.py --seed 1 --seqs 1000 --ops 1000 --scale 10 \| ./target/release/examples/diff_stream` |
| Differential programma (Mollusk) | `python3 tools/redteam/gen.py sol --seed 1000 --seqs 100 --ops 500 --users 8 --out /tmp/seq && cd solana && BERNIE_DIFF_DIR=/tmp/seq cargo test -p bernie-program --release --test differential -- --nocapture` |
| Marathon Solana (120 utenti × 100k) | `python3 tools/redteam/gen.py sol --seed 777 --seqs 1 --ops 100000 --users 120 --sparse 1000 --out /tmp/marathon && cd solana && BERNIE_DIFF_DIR=/tmp/marathon cargo test -p bernie-program --release --test differential` |
| Differential contratti (Foundry) | `cd evm && python3 ../tools/redteam/gen.py evm --seed 5000 --seqs 100 --ops 500 --users 8 --out diff && forge test --match-contract DifferentialTest --gas-limit 18446744073709551615` (oltre ~10⁵ passi il limite di default non basta) |
| Test avversariali | `cargo test -p bernie-program --release --test adversarial` e `forge test --match-contract AdversarialTest` |
| Mutation testing | `python3 tools/mutation/mutate.py state\|evm\|solana` (sottoinsieme: `--only REGEX` sugli id) |
| Kani | `cd solana/state && cargo kani --harness NOME` (5 harness, elenco in Baseline.md) |
| Halmos | `cd evm && halmos --contract BernieMathHalmos --function check_NOME --solver-timeout-assertion 0` |

I seed delle campagne registrate nel report sono in [PreAuditReport.md](PreAuditReport.md).
`.github/workflows/redteam.yml` rilancia ogni domenica le campagne lunghe con gli stessi seed
(moltiplicabili con l'input `scale`).

## Riprodurre un fallimento

1. Il consumatore stampa file (o seed) e passo della prima divergenza, con atteso e ottenuto.
2. Rigenerare solo quella sequenza: `gen.py sol|evm --seed S --seqs 1 --ops M --users U`.
   Per lo stream, `stream.py --seed S --seqs 1` con la stessa `--scale` (il seed è la riga `# seed`).
3. Minimizzare: `python3 tools/redteam/minimize.py sol FILE.json` oppure
   `minimize.py evm --seed S --ops M --users U`. Il risultato va in `docs/testing/repro/` con il
   comando per rieseguirlo. `BERNIE_ROOT=/percorso/copia` esegue i consumatori su un altro albero
   (per esempio una copia con un mutante), senza toccare il repo.
4. Trasformare la sequenza minima in un test di regressione nella suite della chain.

Verifica del minimizzatore: con un mutante iniettato in una copia (quota protocollo
`(total + 1) · FEE_P / 40`), `minimize.py evm --seed 31337 --ops 200 --users 4` ha ridotto
la sequenza da 200 passi a 1 (un mint, fee del creator 2223 contro 2224) in 16 s.
