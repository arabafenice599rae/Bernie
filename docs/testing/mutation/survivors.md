# Mutanti sopravvissuti al primo giro (fase 17)

Ogni mutante SURVIVED nel primo giro e dove è finito. Per un KILLED la colonna indica il test
aggiunto che lo rileva, cioè la proprietà che mancava; per un EQUIVALENT il motivo (analisi
manuale, `tools/mutation/equivalent.json`). I COMPILE_ERROR non contano come test forte: il
compilatore rifiuta il mutante (tipi, borrow) e i test non lo vedono mai.

## solana

61 sopravvissuti al primo giro → EQUIVALENT 13, KILLED 48

| Mutante | Esito finale | Test che lo rileva / motivo |
|---|---|---|
| redeem: rimosso il controllo del mint del token account | KILLED | redeem_token_account_attacks |
| redeem: rimosso il controllo che il delegato sia il vault | KILLED | redeem_token_account_attacks |
| redeem: rimosso signer/writable dell'utente | KILLED | signer_writable_program_and_treasury_checks |
| redeem: tesoreria non verificata | KILLED | signer_writable_program_and_treasury_checks |
| redeem: programma token non verificato | KILLED | signer_writable_program_and_treasury_checks |
| redeem: rimosso supply == S prima | KILLED | mint_must_be_token2022_and_initialized |
| redeem: rimosso supply == S dopo il burn | EQUIVALENT | EQUIVALENT: prima del burn check_supply verifica supply == S; BurnChecked di Token-2022 toglie esattamente u e S scende di u, quindi supply == S dopo il burn vale per costruzione (controllo difensivo contro un Token-2022 difettoso) |
| redeem: rimosso I2 con i lamport reali | KILLED | insolvent_vault_refuses_every_operation |
| mint: rimosso signer/writable dell'utente | KILLED | signer_writable_program_and_treasury_checks |
| mint: System program non verificato | KILLED | signer_writable_program_and_treasury_checks |
| mint: programma token non verificato | KILLED | signer_writable_program_and_treasury_checks |
| mint: rimosso supply == S prima | KILLED | mint_must_be_token2022_and_initialized |
| mint: rimosso supply == S dopo MintTo | EQUIVALENT | EQUIVALENT: come per redeem, MintTo aggiunge esattamente u a supply e a S; supply == S dopo il mint vale per costruzione |
| mint: rimosso I2 con i lamport reali | KILLED | insolvent_vault_refuses_every_operation |
| donate: rimosso supply == S | KILLED | mint_must_be_token2022_and_initialized |
| donate: rimosso I2 | KILLED | insolvent_vault_refuses_every_operation |
| donate: donatore non verificato | KILLED | signer_writable_program_and_treasury_checks |
| sweep: rimosso I2 | EQUIVALENT | EQUIVALENT: sweep_amount = disponibile − ⌈(R+Q)/SCALE⌉, quindi dopo lo sweep I2 vale per costruzione; il controllo resta come difesa |
| sweep: rimosso supply == S | KILLED | mint_supply_must_match_vault_supply |
| create: vault non verificato come PDA [vault, mint] | KILLED | create_account_attacks |
| create: mint non firmatario | KILLED | create_account_attacks |
| vault::load: owner del vault non verificato | KILLED | forged_vault_owned_by_another_program |
| vault::load: vault non legato al mint (cross-market) | KILLED | cross_market_vault_and_mint_are_rejected |
| vault::load: discriminatore e versione ignorati | KILLED | program_owned_account_with_bad_header_is_rejected |
| mint_supply: mint non verificato come Token-2022 | KILLED | mint_must_be_token2022_and_initialized |
| mint_supply: mint non inizializzato accettato | KILLED | mint_must_be_token2022_and_initialized |
| token_account: account non Token-2022 accettato | KILLED | redeem_token_account_attacks |
| token_account: account non inizializzato accettato | KILLED | redeem_token_account_attacks |
| move_lamports: underflow saturato invece che errore | EQUIVALENT | EQUIVALENT: tutti i chiamanti muovono importi ≤ lamport disponibili (solvenza verificata prima); anche se non fosse, la saturazione creerebbe lamport e il runtime rifiuterebbe l'istruzione (UnbalancedInstruction), quindi nessun effetto osservabile diverso dal codice d'errore |
| program/src/processor/mod.rs:87:16: replace > with >= in create_account | EQUIVALENT | EQUIVALENT: con missing = 0 il mutante invia un Transfer di 0 lamport; trasferimento di 0 lamport: il System program lo accetta senza effetti (cambia solo il consumo di CU) |
| program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 0 | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 1 | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:57: replace + with - in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:57: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:45: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:30: replace + with - in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:30: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:17: replace + with - in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:17: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:12: replace + with - in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:12: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:7: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:22: replace + with - in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:22: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:35: replace + with - in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:35: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:20:50: replace + with * in metadata_tlv_len | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/token.rs:39:21: replace \|\| with && in mint_supply | KILLED | mint_must_be_token2022_and_initialized |
| program/src/token.rs:39:16: replace < with == in mint_supply | EQUIVALENT | EQUIVALENT: vault::load lega il mint all'indirizzo registrato nel vault, che è sempre un mint Token-2022 con MetadataPointer e TokenMetadata (> 82 byte); un account Token-2022 inizializzato più corto di 82 byte non esiste |
| program/src/token.rs:39:16: replace < with <= in mint_supply | EQUIVALENT | EQUIVALENT: rifiuterebbe solo un mint Token-2022 di esattamente 82 byte (senza estensioni); il mint di Bernie, vincolato dal vault, ha sempre le estensioni e non è mai lungo 82 byte |
| program/src/token.rs:68:22: replace \|\| with && in token_account | KILLED | redeem_token_account_attacks |
| program/src/token.rs:73:9: delete match arm 0 in token_account | EQUIVALENT | EQUIVALENT rispetto alla specifica: Token-2022 mantiene delegate = None ⇒ delegated_amount = 0 (revoke e l'uso completo azzerano entrambi) ma non cancella i byte del vecchio delegato. Il mutante legge quel delegato; per u ≥ 1 serve delegated_amount ≥ u e l'errore resta MissingDelegation; per u = 0 l'errore diventa ZeroAmount invece di MissingDelegation. La specifica non fissa la precedenza tra i due errori (l'oracolo ammette entrambi) e lo stato non cambia |
| program/src/token.rs:95:58: replace + with * in initialize_metadata | EQUIVALENT | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| program/src/token.rs:95:45: replace + with * in initialize_metadata | EQUIVALENT | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| program/src/token.rs:95:34: replace + with * in initialize_metadata | EQUIVALENT | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| program/src/token.rs:95:26: replace + with * in initialize_metadata | EQUIVALENT | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| program/src/vault.rs:51:53: replace \|\| with && in load | KILLED | program_owned_account_with_bad_header_is_rejected |
| program/src/vault.rs:51:23: replace \|\| with && in load | KILLED | program_owned_account_with_bad_header_is_rejected |
| program/src/vault.rs:123:5: replace log_state with () | KILLED | state_log_matches_vault_after_every_operation |
| program/src/processor/create.rs:55:38: replace + with * in process | KILLED | create_funds_exact_rent_for_final_sizes |
| program/src/processor/mint.rs:49:28: replace > with >= in process | EQUIVALENT | EQUIVALENT: con fee di protocollo 0 il mutante invia un Transfer di 0 lamport alla tesoreria; trasferimento di 0 lamport: il System program lo accetta senza effetti (cambia solo il consumo di CU) |

## evm

38 sopravvissuti al primo giro → EQUIVALENT 3, KILLED 35

| Mutante | Esito finale | Test che lo rileva / motivo |
|---|---|---|
| mint: nessun controllo di solvenza | KILLED | test_short_vault_refuses_every_operation |
| mint: guard di rientro rimosso | KILLED | test_reentrancy_during_claim_and_sweep |
| redeem: nessun controllo di solvenza | KILLED | test_short_vault_refuses_every_operation |
| donate: nessun controllo di solvenza | KILLED | test_short_vault_refuses_every_operation |
| claimFeesFor: guard di rientro rimosso | KILLED | test_reentrancy_during_claim_and_sweep |
| claimFees: guard di rientro rimosso | KILLED | test_reentrancy_during_claim_and_sweep |
| sweep: guard di rientro rimosso | KILLED | test_reentrancy_during_claim_and_sweep |
| solvenza: supply ERC-20 non confrontata con S | EQUIVALENT | EQUIVALENT: _state() legge S da totalSupply() e _mint/_burn muovono esattamente u, quindi totalSupply() == s.S vale per costruzione; il controllo è difensivo e non raggiungibile senza modificare altro codice |
| solvenza: pagamenti in uscita ignorati | KILLED | test_short_vault_refuses_every_operation |
| solvenza: fee dovute ignorate | KILLED | test_short_vault_refuses_every_operation |
| fee: arrotondamento separato della quota protocollo | EQUIVALENT | EQUIVALENT con le costanti v1.6: ⌊⌊40b/10⁴⌋·20/40⌋ = ⌊⌊b/250⌋/2⌋ = ⌊b/500⌋ = ⌊20b/10⁴⌋ (identità dei floor annidati). Con FEE_C ≠ FEE_P il mutante diventerebbe osservabile e va riesaminato |
| mint: asserzione I1–I6 rimossa | KILLED | test_operations_refuse_corrupt_input_state |
| evm/src/Bernie.sol:84: riga cancellata `_checkSolvency(s, refund);` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/Bernie.sol:87: `>` → `>=` in `if (refund > 0) Address.sendValue(payable(msg.sender), refund);` | KILLED | test_exact_mint_from_contract_without_receive |
| evm/src/Bernie.sol:97: riga cancellata `_checkSolvency(s, out);` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/Bernie.sol:107: riga cancellata `_checkSolvency(s, 0);` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/Bernie.sol:124: `+` → `-` in `uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/Bernie.sol:124: `-` → `+` in `uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE` | KILLED | test failed |
| evm/src/Bernie.sol:124: `/` → `*` in `uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE` | KILLED | test_reentrancy_during_claim_and_sweep |
| evm/src/Bernie.sol:157: riga cancellata `if (totalSupply() != s.S) revert InvariantViolated();` | EQUIVALENT | EQUIVALENT: stessa riga di t:solvency-no-supply-check (cancellazione del controllo totalSupply() != s.S), non raggiungibile per costruzione |
| evm/src/Bernie.sol:158: `-` → `+` in `uint256 bal = address(this).balance - pendingOut;` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/Bernie.sol:159: `+` → `-` in `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/Bernie.sol:159: `*` → `/` in `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/Bernie.sol:159: riga cancellata `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | KILLED | test_short_vault_refuses_every_operation |
| evm/src/BernieMath.sol:66: riga cancellata `checkAt(s, kPrev, SCALE);` | KILLED | test_checkAt_rejects_each_invariant_alone |
| evm/src/BernieMath.sol:71: `\|\|` → `&&` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | KILLED | test_checkAt_rejects_each_invariant_alone |
| evm/src/BernieMath.sol:71: `\|\|` → `&&` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | KILLED | test_checkAt_rejects_each_invariant_alone |
| evm/src/BernieMath.sol:71: `\|\|` → `&&` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | KILLED | test_checkAt_rejects_each_invariant_alone |
| evm/src/BernieMath.sol:72: `>=` → `>` in `\|\| (s.S != 0 && s.Q >= s.S)` | KILLED | test_checkAt_rejects_each_invariant_alone |
| evm/src/BernieMath.sol:72: `\|\|` → `&&` in `\|\| (s.S != 0 && s.Q >= s.S)` | KILLED | test_checkAt_rejects_each_invariant_alone |
| evm/src/BernieMath.sol:103: riga cancellata `checkAt(s, s0.k, scale);` | KILLED | test_operations_refuse_corrupt_input_state |
| evm/src/BernieMath.sol:125: `<=` → `<` in `if (full <= pen) revert Dust();` | KILLED | test_dust_is_its_own_error |
| evm/src/BernieMath.sol:125: riga cancellata `if (full <= pen) revert Dust();` | KILLED | test_dust_is_its_own_error |
| evm/src/BernieMath.sol:130: `<` → `<=` in `if (out < minOut) revert Slippage();` | KILLED | test_min_out_and_value_are_inclusive |
| evm/src/BernieMath.sol:139: riga cancellata `checkAt(s, s0.k, scale);` | KILLED | test_operations_refuse_corrupt_input_state |
| evm/src/BernieMath.sol:151: riga cancellata `checkAt(s, s0.k, scale);` | KILLED | test_operations_refuse_corrupt_input_state |
| evm/src/BernieMath.sol:158: `>` → `>=` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | KILLED | test_metadata_bounds_inclusive |
| evm/src/BernieMath.sol:158: `>` → `>=` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | KILLED | test_metadata_bounds_inclusive |
