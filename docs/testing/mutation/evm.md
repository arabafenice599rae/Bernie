# Mutation testing: evm

Generato da `tools/mutation/mutate.py evm`. EQUIVALENT: 3, KILLED: 178

| Esito | Mutante | Rilevato da (o motivo dell'equivalenza) |
|---|---|---|
| EQUIVALENT | evm/src/Bernie.sol:157: riga cancellata `if (totalSupply() != s.S) revert InvariantViolated();` | EQUIVALENT: stessa riga di t:solvency-no-supply-check (cancellazione del controllo totalSupply() != s.S), non raggiungibile per costruzione |
| EQUIVALENT | fee: arrotondamento separato della quota protocollo | EQUIVALENT con le costanti v1.6: ⌊⌊40b/10⁴⌋·20/40⌋ = ⌊⌊b/250⌋/2⌋ = ⌊b/500⌋ = ⌊20b/10⁴⌋ (identità dei floor annidati). Con FEE_C ≠ FEE_P il mutante diventerebbe osservabile e va riesaminato |
| EQUIVALENT | solvenza: supply ERC-20 non confrontata con S | EQUIVALENT: _state() legge S da totalSupply() e _mint/_burn muovono esattamente u, quindi totalSupply() == s.S vale per costruzione; il controllo è difensivo e non raggiungibile senza modificare altro codice |
| KILLED | evm/src/Bernie.sol:107: riga cancellata `_checkSolvency(s, 0);` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:125: riga cancellata `if (excess == 0) revert NothingToClaim();` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/Bernie.sol:133: riga cancellata `if (amt == 0) revert NothingToClaim();` | test_claimFeesFor_pays_the_account_not_the_caller |
| KILLED | evm/src/Bernie.sol:159: riga cancellata `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:164: riga cancellata `if (to == address(this)) revert TransferToSelf();` | test_transfer_to_self_rejected |
| KILLED | evm/src/Bernie.sol:79: riga cancellata `if (msg.value < paid) revert Slippage();` | test_mint_rejects_underpayment |
| KILLED | evm/src/Bernie.sol:84: riga cancellata `_checkSolvency(s, refund);` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:97: riga cancellata `_checkSolvency(s, out);` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/BernieFactory.sol:32: riga cancellata `if (!any) revert NothingToClaim();` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:103: riga cancellata `checkAt(s, s0.k, scale);` | test_operations_refuse_corrupt_input_state |
| KILLED | evm/src/BernieMath.sol:120: riga cancellata `if (u == 0) revert ZeroAmount();` | test_failed_operations_leave_state_unchanged |
| KILLED | evm/src/BernieMath.sol:121: riga cancellata `if (u > s0.S) revert ExceedsSupply();` | test_failed_operations_leave_state_unchanged |
| KILLED | evm/src/BernieMath.sol:125: riga cancellata `if (full <= pen) revert Dust();` | test_dust_is_its_own_error |
| KILLED | evm/src/BernieMath.sol:129: riga cancellata `if (out == 0) revert ZeroPayout();` | test_vectors_scale_1e18 |
| KILLED | evm/src/BernieMath.sol:130: riga cancellata `if (out < minOut) revert Slippage();` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:139: riga cancellata `checkAt(s, s0.k, scale);` | test_operations_refuse_corrupt_input_state |
| KILLED | evm/src/BernieMath.sol:147: riga cancellata `if (a == 0) revert ZeroAmount();` | test_failed_operations_leave_state_unchanged |
| KILLED | evm/src/BernieMath.sol:148: riga cancellata `if (s0.S == 0) revert NoHolders();` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:151: riga cancellata `checkAt(s, s0.k, scale);` | test_operations_refuse_corrupt_input_state |
| KILLED | evm/src/BernieMath.sol:156: riga cancellata `if (price < MIN_PRICE \|\| price > MAX_PRICE) revert PriceOutOfRange();` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:157: riga cancellata `if (p < PEN_MIN \|\| p > PEN_MAX \|\| e > p) revert PenaltyOutOfRange();` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:159: riga cancellata `revert MetadataTooLong();` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:66: riga cancellata `checkAt(s, kPrev, SCALE);` | test_checkAt_rejects_each_invariant_alone |
| KILLED | evm/src/BernieMath.sol:91: riga cancellata `if (u == 0) revert ZeroAmount();` | test_mint_rejects_underpayment |
| KILLED | evm/src/Bernie.sol:124: `-` → `+` in `uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE` | test failed |
| KILLED | evm/src/Bernie.sol:124: `+` → `-` in `uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:124: `/` → `*` in `uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE` | test_reentrancy_during_claim_and_sweep |
| KILLED | evm/src/Bernie.sol:124: `-` → `+` in `uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:125: `==` → `!=` in `if (excess == 0) revert NothingToClaim();` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/Bernie.sol:133: `==` → `!=` in `if (amt == 0) revert NothingToClaim();` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/Bernie.sol:135: `-=` → `+=` in `totalFeesOwed -= amt;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/Bernie.sol:140: `+` → `-` in `return BernieMath.State(kInit + kGrowth, reserve, residual, totalSupply());` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/Bernie.sol:144: `-` → `+` in `kGrowth = s.k - kInit;` | test_double_redeem_and_transfer_attempts |
| KILLED | evm/src/Bernie.sol:150: `+=` → `-=` in `feesOwed[c] += f.creator;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/Bernie.sol:151: `+=` → `-=` in `feesOwed[treasury] += f.protocol;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/Bernie.sol:152: `+=` → `-=` in `totalFeesOwed += f.total;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/Bernie.sol:157: `!=` → `==` in `if (totalSupply() != s.S) revert InvariantViolated();` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/Bernie.sol:158: `-` → `+` in `uint256 bal = address(this).balance - pendingOut;` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:159: `*` → `/` in `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/Bernie.sol:159: `<` → `<=` in `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/Bernie.sol:159: `+` → `-` in `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:159: `+` → `-` in `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/Bernie.sol:159: `*` → `/` in `if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) rever` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/Bernie.sol:164: `==` → `!=` in `if (to == address(this)) revert TransferToSelf();` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/Bernie.sol:69: `+` → `-` in `return k0() + kGrowth;` | test_donate_raises_k |
| KILLED | evm/src/Bernie.sol:78: `+` → `-` in `uint256 paid = cost + f.total;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/Bernie.sol:79: `<` → `<=` in `if (msg.value < paid) revert Slippage();` | test_exact_mint_from_contract_without_receive |
| KILLED | evm/src/Bernie.sol:83: `-` → `+` in `uint256 refund = msg.value - paid;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/Bernie.sol:87: `>` → `>=` in `if (refund > 0) Address.sendValue(payable(msg.sender), refund);` | test_exact_mint_from_contract_without_receive |
| KILLED | evm/src/BernieFactory.sol:27: `<` → `<=` in `for (uint256 i; i < list.length; ++i) {` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieFactory.sol:28: `==` → `!=` in `if (Bernie(list[i]).feesOwed(account) == 0) continue;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:100: `+=` → `-=` in `s.S += u;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:101: `+=` → `-=` in `s.R += full;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:120: `==` → `!=` in `if (u == 0) revert ZeroAmount();` | test_double_redeem_and_transfer_attempts |
| KILLED | evm/src/BernieMath.sol:121: `>` → `>=` in `if (u > s0.S) revert ExceedsSupply();` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:123: `*` → `/` in `uint256 full = u * s.k;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:124: `*` → `/` in `uint256 pen = cdiv(full * p, BPS);` | test_vectors_scale_1e18 |
| KILLED | evm/src/BernieMath.sol:125: `<=` → `<` in `if (full <= pen) revert Dust();` | test_dust_is_its_own_error |
| KILLED | evm/src/BernieMath.sol:126: `-` → `+` in `g = (full - pen) / scale;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:126: `/` → `*` in `g = (full - pen) / scale;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:128: `-` → `+` in `out = g - f.total;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:129: `==` → `!=` in `if (out == 0) revert ZeroPayout();` | test_double_redeem_and_transfer_attempts |
| KILLED | evm/src/BernieMath.sol:130: `<` → `<=` in `if (out < minOut) revert Slippage();` | test_min_out_and_value_are_inclusive |
| KILLED | evm/src/BernieMath.sol:131: `-=` → `+=` in `s.S -= u;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:132: `-=` → `+=` in `s.R -= full;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:133: `==` → `!=` in `if (s.S == 0) {` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:136: `+=` → `-=` in `s.Q += full - g * scale;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:136: `-` → `+` in `s.Q += full - g * scale;` | test_min_out_and_value_are_inclusive |
| KILLED | evm/src/BernieMath.sol:136: `*` → `/` in `s.Q += full - g * scale;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:147: `==` → `!=` in `if (a == 0) revert ZeroAmount();` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:148: `==` → `!=` in `if (s0.S == 0) revert NoHolders();` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:149: `+` → `-` in `s = State(s0.k, s0.R, s0.Q + a * scale, s0.S);` | test_operations_refuse_corrupt_input_state |
| KILLED | evm/src/BernieMath.sol:149: `*` → `/` in `s = State(s0.k, s0.R, s0.Q + a * scale, s0.S);` | test_short_vault_refuses_every_operation |
| KILLED | evm/src/BernieMath.sol:156: `<` → `<=` in `if (price < MIN_PRICE \|\| price > MAX_PRICE) revert PriceOutOfRange();` | test_metadata_bounds_inclusive |
| KILLED | evm/src/BernieMath.sol:156: `\|\|` → `&&` in `if (price < MIN_PRICE \|\| price > MAX_PRICE) revert PriceOutOfRange();` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:156: `>` → `>=` in `if (price < MIN_PRICE \|\| price > MAX_PRICE) revert PriceOutOfRange();` | test_metadata_bounds_inclusive |
| KILLED | evm/src/BernieMath.sol:157: `<` → `<=` in `if (p < PEN_MIN \|\| p > PEN_MAX \|\| e > p) revert PenaltyOutOfRange();` | test_metadata_bounds_inclusive |
| KILLED | evm/src/BernieMath.sol:157: `\|\|` → `&&` in `if (p < PEN_MIN \|\| p > PEN_MAX \|\| e > p) revert PenaltyOutOfRange();` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:157: `>` → `>=` in `if (p < PEN_MIN \|\| p > PEN_MAX \|\| e > p) revert PenaltyOutOfRange();` | test_vectors_scale_1e18 |
| KILLED | evm/src/BernieMath.sol:157: `\|\|` → `&&` in `if (p < PEN_MIN \|\| p > PEN_MAX \|\| e > p) revert PenaltyOutOfRange();` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:157: `>` → `>=` in `if (p < PEN_MIN \|\| p > PEN_MAX \|\| e > p) revert PenaltyOutOfRange();` | test_vectors_scale_1e18 |
| KILLED | evm/src/BernieMath.sol:158: `==` → `!=` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | setUp |
| KILLED | evm/src/BernieMath.sol:158: `\|\|` → `&&` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:158: `>` → `>=` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | test_metadata_bounds_inclusive |
| KILLED | evm/src/BernieMath.sol:158: `\|\|` → `&&` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:158: `==` → `!=` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | setUp |
| KILLED | evm/src/BernieMath.sol:158: `\|\|` → `&&` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | test_create_validates_parameters |
| KILLED | evm/src/BernieMath.sol:158: `>` → `>=` in `if (name.length == 0 \|\| name.length > 32 \|\| symbol.length == 0 \|\| symbol.length ` | test_metadata_bounds_inclusive |
| KILLED | evm/src/BernieMath.sol:22: `1e18` → `1e17` in `uint256 internal constant SCALE = 1e18;` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:23: `10_000` → `9_999` in `uint256 internal constant BPS = 10_000;` | test_mint_refunds_overpayment_and_accrues_fees |
| KILLED | evm/src/BernieMath.sol:24: `20` → `21` in `uint256 internal constant FEE_C = 20;` | test_claimFeesFor_pays_the_account_not_the_caller |
| KILLED | evm/src/BernieMath.sol:25: `20` → `21` in `uint256 internal constant FEE_P = 20;` | test_claimFeesFor_pays_the_account_not_the_caller |
| KILLED | evm/src/BernieMath.sol:46: `==` → `!=` in `return a == 0 ? 0 : (a - 1) / b + 1;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:46: `-` → `+` in `return a == 0 ? 0 : (a - 1) / b + 1;` | test_mint_refunds_overpayment_and_accrues_fees |
| KILLED | evm/src/BernieMath.sol:46: `/` → `*` in `return a == 0 ? 0 : (a - 1) / b + 1;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:46: `+` → `-` in `return a == 0 ? 0 : (a - 1) / b + 1;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:51: `*` → `/` in `f.total = base * (FEE_C + FEE_P) / BPS;` | test_mint_refunds_overpayment_and_accrues_fees |
| KILLED | evm/src/BernieMath.sol:51: `+` → `-` in `f.total = base * (FEE_C + FEE_P) / BPS;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:51: `/` → `*` in `f.total = base * (FEE_C + FEE_P) / BPS;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:52: `*` → `/` in `f.protocol = f.total * FEE_P / (FEE_C + FEE_P);` | test_claimFeesFor_pays_the_account_not_the_caller |
| KILLED | evm/src/BernieMath.sol:52: `/` → `*` in `f.protocol = f.total * FEE_P / (FEE_C + FEE_P);` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:52: `+` → `-` in `f.protocol = f.total * FEE_P / (FEE_C + FEE_P);` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:53: `-` → `+` in `f.creator = f.total - f.protocol;` | test_claimAll_for_creator_across_tokens |
| KILLED | evm/src/BernieMath.sol:57: `==` → `!=` in `if (s.S == 0) return;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:58: `/` → `*` in `uint256 d = s.Q / s.S;` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:59: `+=` → `-=` in `s.k += d;` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:60: `+=` → `-=` in `s.R += d * s.S;` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:60: `*` → `/` in `s.R += d * s.S;` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:61: `-=` → `+=` in `s.Q -= d * s.S;` | test_failed_operations_leave_state_unchanged |
| KILLED | evm/src/BernieMath.sol:61: `*` → `/` in `s.Q -= d * s.S;` | test_donate_raises_k |
| KILLED | evm/src/BernieMath.sol:71: `!=` → `==` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:71: `*` → `/` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:71: `\|\|` → `&&` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_checkAt_rejects_each_invariant_alone |
| KILLED | evm/src/BernieMath.sol:71: `<` → `<=` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:71: `\|\|` → `&&` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_checkAt_rejects_each_invariant_alone |
| KILLED | evm/src/BernieMath.sol:71: `==` → `!=` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:71: `&&` → `\|\|` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:71: `!=` → `==` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:71: `\|\|` → `&&` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_checkAt_rejects_each_invariant_alone |
| KILLED | evm/src/BernieMath.sol:71: `+` → `-` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_checkAt_rejects_each_invariant_alone |
| KILLED | evm/src/BernieMath.sol:71: `!=` → `==` in `s.R != s.S * s.k \|\| s.k < kPrev \|\| (s.S == 0 && s.Q != 0) \|\| (s.R + s.Q) % scale` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:72: `\|\|` → `&&` in `\|\| (s.S != 0 && s.Q >= s.S)` | test_checkAt_rejects_each_invariant_alone |
| KILLED | evm/src/BernieMath.sol:72: `!=` → `==` in `\|\| (s.S != 0 && s.Q >= s.S)` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:72: `&&` → `\|\|` in `\|\| (s.S != 0 && s.Q >= s.S)` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:72: `>=` → `>` in `\|\| (s.S != 0 && s.Q >= s.S)` | test_checkAt_rejects_each_invariant_alone |
| KILLED | evm/src/BernieMath.sol:91: `==` → `!=` in `if (u == 0) revert ZeroAmount();` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:93: `==` → `!=` in `uint256 epen = s.S == 0 ? 0 : cdiv(u * s.k * e, BPS);` | test_mint_refunds_overpayment_and_accrues_fees |
| KILLED | evm/src/BernieMath.sol:93: `*` → `/` in `uint256 epen = s.S == 0 ? 0 : cdiv(u * s.k * e, BPS);` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:93: `*` → `/` in `uint256 epen = s.S == 0 ? 0 : cdiv(u * s.k * e, BPS);` | test_vectors_scale_1e18 |
| KILLED | evm/src/BernieMath.sol:94: `+=` → `-=` in `s.Q += epen;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:96: `*` → `/` in `uint256 full = u * s.k;` | test_claimAll_collects_every_token_in_one_call |
| KILLED | evm/src/BernieMath.sol:97: `+` → `-` in `c = cdiv(full + epen, scale);` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:99: `+=` → `-=` in `s.Q += c * scale - full - epen;` | test_lifecycle_claim_sweep_reopen |
| KILLED | evm/src/BernieMath.sol:99: `*` → `/` in `s.Q += c * scale - full - epen;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:99: `-` → `+` in `s.Q += c * scale - full - epen;` | test_arbitrary_clone_cannot_redirect_treasury |
| KILLED | evm/src/BernieMath.sol:99: `-` → `+` in `s.Q += c * scale - full - epen;` | test_lifecycle_claim_sweep_reopen |
| KILLED | fee: il creator riceve anche la quota del protocollo | test_claimFeesFor_pays_the_account_not_the_caller |
| KILLED | fee del creator accreditate al chiamante | test_claimAll_collects_every_token_in_one_call |
| KILLED | fee: totalFeesOwed sottostimato | test_lifecycle_claim_sweep_reopen |
| KILLED | fee: la tesoreria riceve la quota del creator | test_lifecycle_claim_sweep_reopen |
| KILLED | claimFees: guard di rientro rimosso | test_reentrancy_during_claim_and_sweep |
| KILLED | claim: totalFeesOwed non aggiornato | test_lifecycle_claim_sweep_reopen |
| KILLED | claim: feesOwed non azzerato (doppio prelievo) | test_claimAll_collects_every_token_in_one_call |
| KILLED | claimFeesFor paga il chiamante invece del titolare | test_claimAll_collects_every_token_in_one_call |
| KILLED | claimAll: ritira per il chiamante | test_claimAll_collects_every_token_in_one_call |
| KILLED | claimAll: nessun errore se non c'è nulla | test_claimAll_collects_every_token_in_one_call |
| KILLED | claimAll: token senza fee non saltati | test_claimAll_collects_every_token_in_one_call |
| KILLED | claimFeesFor: guard di rientro rimosso | test_reentrancy_during_claim_and_sweep |
| KILLED | donate: nessun controllo di solvenza | test_short_vault_refuses_every_operation |
| KILLED | factory: creator = tx.origin | test_claimAll_collects_every_token_in_one_call |
| KILLED | factory: token non registrato | test_create_is_deterministic_and_enumerable |
| KILLED | factory: parametri non validati | test_create_validates_parameters |
| KILLED | factory: salt non legato al chiamante (front-running dell'indirizzo) | test_create_is_deterministic_and_enumerable |
| KILLED | factory: implementazione con tesoreria diversa da treasury() | test_claimAll_collects_every_token_in_one_call |
| KILLED | absorb: floor → ceil | test_lifecycle_claim_sweep_reopen |
| KILLED | cdiv: ceil → floor ovunque | test_lifecycle_claim_sweep_reopen |
| KILLED | mint: asserzione I1–I6 rimossa | test_operations_refuse_corrupt_input_state |
| KILLED | create: e > p ammesso | test_create_validates_parameters |
| KILLED | create: prezzo minimo non verificato | test_create_validates_parameters |
| KILLED | mint: conia u + 1 | test_claimAll_collects_every_token_in_one_call |
| KILLED | mint: fee non accreditate | test_claimAll_collects_every_token_in_one_call |
| KILLED | mint: nessun controllo che msg.value copra il costo | test_mint_rejects_underpayment |
| KILLED | mint: guard di rientro rimosso | test_reentrancy_during_claim_and_sweep |
| KILLED | mint: nessun controllo di solvenza | test_short_vault_refuses_every_operation |
| KILLED | mint: le fee vengono rimborsate | test_claimAll_collects_every_token_in_one_call |
| KILLED | redeem: brucia u − 1 | test_lifecycle_claim_sweep_reopen |
| KILLED | redeem: fee non accreditate | test_vectors_scale_1e18 |
| KILLED | redeem: nessun burn (doppio prelievo) | test_lifecycle_claim_sweep_reopen |
| KILLED | redeem: guard di rientro rimosso | test_reentrancy_blocked |
| KILLED | redeem: nessun controllo di solvenza | test_short_vault_refuses_every_operation |
| KILLED | redeem: 1 wei in più all'utente | test_lifecycle_claim_sweep_reopen |
| KILLED | solvenza: fee dovute ignorate | test_short_vault_refuses_every_operation |
| KILLED | solvenza: pagamenti in uscita ignorati | test_short_vault_refuses_every_operation |
| KILLED | k non aggiornato in storage | test_donate_raises_k |
| KILLED | sweep: le fee dovute contano come excess | test_lifecycle_claim_sweep_reopen |
| KILLED | sweep: guard di rientro rimosso | test_reentrancy_during_claim_and_sweep |
| KILLED | sweep: l'excess va al chiamante invece che al creator | test_lifecycle_claim_sweep_reopen |
| KILLED | ERC-20: trasferimenti verso il contratto ammessi | test_transfer_to_self_rejected |
