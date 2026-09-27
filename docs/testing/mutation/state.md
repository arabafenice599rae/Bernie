# Mutation testing: state

Generato da `tools/mutation/mutate.py state`. KILLED: 155

| Esito | Mutante | Rilevato da (o motivo dell'equivalenza) |
|---|---|---|
| KILLED | state/src/error.rs:28:9: replace BernieError::code -> u32 with 0 | error_codes_match_section_7 |
| KILLED | state/src/error.rs:28:9: replace BernieError::code -> u32 with 1 | error_codes_match_section_7 |
| KILLED | state/src/error.rs:33:9: replace BernieError::from_code -> Option<Self> with None | error_codes_match_section_7 |
| KILLED | state/src/error.rs:33:9: replace BernieError::from_code -> Option<Self> with Some(Default::default()) | test failed |
| KILLED | state/src/error.rs:34:13: delete match arm 1 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:35:13: delete match arm 2 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:36:13: delete match arm 3 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:37:13: delete match arm 4 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:38:13: delete match arm 5 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:39:13: delete match arm 6 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:40:13: delete match arm 8 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:41:13: delete match arm 9 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:42:13: delete match arm 10 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:43:13: delete match arm 11 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:44:13: delete match arm 12 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:45:13: delete match arm 13 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:46:13: delete match arm 14 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:47:13: delete match arm 15 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:48:13: delete match arm 16 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:49:13: delete match arm 17 in BernieError::from_code | error_codes_match_section_7 |
| KILLED | state/src/error.rs:56:9: replace BernieError::name -> &'static str with "" | boundary_vectors_solana |
| KILLED | state/src/error.rs:56:9: replace BernieError::name -> &'static str with "xyzzy" | boundary_vectors_solana |
| KILLED | state/src/state.rs:100:17: replace * with + in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:100:17: replace * with / in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:100:25: replace / with % in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:100:25: replace / with * in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:100:34: replace + with * in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:100:34: replace + with - in split_fees | test failed |
| KILLED | state/src/state.rs:104:22: replace - with + in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:104:22: replace - with / in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:111:5: replace validate_params -> Result<()> with Ok(()) | create_params |
| KILLED | state/src/state.rs:111:8: delete ! in validate_params | create_params |
| KILLED | state/src/state.rs:118:52: replace \|\| with && in validate_penalties | create_params |
| KILLED | state/src/state.rs:118:5: replace validate_penalties -> Result<()> with Ok(()) | create_params |
| KILLED | state/src/state.rs:118:65: replace > with < in validate_penalties | absorb_boundaries_and_idempotence |
| KILLED | state/src/state.rs:118:65: replace > with == in validate_penalties | create_params |
| KILLED | state/src/state.rs:118:65: replace > with >= in validate_penalties | create_params |
| KILLED | state/src/state.rs:118:8: delete ! in validate_penalties | absorb_boundaries_and_idempotence |
| KILLED | state/src/state.rs:126:5: replace validate_metadata -> Result<()> with Ok(()) | metadata_limits |
| KILLED | state/src/state.rs:127:9: replace && with \|\| in validate_metadata | metadata_limits |
| KILLED | state/src/state.rs:128:22: replace <= with > in validate_metadata | metadata_limits |
| KILLED | state/src/state.rs:128:9: replace && with \|\| in validate_metadata | metadata_limits |
| KILLED | state/src/state.rs:140:9: replace Vault<SCALE>::create -> Result<Self> with Ok(Default::default()) | test failed |
| KILLED | state/src/state.rs:141:18: replace == with != in Vault<SCALE>::create | absorb_boundaries_and_idempotence |
| KILLED | state/src/state.rs:156:24: replace == with != in Vault<SCALE>::absorb | boundary_vectors_solana |
| KILLED | state/src/state.rs:156:9: replace Vault<SCALE>::absorb -> Result<()> with Ok(()) | boundary_vectors_solana |
| KILLED | state/src/state.rs:160:35: replace / with % in Vault<SCALE>::absorb | boundary_vectors_solana |
| KILLED | state/src/state.rs:160:35: replace / with * in Vault<SCALE>::absorb | boundary_vectors_solana |
| KILLED | state/src/state.rs:161:27: replace * with + in Vault<SCALE>::absorb | absorb_boundaries_and_idempotence |
| KILLED | state/src/state.rs:161:27: replace * with / in Vault<SCALE>::absorb | absorb_boundaries_and_idempotence |
| KILLED | state/src/state.rs:167:23: replace -= with += in Vault<SCALE>::absorb | boundary_vectors_solana |
| KILLED | state/src/state.rs:167:23: replace -= with /= in Vault<SCALE>::absorb | boundary_vectors_solana |
| KILLED | state/src/state.rs:173:9: replace Vault<SCALE>::check -> Result<()> with Ok(()) | check_rejects_each_invariant_in_isolation |
| KILLED | state/src/state.rs:174:40: replace == with != in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:175:25: replace >= with < in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:176:30: replace > with < in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:176:30: replace > with == in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:176:30: replace > with >= in Vault<SCALE>::check | check_rejects_each_invariant_in_isolation |
| KILLED | state/src/state.rs:176:34: replace \|\| with && in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:176:51: replace == with != in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:178:26: replace % with + in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:178:26: replace % with / in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:178:34: replace == with != in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:181:30: replace == with != in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:181:35: replace \|\| with && in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:181:52: replace < with <= in Vault<SCALE>::check | check_rejects_each_invariant_in_isolation |
| KILLED | state/src/state.rs:181:52: replace < with == in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:181:52: replace < with > in Vault<SCALE>::check | boundary_vectors_solana |
| KILLED | state/src/state.rs:182:15: replace && with \|\| in Vault<SCALE>::check | check_rejects_each_invariant_in_isolation |
| KILLED | state/src/state.rs:182:21: replace && with \|\| in Vault<SCALE>::check | check_rejects_each_invariant_in_isolation |
| KILLED | state/src/state.rs:182:27: replace && with \|\| in Vault<SCALE>::check | check_rejects_each_invariant_in_isolation |
| KILLED | state/src/state.rs:182:33: replace && with \|\| in Vault<SCALE>::check | check_rejects_each_invariant_in_isolation |
| KILLED | state/src/state.rs:191:9: replace Vault<SCALE>::liabilities -> Result<u64> with Ok(0) | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:191:9: replace Vault<SCALE>::liabilities -> Result<u64> with Ok(1) | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:195:18: replace / with % in Vault<SCALE>::liabilities | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:195:18: replace / with * in Vault<SCALE>::liabilities | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:200:22: replace >= with < in Vault<SCALE>::check_solvency | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:200:9: replace Vault<SCALE>::check_solvency -> Result<()> with Ok(()) | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:210:9: replace Vault<SCALE>::excess -> Result<u64> with Ok(0) | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:210:9: replace Vault<SCALE>::excess -> Result<u64> with Ok(1) | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:217:9: replace Vault<SCALE>::sweep_amount -> Result<u64> with Ok(0) | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:217:9: replace Vault<SCALE>::sweep_amount -> Result<u64> with Ok(1) | solvency_excess_and_sweep_thresholds |
| KILLED | state/src/state.rs:225:14: replace == with != in Vault<SCALE>::mint | boundary_vectors_solana |
| KILLED | state/src/state.rs:225:9: replace Vault<SCALE>::mint -> Result<Mint> with Ok(Default::default()) | test failed |
| KILLED | state/src/state.rs:231:32: replace == with != in Vault<SCALE>::mint | boundary_vectors_solana |
| KILLED | state/src/state.rs:247:24: replace match guard t <= max_cost with false in Vault<SCALE>::mint | boundary_vectors_solana |
| KILLED | state/src/state.rs:247:24: replace match guard t <= max_cost with true in Vault<SCALE>::mint | boundary_vectors_solana |
| KILLED | state/src/state.rs:247:26: replace <= with > in Vault<SCALE>::mint | boundary_vectors_solana |
| KILLED | state/src/state.rs:257:31: replace - with + in Vault<SCALE>::mint | boundary_vectors_solana |
| KILLED | state/src/state.rs:257:31: replace - with / in Vault<SCALE>::mint | boundary_vectors_solana |
| KILLED | state/src/state.rs:274:14: replace == with != in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:274:9: replace Vault<SCALE>::redeem -> Result<Redeem> with Ok(Default::default()) | test failed |
| KILLED | state/src/state.rs:277:14: replace > with < in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:277:14: replace > with == in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:277:14: replace > with >= in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:290:17: replace <= with > in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:293:30: replace - with + in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:293:30: replace - with / in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:293:37: replace / with % in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:293:37: replace / with * in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:295:21: replace - with + in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:295:21: replace - with / in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:296:16: replace == with != in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:299:16: replace < with <= in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:299:16: replace < with == in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:299:16: replace < with > in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:302:18: replace -= with += in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:302:18: replace -= with /= in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:307:21: replace == with != in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:311:29: replace - with + in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:311:29: replace - with / in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:311:45: replace * with + in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:311:45: replace * with / in Vault<SCALE>::redeem | boundary_vectors_solana |
| KILLED | state/src/state.rs:328:14: replace == with != in Vault<SCALE>::donate | boundary_vectors_solana |
| KILLED | state/src/state.rs:328:9: replace Vault<SCALE>::donate -> Result<()> with Ok(()) | boundary_vectors_solana |
| KILLED | state/src/state.rs:331:24: replace == with != in Vault<SCALE>::donate | boundary_vectors_solana |
| KILLED | state/src/state.rs:72:19: replace + with * in Mint::total_paid | boundary_vectors_solana |
| KILLED | state/src/state.rs:72:19: replace + with - in Mint::total_paid | boundary_vectors_solana |
| KILLED | state/src/state.rs:72:9: replace Mint::total_paid -> u64 with 0 | boundary_vectors_solana |
| KILLED | state/src/state.rs:72:9: replace Mint::total_paid -> u64 with 1 | boundary_vectors_solana |
| KILLED | state/src/state.rs:90:5: replace cdiv -> u128 with 0 | boundary_vectors_solana |
| KILLED | state/src/state.rs:90:5: replace cdiv -> u128 with 1 | boundary_vectors_solana |
| KILLED | state/src/state.rs:94:5: replace to_u64 -> Result<u64> with Ok(0) | boundary_vectors_solana |
| KILLED | state/src/state.rs:94:5: replace to_u64 -> Result<u64> with Ok(1) | boundary_vectors_solana |
| KILLED | state/src/state.rs:99:31: replace * with + in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:99:31: replace * with / in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:99:40: replace + with * in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:99:40: replace + with - in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:99:49: replace / with % in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:99:49: replace / with * in split_fees | boundary_vectors_solana |
| KILLED | state/src/state.rs:99:5: replace split_fees -> Fees with Default::default() | test failed |
| KILLED | absorb: floor(Q/S) → ceil | boundary_vectors_solana |
| KILLED | donate: a · (SCALE − 1) | boundary_vectors_solana |
| KILLED | penalità d'ingresso: ceil → floor | vectors_scale_10 |
| KILLED | penalità d'ingresso anche con S = 0 | boundary_vectors_solana |
| KILLED | fee: due arrotondamenti separati invece di uno (§5) | boundary_vectors_solana |
| KILLED | fee creator 0,20% → 0,21% | boundary_vectors_solana |
| KILLED | fee protocollo 0,20% → 0,30% | boundary_vectors_solana |
| KILLED | split fee: floor → ceil sulla quota protocollo | fee_table |
| KILLED | fee totale: floor → ceil | fee_table |
| KILLED | mint: nessun absorb prima dell'ingresso (epen diluita al nuovo entrante) | boundary_vectors_solana |
| KILLED | mint: c = ceil → floor | boundary_vectors_solana |
| KILLED | mint: base delle fee ceil → floor | boundary_vectors_solana |
| KILLED | mint: fee su backing + epen invece che sul backing | boundary_vectors_solana |
| KILLED | mint: R += full → R += full − epen | boundary_vectors_solana |
| KILLED | mint: resto di arrotondamento non in Q | boundary_vectors_solana |
| KILLED | mint: S += u → S += u − 1 | boundary_vectors_solana |
| KILLED | create: e > p ammesso | create_params |
| KILLED | create: MIN_PRICE escluso | create_params |
| KILLED | redeem: con S → 0 Q non azzerato | boundary_vectors_solana |
| KILLED | redeem: g floor → ceil | boundary_vectors_solana |
| KILLED | redeem: fee del creator non trattenuta | boundary_vectors_solana |
| KILLED | redeem: penalità ceil → floor | boundary_vectors_solana |
| KILLED | redeem: 1 sotto-unità di penalità persa | boundary_vectors_solana |
| KILLED | metadati: URI fino a 201 byte | metadata_limits |
