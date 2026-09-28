# Mutation testing: solana

Generato da `tools/mutation/mutate.py solana`. KILLED: 19, SURVIVED: 13

| Esito | Mutante | Rilevato da (o motivo dell'equivalenza) |
|---|---|---|
| SURVIVED | program/src/processor/mint.rs:49:28: replace > with >= in process |  |
| SURVIVED | program/src/processor/mod.rs:87:16: replace > with >= in create_account |  |
| SURVIVED | program/src/token.rs:39:16: replace < with <= in mint_supply |  |
| SURVIVED | program/src/token.rs:39:16: replace < with == in mint_supply |  |
| SURVIVED | program/src/token.rs:73:9: delete match arm 0 in token_account |  |
| SURVIVED | program/src/token.rs:95:26: replace + with * in initialize_metadata |  |
| SURVIVED | program/src/token.rs:95:34: replace + with * in initialize_metadata |  |
| SURVIVED | program/src/token.rs:95:45: replace + with * in initialize_metadata |  |
| SURVIVED | program/src/token.rs:95:58: replace + with * in initialize_metadata |  |
| SURVIVED | mint: rimosso supply == S dopo MintTo |  |
| SURVIVED | move_lamports: underflow saturato invece che errore |  |
| SURVIVED | redeem: rimosso supply == S dopo il burn |  |
| SURVIVED | sweep: rimosso I2 |  |
| KILLED | program/src/processor/create.rs:55:38: replace + with * in process | create_funds_exact_rent_for_final_sizes |
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
| KILLED | program/src/token.rs:20:50: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:57: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:57: replace + with - in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 0 | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:5: replace metadata_tlv_len -> usize with 1 | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/token.rs:20:7: replace + with * in metadata_tlv_len | create_funds_exact_rent_for_final_sizes |
| KILLED | program/src/vault.rs:123:5: replace log_state with () | state_log_matches_vault_after_every_operation |
