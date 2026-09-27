# Mutation testing: solana

Generato da `tools/mutation/mutate.py solana`. COMPILE_ERROR: 36, EQUIVALENT: 13, KILLED: 174

| Esito | Mutante | Rilevato da (o motivo dell'equivalenza) |
|---|---|---|
| EQUIVALENT | program/src/processor/mint.rs:49:28: replace > with >= in process | EQUIVALENT: con fee di protocollo 0 il mutante invia un Transfer di 0 lamport alla tesoreria; trasferimento di 0 lamport: il System program lo accetta senza effetti (cambia solo il consumo di CU) |
| EQUIVALENT | program/src/processor/mod.rs:87:16: replace > with >= in create_account | EQUIVALENT: con missing = 0 il mutante invia un Transfer di 0 lamport; trasferimento di 0 lamport: il System program lo accetta senza effetti (cambia solo il consumo di CU) |
| EQUIVALENT | program/src/token.rs:39:16: replace < with <= in mint_supply | EQUIVALENT: rifiuterebbe solo un mint Token-2022 di esattamente 82 byte (senza estensioni); il mint di Bernie, vincolato dal vault, ha sempre le estensioni e non è mai lungo 82 byte |
| EQUIVALENT | program/src/token.rs:39:16: replace < with == in mint_supply | EQUIVALENT: vault::load lega il mint all'indirizzo registrato nel vault, che è sempre un mint Token-2022 con MetadataPointer e TokenMetadata (> 82 byte); un account Token-2022 inizializzato più corto di 82 byte non esiste |
| EQUIVALENT | program/src/token.rs:73:9: delete match arm 0 in token_account | EQUIVALENT rispetto alla specifica: Token-2022 mantiene delegate = None ⇒ delegated_amount = 0 (revoke e l'uso completo azzerano entrambi) ma non cancella i byte del vecchio delegato. Il mutante legge quel delegato; per u ≥ 1 serve delegated_amount ≥ u e l'errore resta MissingDelegation; per u = 0 l'errore diventa ZeroAmount invece di MissingDelegation. La specifica non fissa la precedenza tra i due errori (l'oracolo ammette entrambi) e lo stato non cambia |
| EQUIVALENT | program/src/token.rs:95:26: replace + with * in initialize_metadata | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| EQUIVALENT | program/src/token.rs:95:34: replace + with * in initialize_metadata | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| EQUIVALENT | program/src/token.rs:95:45: replace + with * in initialize_metadata | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| EQUIVALENT | program/src/token.rs:95:58: replace + with * in initialize_metadata | EQUIVALENT: cambia solo la capacità MAX del buffer locale (i prodotti sono tutti ≥ delle somme originali, quindi il buffer si allarga); i dati inviati sono data[..at], identici |
| EQUIVALENT | mint: rimosso supply == S dopo MintTo | EQUIVALENT: come per redeem, MintTo aggiunge esattamente u a supply e a S; supply == S dopo il mint vale per costruzione |
| EQUIVALENT | move_lamports: underflow saturato invece che errore | EQUIVALENT: tutti i chiamanti muovono importi ≤ lamport disponibili (solvenza verificata prima); anche se non fosse, la saturazione creerebbe lamport e il runtime rifiuterebbe l'istruzione (UnbalancedInstruction), quindi nessun effetto osservabile diverso dal codice d'errore |
| EQUIVALENT | redeem: rimosso supply == S dopo il burn | EQUIVALENT: prima del burn check_supply verifica supply == S; BurnChecked di Token-2022 toglie esattamente u e S scende di u, quindi supply == S dopo il burn vale per costruzione (controllo difensivo contro un Token-2022 difettoso) |
| EQUIVALENT | sweep: rimosso I2 | EQUIVALENT: sweep_amount = disponibile − ⌈(R+Q)/SCALE⌉, quindi dopo lo sweep I2 vale per costruzione; il controllo resta come difesa |
| COMPILE_ERROR | program/src/lib.rs:103:9: replace Reader<'a>::finish -> ProgramResult with Default::default() | ) error[E0277]: the trait bound `Result<(), ProgramError>: Default` is not satis |
| COMPILE_ERROR | program/src/lib.rs:53:5: replace process_instruction -> ProgramResult with Default::default() | olana/program) error[E0277]: the trait bound `Result<(), ProgramError>: Default` |
| COMPILE_ERROR | program/src/lib.rs:68:5: replace err -> ProgramError with Default::default() | p/claude-0/mutwork-sol/solana/program) error[E0277]: the trait bound `ProgramErr |
| COMPILE_ERROR | program/src/lib.rs:80:9: replace Reader<'a>::take -> Result<&'a[u8], ProgramError> with Ok(Vec::leak(Vec::new())) | ec::new()))    \|            ^^^ use of undeclared type `Vec`  error[E0433]: fai |
| COMPILE_ERROR | program/src/lib.rs:80:9: replace Reader<'a>::take -> Result<&'a[u8], ProgramError> with Ok(Vec::leak(vec![0])) | 22    \| 80 \|         Ok(Vec::leak(vec![0]))    \|                      ^^^  er |
| COMPILE_ERROR | program/src/lib.rs:80:9: replace Reader<'a>::take -> Result<&'a[u8], ProgramError> with Ok(Vec::leak(vec![1])) | 22    \| 80 \|         Ok(Vec::leak(vec![1]))    \|                      ^^^  er |
| COMPILE_ERROR | program/src/lib.rs:97:9: replace Reader<'a>::string -> Result<&'a[u8], ProgramError> with Ok(Vec::leak(Vec::new())) | ec::new()))    \|            ^^^ use of undeclared type `Vec`  error[E0433]: fai |
| COMPILE_ERROR | program/src/lib.rs:97:9: replace Reader<'a>::string -> Result<&'a[u8], ProgramError> with Ok(Vec::leak(vec![0])) | 22    \| 97 \|         Ok(Vec::leak(vec![0]))    \|                      ^^^  er |
| COMPILE_ERROR | program/src/lib.rs:97:9: replace Reader<'a>::string -> Result<&'a[u8], ProgramError> with Ok(Vec::leak(vec![1])) | 22    \| 97 \|         Ok(Vec::leak(vec![1]))    \|                      ^^^  er |
| COMPILE_ERROR | program/src/processor/create.rs:18:5: replace process -> ProgramResult with Default::default() | t satisfied   --> program/src/processor/create.rs:18:5    \| 18 \|     Default:: |
| COMPILE_ERROR | program/src/processor/donate.rs:9:5: replace process -> ProgramResult with Default::default() | is not satisfied  --> program/src/processor/donate.rs:9:5   \| 9 \|     Default: |
| COMPILE_ERROR | program/src/processor/mint.rs:18:5: replace process -> ProgramResult with Default::default() | not satisfied   --> program/src/processor/mint.rs:18:5    \| 18 \|     Default:: |
| COMPILE_ERROR | program/src/processor/mod.rs:16:5: replace signer_writable -> ProgramResult with Default::default() | ram) error[E0277]: the trait bound `Result<(), ProgramError>: Default` is not sa |
| COMPILE_ERROR | program/src/processor/mod.rs:23:5: replace writable -> ProgramResult with Default::default() | ram) error[E0277]: the trait bound `Result<(), ProgramError>: Default` is not sa |
| COMPILE_ERROR | program/src/processor/mod.rs:30:5: replace system_program -> ProgramResult with Default::default() | is not satisfied   --> program/src/processor/mod.rs:30:5    \| 30 \|     Default |
| COMPILE_ERROR | program/src/processor/mod.rs:37:5: replace treasury -> ProgramResult with Default::default() | is not satisfied   --> program/src/processor/mod.rs:37:5    \| 37 \|     Default |
| COMPILE_ERROR | program/src/processor/mod.rs:45:5: replace move_lamports -> ProgramResult with Default::default() | ram) error[E0277]: the trait bound `Result<(), ProgramError>: Default` is not sa |
| COMPILE_ERROR | program/src/processor/mod.rs:75:5: replace create_account -> ProgramResult with Default::default() | is not satisfied   --> program/src/processor/mod.rs:75:5    \| 75 \|     Default |
| COMPILE_ERROR | program/src/processor/redeem.rs:19:5: replace process -> ProgramResult with Default::default() | t satisfied   --> program/src/processor/redeem.rs:19:5    \| 19 \|     Default:: |
| COMPILE_ERROR | program/src/processor/sweep.rs:11:5: replace process -> ProgramResult with Default::default() | ot satisfied   --> program/src/processor/sweep.rs:11:5    \| 11 \|     Default:: |
| COMPILE_ERROR | program/src/token.rs:125:28: replace + with - in revoke_metadata_authority | ]: attempt to compute `8_usize - 32_usize`, which would overflow    --> program/ |
| COMPILE_ERROR | program/src/token.rs:125:5: replace revoke_metadata_authority -> ProgramResult with Default::default() | rogram) error[E0277]: the trait bound `Result<(), ProgramError>: Default` is not |
| COMPILE_ERROR | program/src/token.rs:20:7: replace + with - in metadata_tlv_len | p/claude-0/mutwork-sol/solana/program) error: this arithmetic operation will ove |
| COMPILE_ERROR | program/src/token.rs:27:5: replace check_program -> ProgramResult with Default::default() | ana/program) error[E0277]: the trait bound `Result<(), ProgramError>: Default` i |
| COMPILE_ERROR | program/src/token.rs:47:5: replace check_supply -> ProgramResult with Default::default() | ault` is not satisfied   --> program/src/token.rs:47:5    \| 47 \|     Default:: |
| COMPILE_ERROR | program/src/token.rs:62:5: replace token_account -> Result<TokenAccount, ProgramError> with Ok(Default::default()) |  62 \|     Ok(Default::default())    \|        ^^^^^^^^^^^^^^^^^^ the trait `Def |
| COMPILE_ERROR | program/src/token.rs:95:26: replace + with - in initialize_metadata | MAX + URI_MAX;    \|                        ^^^^^^^^^ evaluation of `token::init |
| COMPILE_ERROR | program/src/token.rs:95:34: replace + with - in initialize_metadata | AX;    \|                        ^^^^^^^^^^^^^^^^^^^^ evaluation of `token::init |
| COMPILE_ERROR | program/src/token.rs:95:58: replace + with - in initialize_metadata |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ evaluation of `token::initi |
| COMPILE_ERROR | program/src/token.rs:95:5: replace initialize_metadata -> ProgramResult with Default::default() | efault` is not satisfied   --> program/src/token.rs:95:5    \| 95 \|     Default |
| COMPILE_ERROR | program/src/vault.rs:118:5: replace check_solvency -> ProgramResult with Default::default() | rogram) error[E0277]: the trait bound `Result<(), ProgramError>: Default` is not |
| COMPILE_ERROR | program/src/vault.rs:47:5: replace load -> Result<(Header, State), ProgramError> with Ok((Default::default(), Default::default())) | ound `Vault<1000000000>: Default` is not satisfied   --> program/src/vault.rs:47 |
| COMPILE_ERROR | program/src/vault.rs:56:65: replace + with - in load |  program/src/vault.rs:56:55    \| 56 \|         creator: Address::new_from_array |
| COMPILE_ERROR | program/src/vault.rs:81:5: replace init -> ProgramResult with Default::default() | ana/program) error[E0277]: the trait bound `Result<(), ProgramError>: Default` i |
| COMPILE_ERROR | program/src/vault.rs:91:28: replace + with - in init | program) error: this arithmetic operation will overflow   --> program/src/vault. |
| COMPILE_ERROR | program/src/vault.rs:99:5: replace store -> ProgramResult with Default::default() | ana/program) error[E0277]: the trait bound `Result<(), ProgramError>: Default` i |
| KILLED | program/src/lib.rs:57:9: delete match arm tag::CREATE in process_instruction | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/lib.rs:58:9: delete match arm tag::MINT in process_instruction | forged_vault_owned_by_another_program |
| KILLED | program/src/lib.rs:59:9: delete match arm tag::REDEEM in process_instruction | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/lib.rs:60:9: delete match arm tag::DONATE in process_instruction | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/lib.rs:61:9: delete match arm tag::SWEEP in process_instruction | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/lib.rs:80:25: replace < with <= in Reader<'a>::take | create_account_attacks |
| KILLED | program/src/lib.rs:80:25: replace < with == in Reader<'a>::take | forged_vault_owned_by_another_program |
| KILLED | program/src/lib.rs:80:25: replace < with > in Reader<'a>::take | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/lib.rs:89:9: replace Reader<'a>::u16 -> Result<u16, ProgramError> with Ok(0) | forged_vault_owned_by_another_program |
| KILLED | program/src/lib.rs:89:9: replace Reader<'a>::u16 -> Result<u16, ProgramError> with Ok(1) | create_account_attacks |
| KILLED | program/src/lib.rs:93:9: replace Reader<'a>::u64 -> Result<u64, ProgramError> with Ok(0) | forged_vault_owned_by_another_program |
| KILLED | program/src/lib.rs:93:9: replace Reader<'a>::u64 -> Result<u64, ProgramError> with Ok(1) | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/processor/create.rs:41:28: replace != with == in process | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/processor/create.rs:55:38: replace + with * in process | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/processor/create.rs:55:38: replace + with - in process | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/processor/mint.rs:49:28: replace > with < in process | differential_sequences |
| KILLED | program/src/processor/mint.rs:49:28: replace > with == in process | differential_sequences |
| KILLED | program/src/processor/mod.rs:16:8: delete ! in signer_writable | forged_vault_owned_by_another_program |
| KILLED | program/src/processor/mod.rs:23:8: delete ! in writable | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/processor/mod.rs:30:22: replace != with == in system_program | forged_vault_owned_by_another_program |
| KILLED | program/src/processor/mod.rs:37:8: delete ! in treasury | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/processor/mod.rs:45:15: replace == with != in move_lamports | sweep_beneficiary_is_always_the_registered_creator |
| KILLED | program/src/processor/mod.rs:76:16: replace == with != in create_account | create_with_prefunded_mint_and_vault |
| KILLED | program/src/processor/mod.rs:87:16: replace > with < in create_account | create_with_prefunded_mint_and_vault |
| KILLED | program/src/processor/mod.rs:87:16: replace > with == in create_account | create_with_prefunded_mint_and_vault |
| KILLED | program/src/processor/redeem.rs:40:19: replace != with == in process | redeem_token_account_attacks |
| KILLED | program/src/processor/redeem.rs:43:20: replace != with == in process | redeem_token_account_attacks |
| KILLED | program/src/processor/redeem.rs:46:31: replace != with == in process | insolvent_vault_refuses_every_operation |
| KILLED | program/src/processor/redeem.rs:46:60: replace \|\| with && in process | redeem_token_account_attacks |
| KILLED | program/src/processor/redeem.rs:46:85: replace < with <= in process | redeem_token_account_attacks |
| KILLED | program/src/processor/redeem.rs:46:85: replace < with == in process | redeem_token_account_attacks |
| KILLED | program/src/processor/redeem.rs:46:85: replace < with > in process | insolvent_vault_refuses_every_operation |
| KILLED | program/src/processor/sweep.rs:19:26: replace != with == in process | mint_must_be_token2022_and_initialized |
| KILLED | program/src/token.rs:100:21: replace + with * in initialize_metadata | forged_vault_owned_by_another_program |
| KILLED | program/src/token.rs:100:21: replace + with - in initialize_metadata | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:101:12: replace += with *= in initialize_metadata | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/token.rs:101:12: replace += with -= in initialize_metadata | forged_vault_owned_by_another_program |
| KILLED | program/src/token.rs:102:21: replace + with * in initialize_metadata | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/token.rs:102:21: replace + with - in initialize_metadata | forged_vault_owned_by_another_program |
| KILLED | program/src/token.rs:103:12: replace += with *= in initialize_metadata | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:103:12: replace += with -= in initialize_metadata | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/token.rs:125:28: replace + with * in revoke_metadata_authority | forged_vault_owned_by_another_program |
| KILLED | program/src/token.rs:15:46: replace + with * | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:15:46: replace + with - | create_account_attacks |
| KILLED | program/src/token.rs:15:50: replace + with * | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/token.rs:15:50: replace + with - | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:15:54: replace + with * | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/token.rs:15:54: replace + with - | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:20:12: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:12: replace + with - in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:17: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:17: replace + with - in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:22: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:22: replace + with - in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:30: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:30: replace + with - in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:35: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:35: replace + with - in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:45: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:45: replace + with - in metadata_tlv_len | compute_units |
| KILLED | program/src/token.rs:20:50: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:50: replace + with - in metadata_tlv_len | compute_units |
| KILLED | program/src/token.rs:20:57: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:57: replace + with - in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 0 | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 1 | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:7: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:27:32: replace != with == in check_program | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:35:5: replace mint_supply -> Result<u64, ProgramError> with Ok(0) | forged_vault_owned_by_another_program |
| KILLED | program/src/token.rs:35:5: replace mint_supply -> Result<u64, ProgramError> with Ok(1) | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:35:8: delete ! in mint_supply | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/token.rs:39:16: replace < with > in mint_supply | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:39:21: replace \|\| with && in mint_supply | mint_must_be_token2022_and_initialized |
| KILLED | program/src/token.rs:39:30: replace != with == in mint_supply | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:47:27: replace != with == in check_supply | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/token.rs:62:8: delete ! in token_account | insolvent_vault_refuses_every_operation |
| KILLED | program/src/token.rs:68:16: replace < with <= in token_account | insolvent_vault_refuses_every_operation |
| KILLED | program/src/token.rs:68:16: replace < with == in token_account | redeem_token_account_attacks |
| KILLED | program/src/token.rs:68:16: replace < with > in token_account | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:68:22: replace \|\| with && in token_account | redeem_token_account_attacks |
| KILLED | program/src/token.rs:68:32: replace == with != in token_account | insolvent_vault_refuses_every_operation |
| KILLED | program/src/token.rs:71:58: replace + with * in token_account | insolvent_vault_refuses_every_operation |
| KILLED | program/src/token.rs:71:58: replace + with - in token_account | insolvent_vault_refuses_every_operation |
| KILLED | program/src/token.rs:95:30: replace * with + in initialize_metadata | compute_units |
| KILLED | program/src/token.rs:95:30: replace * with / in initialize_metadata | compute_units |
| KILLED | program/src/token.rs:95:45: replace + with - in initialize_metadata | compute_units |
| KILLED | program/src/vault.rs:100:16: replace + with * in store | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/vault.rs:100:16: replace + with - in store | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:101:28: replace + with * in store | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/vault.rs:101:28: replace + with - in store | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:102:30: replace + with * in store | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/vault.rs:102:30: replace + with - in store | create_account_attacks |
| KILLED | program/src/vault.rs:103:26: replace + with * in store | forged_vault_owned_by_another_program |
| KILLED | program/src/vault.rs:103:26: replace + with - in store | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:109:5: replace available -> Result<u64, ProgramError> with Ok(0) | insolvent_vault_refuses_every_operation |
| KILLED | program/src/vault.rs:109:5: replace available -> Result<u64, ProgramError> with Ok(1) | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:123:5: replace log_state with () | state_log_matches_vault_after_every_operation |
| KILLED | program/src/vault.rs:38:32: replace + with * in u128_at | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:38:32: replace + with - in u128_at | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/vault.rs:38:5: replace u128_at -> u128 with 0 | insolvent_vault_refuses_every_operation |
| KILLED | program/src/vault.rs:38:5: replace u128_at -> u128 with 1 | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:47:8: delete ! in load | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:51:16: replace != with == in load | insolvent_vault_refuses_every_operation |
| KILLED | program/src/vault.rs:51:23: replace \|\| with && in load | program_owned_account_with_bad_header_is_rejected |
| KILLED | program/src/vault.rs:51:36: replace != with == in load | forged_vault_owned_by_another_program |
| KILLED | program/src/vault.rs:51:53: replace \|\| with && in load | program_owned_account_with_bad_header_is_rejected |
| KILLED | program/src/vault.rs:51:69: replace != with == in load | forged_vault_owned_by_another_program |
| KILLED | program/src/vault.rs:56:65: replace + with * in load | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/vault.rs:57:56: replace + with * in load | insolvent_vault_refuses_every_operation |
| KILLED | program/src/vault.rs:57:56: replace + with - in load | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/vault.rs:59:21: replace != with == in load | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:63:68: replace + with * in load | insolvent_vault_refuses_every_operation |
| KILLED | program/src/vault.rs:63:68: replace + with - in load | differential_sequences |
| KILLED | program/src/vault.rs:64:62: replace + with * in load | differential_sequences |
| KILLED | program/src/vault.rs:64:62: replace + with - in load | differential_sequences |
| KILLED | program/src/vault.rs:68:57: replace + with * in load | forged_vault_owned_by_another_program |
| KILLED | program/src/vault.rs:68:57: replace + with - in load | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:82:16: replace != with == in init | cross_market_vault_and_mint_are_rejected |
| KILLED | program/src/vault.rs:89:28: replace + with * in init | create_account_attacks |
| KILLED | program/src/vault.rs:89:28: replace + with - in init | create_account_attacks |
| KILLED | program/src/vault.rs:90:24: replace + with * in init | create_account_attacks |
| KILLED | program/src/vault.rs:90:24: replace + with - in init | account_permutations_and_duplicates_are_rejected |
| KILLED | program/src/vault.rs:91:28: replace + with * in init | create_account_attacks |
| KILLED | program/src/vault.rs:92:22: replace + with * in init | create_account_attacks |
| KILLED | program/src/vault.rs:92:22: replace + with - in init | cross_market_vault_and_mint_are_rejected |
| KILLED | create: freeze authority al creator | create_mint_metadata_and_vault |
| KILLED | create: mint authority al creator invece che al vault | account_permutations_and_duplicates_are_rejected |
| KILLED | create: metadati non validati | create_rejects_bad_parameters |
| KILLED | create: mint non firmatario | create_account_attacks |
| KILLED | create: parametri non validati | create_rejects_bad_parameters |
| KILLED | create: vault non verificato come PDA [vault, mint] | create_account_attacks |
| KILLED | create: update authority dei metadati non revocata | create_mint_metadata_and_vault |
| KILLED | donate: donatore non verificato | signer_writable_program_and_treasury_checks |
| KILLED | donate: rimosso I2 | insolvent_vault_refuses_every_operation |
| KILLED | donate: rimosso supply == S | mint_must_be_token2022_and_initialized |
| KILLED | donate: trasferito a − 1 | differential_sequences |
| KILLED | mint: conia u + 1 token | insolvent_vault_refuses_every_operation |
| KILLED | mint: la fee del creator non entra nel vault | differential_sequences |
| KILLED | mint: fee del protocollo non pagata | differential_sequences |
| KILLED | mint: rimosso signer/writable dell'utente | signer_writable_program_and_treasury_checks |
| KILLED | mint: rimosso I2 con i lamport reali | insolvent_vault_refuses_every_operation |
| KILLED | mint: stato del vault non scritto | forged_vault_owned_by_another_program |
| KILLED | mint: rimosso supply == S prima | mint_must_be_token2022_and_initialized |
| KILLED | mint: System program non verificato | signer_writable_program_and_treasury_checks |
| KILLED | mint: programma token non verificato | signer_writable_program_and_treasury_checks |
| KILLED | mint: tesoreria non verificata (fee dirottabile) | signer_writable_program_and_treasury_checks |
| KILLED | mint: l'utente paga 1 lamport in meno di backing | differential_sequences |
| KILLED | redeem: brucia u − 1 | insolvent_vault_refuses_every_operation |
| KILLED | redeem: rimosso il controllo che il delegato sia il vault | redeem_token_account_attacks |
| KILLED | redeem: rimosso il controllo della quantità delegata | redeem_token_account_attacks |
| KILLED | redeem: rimosso il controllo che il token account sia del firmatario | redeem_token_account_attacks |
| KILLED | redeem: fee del protocollo non pagata (resta nel vault) | differential_sequences |
| KILLED | redeem: rimosso signer/writable dell'utente | signer_writable_program_and_treasury_checks |
| KILLED | redeem: rimosso I2 con i lamport reali | insolvent_vault_refuses_every_operation |
| KILLED | redeem: stato del vault non scritto | differential_sequences |
| KILLED | redeem: rimosso supply == S prima | mint_must_be_token2022_and_initialized |
| KILLED | redeem: rimosso il controllo del mint del token account | redeem_token_account_attacks |
| KILLED | redeem: programma token non verificato | signer_writable_program_and_treasury_checks |
| KILLED | redeem: tesoreria non verificata | signer_writable_program_and_treasury_checks |
| KILLED | redeem: 1 lamport in più all'utente | differential_sequences |
| KILLED | redeem: l'utente riceve il lordo (fee non trattenute) | differential_sequences |
| KILLED | redeem: la tesoreria riceve anche la fee del creator | differential_sequences |
| KILLED | sweep: 1 lamport oltre l'excess | sweep_beneficiary_is_always_the_registered_creator |
| KILLED | sweep: beneficiario non verificato (furto dell'excess) | sweep_beneficiary_is_always_the_registered_creator |
| KILLED | sweep: rimosso supply == S | mint_supply_must_match_vault_supply |
| KILLED | token_account: account non inizializzato accettato | redeem_token_account_attacks |
| KILLED | token_account: account non Token-2022 accettato | redeem_token_account_attacks |
| KILLED | mint_supply: mint non inizializzato accettato | mint_must_be_token2022_and_initialized |
| KILLED | mint_supply: mint non verificato come Token-2022 | mint_must_be_token2022_and_initialized |
| KILLED | treasury(): qualunque account accettato come tesoreria | signer_writable_program_and_treasury_checks |
| KILLED | vault::available: il rent conta come excess | sweep_beneficiary_is_always_the_registered_creator |
| KILLED | vault::load: discriminatore e versione ignorati | program_owned_account_with_bad_header_is_rejected |
| KILLED | vault::load: vault non legato al mint (cross-market) | cross_market_vault_and_mint_are_rejected |
| KILLED | vault::load: owner del vault non verificato | forged_vault_owned_by_another_program |
