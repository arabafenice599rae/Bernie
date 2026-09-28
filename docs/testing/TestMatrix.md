# Matrice specifica → test (pre-audit, fase 1)

Ogni proprietà normativa della specifica (`README.md`, v1.6) collegata a implementazione e
test. **Esistente** = test presente prima del pre-audit; **Avversariale** = aggiunto dal
pre-audit, con oracolo indipendente (modello dell'Appendice A, invariante della specifica o
rifiuto con stato invariato). Stato: ✅ coperto da test positivo e avversariale; 🟡 coperto
solo in parte (vedi nota).

Legenda dei file:

| Sigla | File |
|---|---|
| py-prop | `tests/test_properties.py` (P1–P7, §8, §9) |
| py-fuzz | `tests/test_fuzz.py` |
| rs-vec | `solana/state/tests/vectors.rs` (vettori del modello, SCALE 10 e 10⁹) |
| rs-exh | `solana/state/tests/exhaustive.rs` |
| rs-grd | `solana/state/tests/guards.rs` (nuovo) |
| rs-bnd | `solana/state/tests/boundary.rs` + `vectors/boundary_sol.txt` (nuovo) |
| rs-str | `solana/state/examples/diff_stream.rs` + `tools/redteam/stream.py` (nuovo) |
| kani | harness Kani in `solana/state/src/state.rs` |
| mol | `solana/program/tests/mollusk.rs` |
| mol-adv | `solana/program/tests/adversarial.rs` (nuovo) |
| mol-dif | `solana/program/tests/differential.rs` + `tools/redteam/gen.py sol` (nuovo) |
| fe | `solana/program/tests/frontend.rs` |
| sol-t | `evm/test/Bernie.t.sol` |
| sol-vec | `evm/test/Vectors.t.sol` |
| sol-adv | `evm/test/Adversarial.t.sol` (nuovo) |
| sol-bnd | `evm/test/Boundary.t.sol` + `vectors/boundary_evm.json` (nuovo) |
| sol-dif | `evm/test/Differential.t.sol` + `tools/redteam/gen.py evm` (nuovo) |
| halmos | `evm/test/BernieMath.halmos.t.sol` |

## Invarianti (§4)

| ID | Proprietà | Implementazione | Esistente | Avversariale | Stato |
|---|---|---|---|---|---|
| I1 | `R = S·k` | `state.rs` `check` (172); `BernieMath.checkAt` (69) | rs-vec, rs-exh, kani, halmos, sol-vec | rs-grd (violazione isolata), sol-adv `checkAt_rejects_each_invariant_alone`, `operations_refuse_corrupt_input_state`; controllo a ogni passo in mol-dif, sol-dif, rs-str | ✅ |
| I2 | solvenza: saldo ≥ (R+Q)/SCALE + fee dovute + rent | `vault::check_solvency` (117); `Bernie._checkSolvency` (156) | mol (lifecycle) | mol-adv `insolvent_vault_refuses_every_operation`; sol-adv `short_vault_refuses_every_operation` (1 wei in meno, con rimborso, uscita e fee in sospeso); conservazione a ogni passo in mol-dif, sol-dif | ✅ |
| I3 | `k` non scende | `check`/`checkAt` | kani, halmos, rs-exh | rs-grd, sol-adv; a ogni passo nei differential | ✅ |
| I4 | `S = 0 ⇒ Q = 0` | `check`/`checkAt`; redeem azzera Q (state.rs 307) | rs-vec, kani | rs-grd (S = 0, Q > 0 isolato), rs-bnd (S = 0) | ✅ |
| I5 | `(R+Q) % SCALE = 0` | `check`/`checkAt` | rs-exh, kani | rs-grd, sol-adv (isolato) | ✅ |
| I6 | `S = 0 ∨ Q < S` | `check`/`checkAt`; absorb | rs-exh, kani, halmos | rs-grd e sol-adv (Q = S, Q > S), rs-bnd absorb con Q = S, S+1, 2S | ✅ |
| Sol | `vault.supply = mint.supply` dopo ogni CPI | `token::check_supply` (46), in ogni processor | mol | mol-adv `mint_supply_must_match_vault_supply` (±1, su tutte e quattro le istruzioni); supply = S a ogni passo in mol-dif | ✅ (il controllo *dopo* la CPI è ridondante con Token-2022 corretto: mutante equivalente) |
| EVM | `totalSupply() = S` | `_checkSolvency` (157) | sol-vec | somma dei saldi = supply a ogni passo in sol-dif | 🟡 irraggiungibile: S si legge da `totalSupply()` (mutante equivalente) |

## Aritmetica (§5) e operazioni (§6)

| ID | Proprietà | Implementazione | Esistente | Avversariale | Stato |
|---|---|---|---|---|---|
| fee | un solo arrotondamento; creator + protocollo = totale; 0,2% + 0,2% | `split_fees` (98); `BernieMath.fees` (50) | py-prop, kani, halmos | rs-bnd `fee_table` (0, 1, 2, 3, 10, 99, 100, 101, 249–251, 499–501, 999, 1000, 10⁴, u64 MAX); mutanti su aliquote e doppio arrotondamento uccisi | ✅ |
| absorb | δ = ⌊Q/S⌋; conserva R+Q; Q' < S; idempotente | `absorb` (155); `BernieMath.absorb` (56) | kani, halmos | rs-bnd e sol-bnd `absorb_boundaries` (Q = 0, 1, S−1, S, S+1, 2S, 2S+1, grande; absorb ripetuto) | ✅ |
| mint | epen ⌈⌉ sugli holder esistenti, absorb, c = ⌈(full+epen)/SCALE⌉, fee su ⌈full/SCALE⌉, resto in Q | `mint` (224); `mintAt` (86) | rs-vec, sol-vec, kani, halmos | rs-bnd, sol-bnd (u = 0, 1, 2, S−1, S, S+1, MAX−1, MAX), rs-str (10⁸ passi), mol-dif, sol-dif; mutanti floor↔ceil uccisi | ✅ |
| redeem | pen ⌈⌉, g = ⌊(full−pen)/SCALE⌋, out = g − ft, Dust, ZeroPayout | `redeem` (273); `redeemAt` (115) | rs-vec, sol-vec, kani, halmos | come mint; `Dust` distinto da `ZeroPayout` (sol-adv); min_out inclusivo (rs-grd, sol-adv) | ✅ |
| donate | Q += a·SCALE, absorb; NoHolders con S = 0 | `donate` (327); `donateAt` (146) | rs-vec, kani, halmos | rs-bnd, sol-bnd (0, 1, 2, grande); differential | ✅ |
| sweep | excess = disponibile − (R+Q)/SCALE (− fee dovute su EVM), al creator registrato, chiamabile da chiunque | `sweep.rs`; `Bernie.sweep` (123) | mol, sol-t (lifecycle) | mol-adv `sweep_beneficiary_is_always_the_registered_creator` (beneficiario diverso rifiutato, chiamante senza guadagno, secondo sweep NothingToClaim); sol-adv `sweep_pays_only_the_creator_and_only_once` (ETH forzato con S > 0); sweep nei differential | ✅ |
| claim | pull delle fee EVM; `claimFeesFor` paga solo il titolare; `claimAll` su più token | `_claim` (131), `claimAll` (Factory 25) | sol-t | sol-adv `fee_theft_attempts` (furto e doppio prelievo), differential con claim e claimAll | ✅ |
| slippage | `c + ft ≤ max_cost`, `out ≥ min_out` inclusivi | state.rs 246, 299; Bernie.sol 79; BernieMath 130 | rs-vec | rs-grd, sol-adv (esatto, esatto ± 1), rs-bnd | ✅ |

## Errori (§7) e casi limite

| ID | Proprietà | Esistente | Avversariale | Stato |
|---|---|---|---|---|
| codici | 1–17, 7 riservato | `params.rs error_codes_match_section_7`, `frontend.rs` | differential confronta il nome dell'errore a ogni passo | ✅ |
| Overflow | nel punto del calcolo; importi nativi e S in u64, k, R, Q in u128 (Solana), uint256 (EVM) | — | rs-bnd, sol-bnd (Panic), rs-str: errore unico dal valutatore di §7 (`tools/redteam/ordered.py`) | ✅ |
| ordine | un solo errore, il primo nell'ordine di §7 (account → argomenti → calcolo → Slippage → piattaforma; InvariantViolated fuori ordine) | — | mol-adv e sol-adv `error_precedence_follows_spec`; mol-dif, sol-dif, rs-bnd, sol-bnd, rs-str con oracolo esatto | ✅ |
| atomicità | un errore non cambia nulla | kani (`Err ⇒ stato identico`) | mol-adv confronta l'intero store di Mollusk; mol-dif confronta tutti gli account; sol-adv `failed_operations_leave_state_unchanged`; sol-dif a ogni passo | ✅ |
| S = 0 / rientro | ultimo uscente, excess al creator, rientro al k raggiunto | mol, sol-t | rs-bnd (S = 0, 1, 2), differential | ✅ |

## Autorizzazioni e account (§11, §12, §13)

| ID | Proprietà | Implementazione | Esistente | Avversariale | Stato |
|---|---|---|---|---|---|
| owner | redeem solo dal conto del firmatario | redeem.rs 43 | mol | mol-adv `redeem_token_account_attacks` (conto di Bob con delega di Bob, firmato da Alice) | ✅ |
| delega | delegato = vault, quantità ≥ u | redeem.rs 46 | mol | mol-adv (delegato diverso, insufficiente, revocata); mol-dif (delega come stato dell'oracolo) | ✅ |
| firma | utente firmatario in mint, redeem, donate | `signer_writable` | — | mol-adv `signer_writable_program_and_treasury_checks` (senza la firma su redeem chiunque potrebbe forzare l'uscita di chi ha una delega attiva) | ✅ |
| programmi | System e Token-2022 verificati | `system_program`, `token::check_program` | — | mol-adv (programmi falsi) | ✅ |
| tesoreria | Solana: lista `TREASURIES`, scrivibile; EVM: immutable dell'implementazione | `treasury` (mod.rs 36) | mol `rejects_unknown_treasury` | mol-adv (fuori lista, non scrivibile), sol-adv `arbitrary_clone_cannot_redirect_treasury` | ✅ |
| vault | owner = programma, discriminatore, versione, lunghezza, legato al mint | `vault::load` (42) | — | mol-adv `forged_vault_owned_by_another_program`, `program_owned_account_with_bad_header_is_rejected`, `cross_market_vault_and_mint_are_rejected` | ✅ |
| PDA | create: vault = PDA [vault, mint]; unico per mint | create.rs 41 | mol | mol-adv `create_account_attacks` (vault non PDA, mint o creator non firmatari, seconda create) | ✅ |
| Token-2022 | mint e token account posseduti da Token-2022 e inizializzati | `mint_supply`, `token_account` | — | mol-adv `mint_must_be_token2022_and_initialized`, `redeem_token_account_attacks` | ✅ |
| ordine account | nessuna permutazione o duplicato accettato | tutti i processor | — | mol-adv `account_permutations_and_duplicates_are_rejected` (tutte le permutazioni: 5.040 per mint; ogni slot duplicato) | ✅ |
| rientro | `nonReentrant` su ogni funzione che cambia stato (§13) | Bernie.sol | sol-t `reentrancy_blocked` (solo redeem) | sol-adv: attaccante che rientra in mint, redeem, donate, sweep, claimFees, claimFeesFor, claimAll durante rimborso del mint, uscita del redeem, claim, claimFeesFor, claimAll e sweep | ✅ |
| clone | nessun initializer; implementazione inutilizzabile; salt legato al chiamante; creator = msg.sender | Factory 39 | sol-t | sol-adv (initialize inesistente, stesso salt da un altro chiamante, tx.origin ≠ msg.sender, clone arbitrario) | ✅ |
| transfer | nessun trasferimento verso il contratto | `_update` (163) | sol-t | sol-adv, sol-dif (`transfer_self`) | ✅ |

## Economia (§8)

| ID | Proprietà | Esistente | Avversariale | Stato |
|---|---|---|---|---|
| P1 | round-trip in perdita | py-prop | — | ✅ (modello) |
| P2 | holder passivo non perde valore di backing | py-prop | I3 a ogni passo dei differential | ✅ |
| P3 | conservazione globale al lamport/wei | py-prop (Ledger) | mol-dif: somma dei lamport di tutti gli account costante a ogni passo; sol-dif: somma degli ETH costante; oracolo `Market.check` | ✅ |
| P3b | ogni resto in Q o excess | py-prop | differential (Q e excess confrontati a ogni passo) | ✅ |
| P4–P7 | penalità non frazionabile, niente auto-rimborso, cattura limitata (ε_c, ε_d), monotonia | py-prop (con test di sensibilità) | — | ✅ (modello; le implementazioni coincidono col modello bit per bit) |

## Metadati, create, log (§6, §11, §12)

| ID | Proprietà | Esistente | Avversariale | Stato |
|---|---|---|---|---|
| metadati | nome 1–32, simbolo 1–10, URI ≤ 200; update authority revocata | params.rs, mol | sol-adv `metadata_bounds_inclusive` (32 e 10 ammessi) | ✅ |
| rent | create finanzia il rent esatto della dimensione finale di mint e vault | — | mol-adv `create_funds_exact_rent_for_final_sizes` | ✅ |
| pre-finanziamento | create regge lamport inviati prima | mol | — | ✅ |
| log | `State k S R Q` dopo ogni operazione, con i valori del vault | — | mol-adv `state_log_matches_vault_after_every_operation` | ✅ |
| frontend | costruttori di istruzioni e limiti di CU | fe | — | ✅ |

## Non verificabile con i test

| Voce | Motivo |
|---|---|
| Token-2022 stesso, runtime Solana, EVM | fuori perimetro: si assume corretto (i controlli dopo le CPI sono difese in profondità) |
| Configurazione di deploy (tesorerie reali, upgrade authority, factory di mainnet) | non ancora decisa: le tesorerie sono segnaposto (§17) |
| Economia oltre il modello (MEV, ordinamento) | descritta in §8 e §10, non testabile come proprietà del codice |
