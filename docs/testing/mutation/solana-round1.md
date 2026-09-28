# Mutation testing: solana

Generato da `tools/mutation/mutate.py solana`. COMPILE_ERROR: 36, KILLED: 126, SURVIVED: 61

| Esito | Mutante | Rilevato da |
|---|---|---|
| SURVIVED | program/src/processor/create.rs:55:38: replace + with * in process |  |
| SURVIVED | program/src/processor/mint.rs:49:28: replace > with >= in process |  |
| SURVIVED | program/src/processor/mod.rs:87:16: replace > with >= in create_account |  |
| SURVIVED | program/src/token.rs:20:12: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:12: replace + with - in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:17: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:17: replace + with - in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:22: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:22: replace + with - in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:30: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:30: replace + with - in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:35: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:35: replace + with - in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:45: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:50: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:57: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:57: replace + with - in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 0 |  |
| SURVIVED | program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 1 |  |
| SURVIVED | program/src/token.rs:20:7: replace + with * in metadata_tlv_len |  |
| SURVIVED | program/src/token.rs:39:16: replace < with <= in mint_supply |  |
| SURVIVED | program/src/token.rs:39:16: replace < with == in mint_supply |  |
| SURVIVED | program/src/token.rs:39:21: replace \|\| with && in mint_supply |  |
| SURVIVED | program/src/token.rs:68:22: replace \|\| with && in token_account |  |
| SURVIVED | program/src/token.rs:73:9: delete match arm 0 in token_account |  |
| SURVIVED | program/src/token.rs:95:26: replace + with * in initialize_metadata |  |
| SURVIVED | program/src/token.rs:95:34: replace + with * in initialize_metadata |  |
| SURVIVED | program/src/token.rs:95:45: replace + with * in initialize_metadata |  |
| SURVIVED | program/src/token.rs:95:58: replace + with * in initialize_metadata |  |
| SURVIVED | program/src/vault.rs:123:5: replace log_state with () |  |
| SURVIVED | program/src/vault.rs:51:23: replace \|\| with && in load |  |
| SURVIVED | program/src/vault.rs:51:53: replace \|\| with && in load |  |
| SURVIVED | create: mint non firmatario |  |
| SURVIVED | create: vault non verificato come PDA [vault, mint] |  |
| SURVIVED | donate: donatore non verificato |  |
| SURVIVED | donate: rimosso I2 |  |
| SURVIVED | donate: rimosso supply == S |  |
| SURVIVED | mint: rimosso signer/writable dell'utente |  |
| SURVIVED | mint: rimosso I2 con i lamport reali |  |
| SURVIVED | mint: rimosso supply == S dopo MintTo |  |
| SURVIVED | mint: rimosso supply == S prima |  |
| SURVIVED | mint: System program non verificato |  |
| SURVIVED | mint: programma token non verificato |  |
| SURVIVED | move_lamports: underflow saturato invece che errore |  |
| SURVIVED | redeem: rimosso il controllo che il delegato sia il vault |  |
| SURVIVED | redeem: rimosso signer/writable dell'utente |  |
| SURVIVED | redeem: rimosso I2 con i lamport reali |  |
| SURVIVED | redeem: rimosso supply == S dopo il burn |  |
| SURVIVED | redeem: rimosso supply == S prima |  |
| SURVIVED | redeem: rimosso il controllo del mint del token account |  |
| SURVIVED | redeem: programma token non verificato |  |
| SURVIVED | redeem: tesoreria non verificata |  |
| SURVIVED | sweep: rimosso I2 |  |
| SURVIVED | sweep: rimosso supply == S |  |
| SURVIVED | token_account: account non inizializzato accettato |  |
| SURVIVED | token_account: account non Token-2022 accettato |  |
| SURVIVED | mint_supply: mint non inizializzato accettato |  |
| SURVIVED | mint_supply: mint non verificato come Token-2022 |  |
| SURVIVED | vault::load: discriminatore e versione ignorati |  |
| SURVIVED | vault::load: vault non legato al mint (cross-market) |  |
| SURVIVED | vault::load: owner del vault non verificato |  |
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
| KILLED | program/src/lib.rs:57:9: delete match arm tag::CREATE in process_instruction | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:58:9: delete match arm tag::MINT in process_instruction | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:59:9: delete match arm tag::REDEEM in process_instruction | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:60:9: delete match arm tag::DONATE in process_instruction | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:61:9: delete match arm tag::SWEEP in process_instruction | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:80:25: replace < with <= in Reader<'a>::take | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:80:25: replace < with == in Reader<'a>::take | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:80:25: replace < with > in Reader<'a>::take | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:89:9: replace Reader<'a>::u16 -> Result<u16, ProgramError> with Ok(0) | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:89:9: replace Reader<'a>::u16 -> Result<u16, ProgramError> with Ok(1) | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:93:9: replace Reader<'a>::u64 -> Result<u64, ProgramError> with Ok(0) | frontend_transactions_run_on_the_program |
| KILLED | program/src/lib.rs:93:9: replace Reader<'a>::u64 -> Result<u64, ProgramError> with Ok(1) | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/create.rs:41:28: replace != with == in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/create.rs:55:38: replace + with - in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/mint.rs:49:28: replace > with < in process | lifecycle_with_sweep_and_reopen |
| KILLED | program/src/processor/mint.rs:49:28: replace > with == in process | lifecycle_with_sweep_and_reopen |
| KILLED | program/src/processor/mod.rs:16:8: delete ! in signer_writable | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/mod.rs:23:8: delete ! in writable | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/mod.rs:30:22: replace != with == in system_program | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/mod.rs:37:8: delete ! in treasury | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/mod.rs:45:15: replace == with != in move_lamports | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/mod.rs:76:16: replace == with != in create_account | create_with_prefunded_mint_and_vault |
| KILLED | program/src/processor/mod.rs:87:16: replace > with < in create_account | create_with_prefunded_mint_and_vault |
| KILLED | program/src/processor/mod.rs:87:16: replace > with == in create_account | create_with_prefunded_mint_and_vault |
| KILLED | program/src/processor/redeem.rs:40:19: replace != with == in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/redeem.rs:43:20: replace != with == in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/redeem.rs:46:31: replace != with == in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/redeem.rs:46:60: replace \|\| with && in process | redeem_needs_own_account_and_delegation |
| KILLED | program/src/processor/redeem.rs:46:85: replace < with <= in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/redeem.rs:46:85: replace < with == in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/processor/redeem.rs:46:85: replace < with > in process | redeem_needs_own_account_and_delegation |
| KILLED | program/src/processor/sweep.rs:19:26: replace != with == in process | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:100:21: replace + with * in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:100:21: replace + with - in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:101:12: replace += with *= in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:101:12: replace += with -= in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:102:21: replace + with * in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:102:21: replace + with - in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:103:12: replace += with *= in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:103:12: replace += with -= in initialize_metadata | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:125:28: replace + with * in revoke_metadata_authority | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:15:46: replace + with * | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:15:46: replace + with - | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:15:50: replace + with * | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:15:50: replace + with - | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:15:54: replace + with * | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:15:54: replace + with - | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:20:45: replace + with - in metadata_tlv_len | compute_units |
| KILLED | program/src/token.rs:20:50: replace + with - in metadata_tlv_len | compute_units |
| KILLED | program/src/token.rs:27:32: replace != with == in check_program | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:35:5: replace mint_supply -> Result<u64, ProgramError> with Ok(0) | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:35:5: replace mint_supply -> Result<u64, ProgramError> with Ok(1) | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:35:8: delete ! in mint_supply | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:39:16: replace < with > in mint_supply | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:39:30: replace != with == in mint_supply | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:47:27: replace != with == in check_supply | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:62:8: delete ! in token_account | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:68:16: replace < with <= in token_account | compute_units |
| KILLED | program/src/token.rs:68:16: replace < with == in token_account | compute_units |
| KILLED | program/src/token.rs:68:16: replace < with > in token_account | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:68:32: replace == with != in token_account | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:71:58: replace + with * in token_account | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:71:58: replace + with - in token_account | frontend_transactions_run_on_the_program |
| KILLED | program/src/token.rs:95:30: replace * with + in initialize_metadata | compute_units |
| KILLED | program/src/token.rs:95:30: replace * with / in initialize_metadata | compute_units |
| KILLED | program/src/token.rs:95:45: replace + with - in initialize_metadata | compute_units |
| KILLED | program/src/vault.rs:100:16: replace + with * in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:100:16: replace + with - in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:101:28: replace + with * in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:101:28: replace + with - in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:102:30: replace + with * in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:102:30: replace + with - in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:103:26: replace + with * in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:103:26: replace + with - in store | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:109:5: replace available -> Result<u64, ProgramError> with Ok(0) | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:109:5: replace available -> Result<u64, ProgramError> with Ok(1) | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:38:32: replace + with * in u128_at | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:38:32: replace + with - in u128_at | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:38:5: replace u128_at -> u128 with 0 | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:38:5: replace u128_at -> u128 with 1 | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:47:8: delete ! in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:51:16: replace != with == in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:51:36: replace != with == in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:51:69: replace != with == in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:56:65: replace + with * in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:57:56: replace + with * in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:57:56: replace + with - in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:59:21: replace != with == in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:63:68: replace + with * in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:63:68: replace + with - in load | vectors_scale_1e9 |
| KILLED | program/src/vault.rs:64:62: replace + with * in load | vectors_scale_1e9 |
| KILLED | program/src/vault.rs:64:62: replace + with - in load | vectors_scale_1e9 |
| KILLED | program/src/vault.rs:68:57: replace + with * in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:68:57: replace + with - in load | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:82:16: replace != with == in init | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:89:28: replace + with * in init | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:89:28: replace + with - in init | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:90:24: replace + with * in init | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:90:24: replace + with - in init | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:91:28: replace + with * in init | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:92:22: replace + with * in init | frontend_transactions_run_on_the_program |
| KILLED | program/src/vault.rs:92:22: replace + with - in init | frontend_transactions_run_on_the_program |
| KILLED | create: freeze authority al creator | create_mint_metadata_and_vault |
| KILLED | create: mint authority al creator invece che al vault | frontend_transactions_run_on_the_program |
| KILLED | create: metadati non validati | create_rejects_bad_parameters |
| KILLED | create: parametri non validati | create_rejects_bad_parameters |
| KILLED | create: update authority dei metadati non revocata | create_mint_metadata_and_vault |
| KILLED | donate: trasferito a − 1 | lifecycle_with_sweep_and_reopen |
| KILLED | mint: conia u + 1 token | frontend_transactions_run_on_the_program |
| KILLED | mint: la fee del creator non entra nel vault | create_with_prefunded_mint_and_vault |
| KILLED | mint: fee del protocollo non pagata | lifecycle_with_sweep_and_reopen |
| KILLED | mint: stato del vault non scritto | frontend_transactions_run_on_the_program |
| KILLED | mint: tesoreria non verificata (fee dirottabile) | rejects_unknown_treasury |
| KILLED | mint: l'utente paga 1 lamport in meno di backing | create_with_prefunded_mint_and_vault |
| KILLED | redeem: brucia u − 1 | frontend_transactions_run_on_the_program |
| KILLED | redeem: rimosso il controllo della quantità delegata | redeem_needs_own_account_and_delegation |
| KILLED | redeem: rimosso il controllo che il token account sia del firmatario | redeem_needs_own_account_and_delegation |
| KILLED | redeem: fee del protocollo non pagata (resta nel vault) | vectors_scale_1e9 |
| KILLED | redeem: stato del vault non scritto | frontend_transactions_run_on_the_program |
| KILLED | redeem: 1 lamport in più all'utente | vectors_scale_1e9 |
| KILLED | redeem: l'utente riceve il lordo (fee non trattenute) | lifecycle_with_sweep_and_reopen |
| KILLED | redeem: la tesoreria riceve anche la fee del creator | vectors_scale_1e9 |
| KILLED | sweep: 1 lamport oltre l'excess | frontend_transactions_run_on_the_program |
| KILLED | sweep: beneficiario non verificato (furto dell'excess) | lifecycle_with_sweep_and_reopen |
| KILLED | treasury(): qualunque account accettato come tesoreria | rejects_unknown_treasury |
| KILLED | vault::available: il rent conta come excess | create_with_prefunded_mint_and_vault |
