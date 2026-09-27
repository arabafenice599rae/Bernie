// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.30;

import {Test} from "forge-std/Test.sol";
import {Clones} from "@openzeppelin/contracts/proxy/Clones.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {Bernie} from "../src/Bernie.sol";
import {BernieFactory} from "../src/BernieFactory.sol";
import "../src/BernieMath.sol";

/// Pre-audit, fasi 5–15, 18, 21 (EVM). Ogni test attacca una proprietà della specifica e
/// verifica il rifiuto con l'errore previsto e lo stato invariato. Aggiunti dopo il
/// mutation testing: i mutanti sopravvissuti indicavano proprio queste lacune
/// (docs/testing/PreAuditReport.md).

/// Espone la library con SCALE come parametro, per i test delle guardie su stati corrotti.
contract MathHarness {
    function checkAt(BernieMath.State memory s, uint256 kPrev, uint256 scale) external pure {
        BernieMath.checkAt(s, kPrev, scale);
    }

    function check(BernieMath.State memory s, uint256 kPrev) external pure {
        BernieMath.check(s, kPrev);
    }

    function mintAt(BernieMath.State memory s, uint256 e, uint256 u, uint256 scale)
        external
        pure
        returns (BernieMath.State memory r, uint256 c, BernieMath.Fees memory f)
    {
        return BernieMath.mintAt(s, e, u, scale);
    }

    function redeemAt(BernieMath.State memory s, uint256 p, uint256 u, uint256 minOut, uint256 scale)
        external
        pure
        returns (BernieMath.State memory r, uint256 g, uint256 out, BernieMath.Fees memory f)
    {
        return BernieMath.redeemAt(s, p, u, minOut, scale);
    }

    function donateAt(BernieMath.State memory s, uint256 a, uint256 scale) external pure returns (BernieMath.State memory) {
        return BernieMath.donateAt(s, a, scale);
    }

    function validate(uint256 price, uint256 p, uint256 e, bytes memory n, bytes memory s) external pure {
        BernieMath.validate(price, p, e, n, s);
    }
}

/// Contratto che rientra nel token quando riceve ETH. `mode` sceglie la chiamata.
contract Attacker {
    Bernie public token;
    BernieFactory public factory;
    uint8 public mode; // 0 niente, 1 mint, 2 redeem, 3 donate, 4 sweep, 5 claimFees, 6 claimFeesFor, 7 claimAll
    bool public armed;

    constructor(BernieFactory f) payable {
        factory = f;
    }

    function setToken(Bernie t) external {
        token = t;
    }

    function arm(uint8 m) external {
        mode = m;
        armed = true;
    }

    function create(uint256 price) external returns (Bernie t) {
        t = Bernie(factory.create(price, 200, 100, "Attacco", "ATK", bytes32(uint256(uint160(address(this))))));
        token = t;
    }

    function doMint(uint256 u, uint256 value) external {
        token.mint{value: value}(u);
    }

    function doRedeem(uint256 u) external {
        token.redeem(u, 0);
    }

    function doClaim() external {
        token.claimFees();
    }

    function doClaimFor() external {
        token.claimFeesFor(address(this));
    }

    function doClaimAll() external {
        address[] memory l = new address[](1);
        l[0] = address(token);
        factory.claimAll(address(this), l);
    }

    function doSweep() external {
        token.sweep();
    }

    receive() external payable {
        if (!armed) return;
        armed = false;
        if (mode == 1) token.mint{value: 1 ether}(1e18);
        else if (mode == 2) token.redeem(1, 0);
        else if (mode == 3) token.donate{value: 1}();
        else if (mode == 4) token.sweep();
        else if (mode == 5) token.claimFees();
        else if (mode == 6) token.claimFeesFor(address(this));
        else if (mode == 7) {
            address[] memory l = new address[](1);
            l[0] = address(token);
            factory.claimAll(address(this), l);
        }
    }
}

/// Senza receive(): un mint pagato esatto non deve tentare nessun rimborso.
contract NoReceive {
    function mintExact(Bernie t, uint256 u, uint256 value) external {
        t.mint{value: value}(u);
    }

    constructor() payable {}
}

contract AdversarialTest is Test {
    address constant TREASURY = address(0xFEE);
    address constant CREATOR = address(0xC0FFEE);
    address constant ALICE = address(0xA11CE);
    address constant BOB = address(0xB0B);
    address constant EVE = address(0xE7E);
    uint256 constant P = 1e15;
    uint256 constant SC = 1e18;

    BernieFactory factory;
    Bernie token;
    MathHarness mh;

    function setUp() public {
        factory = new BernieFactory(TREASURY);
        vm.prank(CREATOR);
        token = Bernie(factory.create(P, 200, 100, "Bernie Test", "BRN", bytes32("s")));
        mh = new MathHarness();
        vm.deal(ALICE, 1000 ether);
        vm.deal(BOB, 1000 ether);
        vm.deal(EVE, 1000 ether);
    }

    function _mint(address who, uint256 u) internal {
        vm.prank(who);
        token.mint{value: 100 ether}(u);
    }

    struct Snap {
        uint256 k;
        uint256 S;
        uint256 R;
        uint256 Q;
        uint256 bal;
        uint256 fees;
        uint256 alice;
        uint256 bob;
        uint256 eve;
        uint256 creator;
        uint256 treasury;
    }

    function _snap() internal view returns (Snap memory s) {
        s = Snap(
            token.k(), token.totalSupply(), token.reserve(), token.residual(), address(token).balance,
            token.totalFeesOwed(), token.balanceOf(ALICE), token.balanceOf(BOB), token.balanceOf(EVE),
            token.feesOwed(CREATOR), token.feesOwed(TREASURY)
        );
    }

    function _same(Snap memory a, Snap memory b) internal pure {
        assertEq(abi.encode(a), abi.encode(b), "stato cambiato da un'operazione fallita");
    }

    // ── fase 4/5: guardie di BernieMath su stati corrotti (I1–I6 in isolamento) ──

    function test_checkAt_rejects_each_invariant_alone() public {
        // canonico: k 7, S 4, R 28, Q 2, SCALE 10
        mh.checkAt(BernieMath.State(7, 28, 2, 4), 7, 10);
        mh.checkAt(BernieMath.State(7, 0, 0, 0), 7, 10);
        BernieMath.State[6] memory bad = [
            BernieMath.State(7, 18, 2, 4), //   I1: R ≠ S·k
            BernieMath.State(7, 28, 2, 4), //   I3: k < kPrev (kPrev 8 sotto)
            BernieMath.State(7, 0, 10, 0), //   I4: S = 0, Q > 0
            BernieMath.State(7, 28, 3, 4), //   I5: R + Q non multiplo di SCALE
            BernieMath.State(4, 16, 4, 4), //   I6: Q = S
            BernieMath.State(4, 16, 14, 4) //   I6: Q > S
        ];
        uint256[6] memory kPrev = [uint256(7), 8, 7, 7, 4, 4];
        for (uint256 i; i < 6; ++i) {
            vm.expectRevert(InvariantViolated.selector);
            mh.checkAt(bad[i], kPrev[i], 10);
        }
        // check() usa SCALE = 1e18
        mh.check(BernieMath.State(1e18, 4e18, 0, 4), 1e18);
        vm.expectRevert(InvariantViolated.selector);
        mh.check(BernieMath.State(1e18, 4e18, 1, 4), 1e18);
    }

    function test_operations_refuse_corrupt_input_state() public {
        // Solo I1 violata in ingresso (R = 378 ≠ S·k = 280; R + Q = 380, Q < S): i controlli
        // di dominio passano, quindi solo l'asserzione finale può rifiutare.
        BernieMath.State memory s = BernieMath.State(70, 378, 2, 4);
        vm.expectRevert(InvariantViolated.selector);
        mh.mintAt(s, 100, 3, 10);
        vm.expectRevert(InvariantViolated.selector);
        mh.redeemAt(s, 200, 1, 0, 10);
        vm.expectRevert(InvariantViolated.selector);
        mh.donateAt(s, 5, 10);
    }

    // ── fase 5/6: soglie e arrotondamenti ──

    function test_dust_is_its_own_error() public {
        // SCALE 10, k 1, u 1, p 100: full 1, pen ⌈1·100/10⁴⌉ = 1 ⇒ full ≤ pen ⇒ Dust (non ZeroPayout)
        vm.expectRevert(Dust.selector);
        mh.redeemAt(BernieMath.State(1, 10, 0, 10), 100, 1, 0, 10);
    }

    function test_min_out_and_value_are_inclusive() public {
        _mint(ALICE, 10e18);
        (,, uint256 out,) = mh.redeemAt(
            BernieMath.State(token.k(), token.reserve(), token.residual(), token.totalSupply()), 200, 1e18, 0, SC
        );
        uint256 snap = vm.snapshotState();
        vm.prank(ALICE);
        vm.expectRevert(Slippage.selector);
        token.redeem(1e18, out + 1);
        vm.prank(ALICE);
        token.redeem(1e18, out); // min_out = out passa
        vm.revertToState(snap);
        // msg.value = costo esatto passa, un wei in meno no
        (, uint256 c, BernieMath.Fees memory f) = mh.mintAt(
            BernieMath.State(token.k(), token.reserve(), token.residual(), token.totalSupply()), 100, 1e18, SC
        );
        vm.prank(BOB);
        vm.expectRevert(Slippage.selector);
        token.mint{value: c + f.total - 1}(1e18);
        vm.prank(BOB);
        token.mint{value: c + f.total}(1e18);
    }

    function test_metadata_bounds_inclusive() public {
        mh.validate(1e12, 100, 0, "12345678901234567890123456789012", "1234567890");
        mh.validate(1e24, 1000, 1000, "N", "S");
        vm.expectRevert(MetadataTooLong.selector);
        mh.validate(1e12, 100, 0, "123456789012345678901234567890123", "S");
        vm.expectRevert(MetadataTooLong.selector);
        mh.validate(1e12, 100, 0, "N", "12345678901");
        vm.expectRevert(MetadataTooLong.selector);
        mh.validate(1e12, 100, 0, "N", "");
    }

    // ── fase 14: rientri, uno per ogni percorso che invia ETH ──

    function _attacker() internal returns (Attacker a) {
        a = new Attacker{value: 100 ether}(factory);
        a.setToken(token);
    }

    /// Errore atteso per il rientro `m`: il guard del token, tranne claimAll annidato senza fee
    /// da ritirare, che la factory ferma prima di chiamare il token (NothingToClaim).
    function _reentryError(uint8 m) internal pure returns (bytes4) {
        return m == 7 ? NothingToClaim.selector : ReentrancyGuardTransient.ReentrancyGuardReentrantCall.selector;
    }

    function test_reentrancy_during_mint_refund() public {
        Attacker a = _attacker();
        for (uint8 m = 1; m <= 7; ++m) {
            a.arm(m);
            Snap memory s0 = _snap();
            vm.expectRevert(_reentryError(m));
            a.doMint(1e18, 10 ether); // rimborso ⇒ receive ⇒ rientro
            _same(s0, _snap());
        }
    }

    function test_reentrancy_during_redeem_payout() public {
        Attacker a = _attacker();
        a.arm(0);
        a.doMint(10e18, 1 ether);
        for (uint8 m = 1; m <= 7; ++m) {
            a.arm(m);
            Snap memory s0 = _snap();
            vm.expectRevert(_reentryError(m));
            a.doRedeem(1e18);
            _same(s0, _snap());
        }
    }

    function test_reentrancy_during_claim_and_sweep() public {
        Attacker a = new Attacker{value: 100 ether}(factory);
        Bernie t = a.create(P); // l'attaccante è il creator: riceve fee e excess
        vm.prank(ALICE);
        t.mint{value: 10 ether}(5e18);
        vm.prank(ALICE);
        t.redeem(1e18, 0);
        vm.deal(address(t), address(t).balance + 1 ether); // excess per lo sweep
        for (uint8 m = 1; m <= 7; ++m) {
            // durante claim le fee sono già azzerate (CEI): claimAll annidato non trova nulla
            uint256 fees0 = t.feesOwed(address(a));
            uint256 bal0 = address(t).balance;
            a.arm(m);
            vm.expectRevert(_reentryError(m));
            a.doClaim();
            a.arm(m);
            vm.expectRevert(_reentryError(m));
            a.doClaimFor();
            a.arm(m);
            vm.expectRevert(_reentryError(m));
            a.doClaimAll();
            a.arm(m);
            // durante sweep le fee dell'attaccante sono ancora dovute: claimAll annidato
            // raggiunge il token e trova il guard
            vm.expectRevert(ReentrancyGuardTransient.ReentrancyGuardReentrantCall.selector);
            a.doSweep();
            assertEq(t.feesOwed(address(a)), fees0, "fee intatte");
            assertEq(address(t).balance, bal0, "nessun ETH uscito");
        }
        // senza rientro tutto funziona
        a.arm(0);
        uint256 owed = t.feesOwed(address(a));
        uint256 b0 = address(a).balance;
        a.doClaim();
        assertEq(address(a).balance - b0, owed);
    }

    function test_exact_mint_from_contract_without_receive() public {
        NoReceive nr = new NoReceive{value: 10 ether}();
        (, uint256 c, BernieMath.Fees memory f) =
            mh.mintAt(BernieMath.State(token.k(), token.reserve(), token.residual(), token.totalSupply()), 100, 1e18, SC);
        nr.mintExact(token, 1e18, c + f.total); // rimborso 0: nessuna chiamata all'utente
        assertEq(token.balanceOf(address(nr)), 1e18);
    }

    // ── fasi 3/18: solvenza e supply come ultima difesa ──

    function test_short_vault_refuses_every_operation() public {
        _mint(ALICE, 10e18);
        _mint(BOB, 5e18);
        vm.prank(ALICE);
        token.redeem(1e18, 0);
        vm.deal(address(token), address(token).balance - 1); // manca un wei
        Snap memory s0 = _snap();
        vm.prank(BOB);
        vm.expectRevert(InvariantViolated.selector);
        token.mint{value: 1 ether}(1e17); // con rimborso
        vm.prank(ALICE);
        vm.expectRevert(InvariantViolated.selector);
        token.redeem(1e18, 0);
        vm.prank(BOB);
        vm.expectRevert(InvariantViolated.selector);
        token.donate{value: 1}();
        vm.expectRevert();
        token.sweep();
        _same(s0, _snap());
    }

    // ── fase 11: sweep ──

    function test_sweep_pays_only_the_creator_and_only_once() public {
        _mint(ALICE, 10e18);
        _mint(BOB, 3e18);
        vm.prank(ALICE);
        token.redeem(2e18, 0);
        uint256 gift = 123_456_789;
        vm.deal(address(token), address(token).balance + gift); // ETH forzato con S > 0
        Snap memory s0 = _snap();
        uint256 c0 = CREATOR.balance;
        uint256 e0 = EVE.balance;
        vm.prank(EVE); // chiunque può chiamarlo
        token.sweep();
        assertEq(CREATOR.balance - c0, gift, "excess esatto al creator");
        assertEq(EVE.balance, e0, "il chiamante non riceve nulla");
        Snap memory s1 = _snap();
        s0.bal -= gift;
        _same(s0, s1); // k, S, R, Q e fee non cambiano
        vm.expectRevert(NothingToClaim.selector);
        token.sweep(); // secondo sweep: niente
    }

    // ── fase 15: cloni, factory, inizializzazione ──

    function test_arbitrary_clone_cannot_redirect_treasury() public {
        // Chiunque può clonare l'implementazione con argomenti propri: la tesoreria resta
        // quella dell'implementazione, e il clone non entra nell'elenco della factory.
        Bernie impl = Bernie(factory.implementation());
        Bernie fake = Bernie(
            Clones.cloneWithImmutableArgs(address(impl), abi.encode(EVE, uint16(1), uint16(9999), P, "F", "F"))
        );
        assertEq(fake.treasury(), TREASURY);
        assertEq(factory.count(), 1);
        assertEq(factory.tokens(0), address(token));
        // parametri fuori dai limiti: nessuna validazione nel clone arbitrario, ma la tesoreria
        // riceve comunque la sua quota e il creator falso solo la propria
        vm.prank(ALICE);
        fake.mint{value: 1 ether}(1e18);
        assertGt(fake.feesOwed(TREASURY), 0);
    }

    function test_implementation_and_clones_have_no_initializer() public {
        Bernie impl = Bernie(factory.implementation());
        vm.expectRevert();
        impl.mint{value: 1 ether}(1);
        vm.expectRevert();
        impl.creator();
        // nessuna funzione di inizializzazione da chiamare prima del creator
        (bool ok,) = address(token).call(abi.encodeWithSignature("initialize()"));
        assertFalse(ok);
        assertEq(token.creator(), CREATOR);
    }

    function test_salt_is_bound_to_sender() public {
        vm.prank(CREATOR);
        vm.expectRevert(); // stesso chiamante, stesso salt: collisione
        factory.create(P, 200, 100, "Bernie Test", "BRN", bytes32("s"));
        vm.prank(EVE); // stesso salt, altro chiamante: indirizzo diverso, creator = EVE
        Bernie t = Bernie(factory.create(P, 200, 100, "Bernie Test", "BRN", bytes32("s")));
        assertTrue(address(t) != address(token));
        assertEq(t.creator(), EVE);
        assertEq(token.creator(), CREATOR);
    }

    function test_creator_is_msg_sender_not_origin() public {
        vm.prank(BOB, ALICE); // msg.sender BOB, tx.origin ALICE
        Bernie t = Bernie(factory.create(P, 200, 100, "Due", "DUE", bytes32("o")));
        assertEq(t.creator(), BOB);
    }

    // ── fase 18: furto e doppio prelievo ──

    function test_fee_theft_attempts() public {
        _mint(ALICE, 10e18);
        uint256 owedC = token.feesOwed(CREATOR);
        uint256 owedT = token.feesOwed(TREASURY);
        vm.startPrank(EVE);
        vm.expectRevert(NothingToClaim.selector);
        token.claimFees();
        vm.expectRevert(NothingToClaim.selector);
        token.claimFeesFor(EVE);
        address[] memory l = new address[](1);
        l[0] = address(token);
        vm.expectRevert(NothingToClaim.selector);
        factory.claimAll(EVE, l);
        uint256 e0 = EVE.balance;
        token.claimFeesFor(CREATOR); // paga il creator, non EVE
        factory.claimAll(TREASURY, l); // paga la tesoreria, non EVE
        vm.stopPrank();
        assertEq(EVE.balance, e0);
        assertEq(CREATOR.balance, owedC);
        assertEq(TREASURY.balance, owedT);
        vm.expectRevert(NothingToClaim.selector);
        token.claimFeesFor(CREATOR); // doppio prelievo
    }

    function test_double_redeem_and_transfer_attempts() public {
        _mint(ALICE, 2e18);
        vm.startPrank(ALICE);
        token.redeem(2e18, 0);
        vm.expectRevert(); // secondo riscatto degli stessi token
        token.redeem(1, 0);
        vm.stopPrank();
        _mint(BOB, 1e18);
        vm.prank(EVE);
        vm.expectRevert(); // senza allowance
        token.transferFrom(BOB, EVE, 1);
        vm.prank(BOB);
        vm.expectRevert(TransferToSelf.selector);
        token.transfer(address(token), 1);
    }

    function test_failed_operations_leave_state_unchanged() public {
        _mint(ALICE, 3e18);
        _mint(BOB, 1e18);
        Snap memory s0 = _snap();
        vm.startPrank(ALICE);
        vm.expectRevert(ZeroAmount.selector);
        token.mint{value: 1 ether}(0);
        vm.expectRevert(Slippage.selector);
        token.mint{value: 1}(1e18);
        vm.expectRevert(ZeroAmount.selector);
        token.redeem(0, 0);
        vm.expectRevert(ExceedsSupply.selector);
        token.redeem(5e18, 0);
        vm.expectRevert(Slippage.selector);
        token.redeem(1e18, type(uint256).max);
        vm.expectRevert(ZeroAmount.selector);
        token.donate{value: 0}();
        vm.expectRevert(); // più del proprio saldo (ma meno della supply)
        token.redeem(3e18 + 1, 0);
        vm.stopPrank();
        _same(s0, _snap());
    }
}
