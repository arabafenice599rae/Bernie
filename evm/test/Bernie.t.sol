// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import {Test, Vm} from "forge-std/Test.sol";
import {Clones} from "@openzeppelin/contracts/proxy/Clones.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {Bernie} from "../src/Bernie.sol";
import {BernieFactory} from "../src/BernieFactory.sol";
import {IBernie, IBernieFactory} from "../src/IBernie.sol";
import "../src/BernieMath.sol";

/// Riscatta di nuovo mentre riceve ETH: il reentrancy guard deve bloccarlo.
contract Reenter {
    Bernie public token;
    bool public armed;

    constructor(Bernie t) payable {
        token = t;
    }

    function enter(uint256 u) external {
        token.mint{value: address(this).balance}(u);
        armed = true;
        token.redeem(u / 2, 0);
    }

    receive() external payable {
        if (armed) {
            armed = false;
            token.redeem(1, 0);
        }
    }
}

contract BernieTest is Test {
    address constant TREASURY = address(0xFEE);
    address constant CREATOR = address(0xC0FFEE);
    address constant ALICE = address(0xA11CE);
    address constant BOB = address(0xB0B);
    uint256 constant P = 1e15; // 0,001 ETH per token intero
    BernieFactory factory;
    Bernie token;

    function setUp() public {
        factory = new BernieFactory(TREASURY);
        vm.prank(CREATOR);
        token = Bernie(factory.create(P, 200, 100, "Bernie Test", "BRN", bytes32("s")));
        vm.deal(ALICE, 1_000 ether);
        vm.deal(BOB, 1_000 ether);
    }

    function _mint(address who, uint256 u) internal returns (uint256 paid) {
        uint256 before = who.balance;
        vm.prank(who);
        token.mint{value: 100 ether}(u);
        paid = before - who.balance;
    }

    // ── factory ──

    function test_create_validates_parameters() public {
        vm.expectRevert(PriceOutOfRange.selector);
        factory.create(1e12 - 1, 200, 100, "N", "S", 0);
        vm.expectRevert(PriceOutOfRange.selector);
        factory.create(1e24 + 1, 200, 100, "N", "S", 0);
        vm.expectRevert(PenaltyOutOfRange.selector);
        factory.create(P, 99, 0, "N", "S", 0);
        vm.expectRevert(PenaltyOutOfRange.selector);
        factory.create(P, 1001, 0, "N", "S", 0);
        vm.expectRevert(PenaltyOutOfRange.selector);
        factory.create(P, 200, 201, "N", "S", 0);
        vm.expectRevert(MetadataTooLong.selector);
        factory.create(P, 200, 100, "", "S", 0);
        vm.expectRevert(MetadataTooLong.selector);
        factory.create(P, 200, 100, "123456789012345678901234567890123", "S", 0);
        vm.expectRevert(MetadataTooLong.selector);
        factory.create(P, 200, 100, "N", "12345678901", 0);
    }

    function test_create_is_deterministic_and_enumerable() public {
        bytes memory args = abi.encode(BOB, uint16(300), uint16(0), P, "Due", "DUE");
        address expected = Clones.predictDeterministicAddressWithImmutableArgs(
            factory.implementation(), args, keccak256(abi.encode(BOB, bytes32("x"))), address(factory)
        );
        vm.recordLogs();
        vm.prank(BOB);
        address t = factory.create(P, 300, 0, "Due", "DUE", bytes32("x"));
        assertEq(t, expected);
        assertEq(factory.count(), 2);
        assertEq(factory.tokens(1), t);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs[logs.length - 1].topics[0], BernieFactory.Created.selector);
        // Stesso salt, altro chiamante: indirizzo diverso, nessuna collisione.
        vm.prank(ALICE);
        assertTrue(factory.create(P, 300, 0, "Due", "DUE", bytes32("x")) != t);
    }

    function test_metadata_and_immutable_args() public view {
        assertEq(token.name(), "Bernie Test");
        assertEq(token.symbol(), "BRN");
        assertEq(token.decimals(), 18);
        assertEq(token.creator(), CREATOR);
        assertEq(token.penaltyBps(), 200);
        assertEq(token.entryBps(), 100);
        assertEq(token.k0(), P);
        assertEq(token.k(), P);
        assertEq(token.treasury(), TREASURY);
    }

    // ── mint ──

    function test_mint_refunds_overpayment_and_accrues_fees() public {
        uint256 paid = _mint(ALICE, 3e18); // 3 token a 0,001 ETH
        assertEq(paid, 3e15 + 3e15 * 40 / 10_000, "backing + fee 0.4%");
        assertEq(token.balanceOf(ALICE), 3e18);
        assertEq(token.feesOwed(CREATOR), 3e15 * 20 / 10_000);
        assertEq(token.feesOwed(TREASURY), 3e15 * 20 / 10_000);
        assertEq(address(token).balance, paid, "solo il costo resta nel contratto");
    }

    function test_mint_rejects_underpayment() public {
        vm.prank(ALICE);
        vm.expectRevert(Slippage.selector);
        token.mint{value: 1e15}(1e18);
        vm.prank(ALICE);
        vm.expectRevert(ZeroAmount.selector);
        token.mint{value: 1 ether}(0);
    }

    function test_events_in_order() public {
        vm.recordLogs();
        _mint(ALICE, 1e18);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        // Transfer (ERC-20), poi l'evento dell'operazione, poi State (§11).
        assertEq(logs.length, 3);
        assertEq(logs[1].topics[0], Bernie.Minted.selector);
        assertEq(logs[2].topics[0], Bernie.State.selector);
        (uint256 k, uint256 S, uint256 R, uint256 Q) = abi.decode(logs[2].data, (uint256, uint256, uint256, uint256));
        assertEq(k, token.k());
        assertEq(S, token.totalSupply());
        assertEq(R, token.reserve());
        assertEq(Q, token.residual());
    }

    // ── redeem, fee, sweep ──

    function test_lifecycle_claim_sweep_reopen() public {
        _mint(ALICE, 3e18);
        _mint(BOB, 1e18);
        assertGt(token.k(), P, "epen di Bob alza k");

        vm.prank(ALICE);
        token.redeem(1e18, 1);
        vm.prank(BOB);
        vm.expectRevert(Slippage.selector);
        token.redeem(1e18, 1 ether);

        // Fee in pull per creator e tesoreria.
        uint256 owed = token.feesOwed(CREATOR);
        uint256 before = CREATOR.balance;
        vm.prank(CREATOR);
        token.claimFees();
        assertEq(CREATOR.balance - before, owed);
        vm.prank(CREATOR);
        vm.expectRevert(NothingToClaim.selector);
        token.claimFees();
        vm.prank(TREASURY);
        token.claimFees();
        assertEq(token.totalFeesOwed(), 0);

        // Tutti escono: stato vuoto, excess al creator con sweep chiamato da chiunque.
        vm.prank(ALICE);
        token.redeem(2e18, 1);
        vm.prank(BOB);
        token.redeem(1e18, 1);
        assertEq(token.totalSupply(), 0);
        assertEq(token.reserve() + token.residual(), 0);
        uint256 kEnd = token.k();
        uint256 excess = address(token).balance - token.totalFeesOwed();
        assertGt(excess, 0, "penalita' dell'ultimo uscente");
        before = CREATOR.balance;
        vm.prank(BOB);
        token.sweep();
        assertEq(CREATOR.balance - before, excess);
        vm.expectRevert(NothingToClaim.selector);
        token.sweep();

        vm.expectRevert(NoHolders.selector);
        token.donate{value: 1}();
        _mint(BOB, 1e15);
        assertGe(token.k(), kEnd, "rientro al k raggiunto");
    }

    function test_donate_raises_k() public {
        _mint(ALICE, 1e18);
        uint256 k0 = token.k();
        vm.prank(BOB);
        token.donate{value: 1e15}();
        assertEq(token.k(), k0 + 1e15 * 1e18 / 1e18);
    }

    // ── sicurezza ──

    function test_transfer_to_self_rejected() public {
        _mint(ALICE, 1e18);
        vm.prank(ALICE);
        vm.expectRevert(TransferToSelf.selector);
        token.transfer(address(token), 1);
        vm.prank(ALICE);
        token.transfer(BOB, 1); // gli altri trasferimenti restano liberi
    }

    function test_no_receive() public {
        vm.prank(ALICE);
        (bool ok,) = address(token).call{value: 1}("");
        assertFalse(ok);
    }

    function test_reentrancy_blocked() public {
        Reenter r = new Reenter{value: 10 ether}(token);
        // Il riscatto annidato nel receive() fallisce; sendValue risale con l'errore del guard.
        vm.expectRevert(ReentrancyGuardTransient.ReentrancyGuardReentrantCall.selector);
        r.enter(1e18);
        assertEq(token.totalSupply(), 0, "tutto annullato");
    }

    function test_implementation_is_not_usable() public {
        Bernie impl = Bernie(factory.implementation());
        vm.expectRevert();
        impl.mint{value: 1 ether}(1);
    }

    // ── interfaccia di §11 ──

    function test_abi_matches_interface() public view {
        assertEq(Bernie.mint.selector, IBernie.mint.selector);
        assertEq(Bernie.redeem.selector, IBernie.redeem.selector);
        assertEq(Bernie.donate.selector, IBernie.donate.selector);
        assertEq(Bernie.claimFees.selector, IBernie.claimFees.selector);
        assertEq(Bernie.sweep.selector, IBernie.sweep.selector);
        assertEq(Bernie.k.selector, IBernie.k.selector);
        assertEq(Bernie.k0.selector, IBernie.k0.selector);
        assertEq(token.reserve.selector, IBernie.reserve.selector);
        assertEq(token.residual.selector, IBernie.residual.selector);
        assertEq(Bernie.penaltyBps.selector, IBernie.penaltyBps.selector);
        assertEq(Bernie.entryBps.selector, IBernie.entryBps.selector);
        assertEq(Bernie.creator.selector, IBernie.creator.selector);
        assertEq(token.feesOwed.selector, IBernie.feesOwed.selector);
        assertEq(token.totalFeesOwed.selector, IBernie.totalFeesOwed.selector);
        assertEq(Bernie.Minted.selector, IBernie.Minted.selector);
        assertEq(Bernie.Redeemed.selector, IBernie.Redeemed.selector);
        assertEq(Bernie.Donated.selector, IBernie.Donated.selector);
        assertEq(Bernie.State.selector, IBernie.State.selector);
        assertEq(BernieFactory.create.selector, IBernieFactory.create.selector);
        assertEq(BernieFactory.count.selector, IBernieFactory.count.selector);
        assertEq(factory.tokens.selector, IBernieFactory.tokens.selector);
        assertEq(BernieFactory.Created.selector, IBernieFactory.Created.selector);
        assertEq(factory.implementation.selector, IBernieFactory.implementation.selector);
        assertEq(token.treasury.selector, IBernie.treasury.selector);
    }
}
