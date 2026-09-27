// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.30;

import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Bernie} from "../src/Bernie.sol";
import {BernieFactory} from "../src/BernieFactory.sol";

/// Differenziale (§17): i vettori SCALE 10¹⁸ del modello, eseguiti dai contratti.
/// Il test stesso è l'utente (riceve rimborsi e riscatti); creator e tesoreria sono fissi.
contract VectorsTest is Test {
    using stdJson for string;

    struct Cols {
        uint256[] first;
        uint256[] price;
        uint256[] p;
        uint256[] e;
        uint256[] op;
        uint256[] arg;
        uint256[] limit;
        uint256[] hasLimit;
        uint256[] ok;
        string[] err;
        uint256[] result;
        uint256[] k;
        uint256[] R;
        uint256[] Q;
        uint256[] S;
        uint256[] bal;
        uint256[] fc;
        uint256[] fp;
    }

    address constant TREASURY = address(0xFEE);
    address constant CREATOR = address(0xC0FFEE);
    BernieFactory factory;

    receive() external payable {}

    function setUp() public {
        factory = new BernieFactory(TREASURY);
        vm.deal(address(this), 1e40);
    }

    function load() internal view returns (Cols memory c) {
        string memory j = vm.readFile(string.concat(vm.projectRoot(), "/../vectors/scale_1e18.flat.json"));
        c.first = j.readUintArray(".first");
        c.price = j.readUintArray(".price");
        c.p = j.readUintArray(".p");
        c.e = j.readUintArray(".e");
        c.op = j.readUintArray(".op");
        c.arg = j.readUintArray(".arg");
        c.limit = j.readUintArray(".limit");
        c.hasLimit = j.readUintArray(".has_limit");
        c.ok = j.readUintArray(".ok");
        c.err = j.readStringArray(".err");
        c.result = j.readUintArray(".result");
        c.k = j.readUintArray(".k");
        c.R = j.readUintArray(".R");
        c.Q = j.readUintArray(".Q");
        c.S = j.readUintArray(".S");
        c.bal = j.readUintArray(".bal");
        c.fc = j.readUintArray(".fc");
        c.fp = j.readUintArray(".fp");
    }

    function selectorOf(string memory name) internal pure returns (bytes4) {
        return bytes4(keccak256(bytes(string.concat(name, "()"))));
    }

    function test_vectors_scale_1e18() public {
        Cols memory c = load();
        uint256 n = c.op.length;
        assertGt(n, 1000);
        Bernie token;
        for (uint256 i = 0; i < n; i++) {
            if (c.first[i] == 1) {
                vm.prank(CREATOR);
                token = Bernie(factory.create(c.price[i], uint16(c.p[i]), uint16(c.e[i]), "V", "V", bytes32(i)));
            }
            _step(token, c, i);
            _checkState(token, c, i);
        }
    }

    function _step(Bernie token, Cols memory c, uint256 i) internal {
        bool ok = c.ok[i] == 1;
        uint256 before = address(this).balance;
        bytes memory reason;
        bool success;
        if (c.op[i] == 1) {
            // Su EVM il limite di costo è msg.value; senza limite si manda il costo esatto.
            uint256 value = c.hasLimit[i] == 1 ? c.limit[i] : (ok ? c.result[i] : 1e30);
            try token.mint{value: value}(c.arg[i]) {
                success = true;
            } catch (bytes memory r) {
                reason = r;
            }
            if (success) assertEq(before - address(this).balance, c.result[i], "costo del mint");
        } else if (c.op[i] == 2) {
            try token.redeem(c.arg[i], c.hasLimit[i] == 1 ? c.limit[i] : 0) {
                success = true;
            } catch (bytes memory r) {
                reason = r;
            }
            if (success) assertEq(address(this).balance - before, c.result[i], "incasso del redeem");
        } else {
            try token.donate{value: c.arg[i]}() {
                success = true;
            } catch (bytes memory r) {
                reason = r;
            }
        }
        assertEq(success, ok, string.concat("esito del passo ", vm.toString(i)));
        if (!ok) assertEq(bytes4(reason), selectorOf(c.err[i]), string.concat("errore del passo ", vm.toString(i)));
    }

    function _checkState(Bernie token, Cols memory c, uint256 i) internal view {
        string memory at = string.concat(" al passo ", vm.toString(i));
        assertEq(token.k(), c.k[i], string.concat("k", at));
        assertEq(token.reserve(), c.R[i], string.concat("R", at));
        assertEq(token.residual(), c.Q[i], string.concat("Q", at));
        assertEq(token.totalSupply(), c.S[i], string.concat("S", at));
        assertEq(token.balanceOf(address(this)), c.S[i], string.concat("saldo token", at));
        // EVM: fee in pull, quindi nel contratto restano backing, resti, excess e fee dovute.
        assertEq(address(token).balance, c.bal[i] + c.fc[i] + c.fp[i], string.concat("saldo ETH", at));
        assertEq(token.feesOwed(CREATOR), c.fc[i], string.concat("fee creator", at));
        assertEq(token.feesOwed(TREASURY), c.fp[i], string.concat("fee tesoreria", at));
        assertEq(token.totalFeesOwed(), c.fc[i] + c.fp[i], string.concat("fee totali", at));
    }
}
