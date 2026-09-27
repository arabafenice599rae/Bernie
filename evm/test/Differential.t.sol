// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.30;

import {Test, Vm} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {IERC20Errors} from "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";
import {Bernie} from "../src/Bernie.sol";
import {BernieFactory} from "../src/BernieFactory.sol";
import "../src/BernieMath.sol";

/// Differential e state-machine fuzzing (pre-audit, fasi 2–4, 19–21) sui contratti.
///
/// Riesegue le sequenze di `tools/redteam/gen.py evm` (oracolo: Appendice A) con più
/// utenti e confronta dopo *ogni* passo: esito (ok o uno degli errori ammessi), k, R, Q, S,
/// saldo del contratto, totalFeesOwed, ETH e token di ogni utente, fee dovute a utenti e
/// tesoreria, ETH ricevuti dalla tesoreria. In modo indipendente dall'oracolo verifica la
/// conservazione dell'ETH. Le sequenze si leggono da `evm/diff/` (`evm_*.json`); senza file
/// il test non fa nulla (la CI le genera prima).
contract DifferentialTest is Test {
    using stdJson for string;

    address constant TREASURY = address(0xFEE0);
    string[12] ERRS = [
        "ok", "ZeroAmount", "ExceedsSupply", "Dust", "ZeroPayout", "Slippage", "NoHolders", "NothingToClaim",
        "TransferToSelf", "InsufficientTokens", "InsufficientNative", "Overflow"
    ];

    struct Seq {
        uint256 n;
        uint256 steps;
        uint256[] op;
        uint256[] a;
        uint256[] b;
        uint256[] x;
        uint256[] y;
        uint256[] expect;
        uint256[] k;
        uint256[] R;
        uint256[] Q;
        uint256[] S;
        uint256[] avail;
        uint256[] tfo;
        uint256[] trin;
        uint256[][] native;
        uint256[][] tok;
        uint256[][] owed; // n + 1 colonne: utenti, poi tesoreria
    }

    function user(uint256 i) internal pure returns (address) {
        return address(uint160(0x10000 + i));
    }

    /// Indice (in ERRS) dell'errore osservato dai dati di revert.
    function errIndex(bytes memory data) internal pure returns (uint256) {
        bytes4 s = bytes4(data);
        if (s == ZeroAmount.selector) return 1;
        if (s == ExceedsSupply.selector) return 2;
        if (s == Dust.selector) return 3;
        if (s == ZeroPayout.selector) return 4;
        if (s == Slippage.selector) return 5;
        if (s == NoHolders.selector) return 6;
        if (s == NothingToClaim.selector) return 7;
        if (s == TransferToSelf.selector) return 8;
        if (s == IERC20Errors.ERC20InsufficientBalance.selector) return 9;
        if (s == bytes4(keccak256("Panic(uint256)"))) return 11;
        return type(uint256).max; // errore non previsto da nessun esito
    }

    function testDifferentialSequences() public {
        string memory dir = string.concat(vm.projectRoot(), "/diff");
        if (!vm.exists(dir)) return;
        Vm.DirEntry[] memory files = vm.readDir(dir);
        uint256 total;
        for (uint256 i; i < files.length; ++i) {
            string memory path = files[i].path;
            if (!vm.contains(path, "/evm_")) continue;
            total += this.runFile(path); // chiamata esterna: memoria nuova per ogni file
        }
        emit log_named_uint("passi confrontati", total);
    }

    function load(string memory j) internal pure returns (Seq memory q) {
        q.n = j.readUint(".users");
        q.steps = j.readUint(".steps");
        q.op = j.readUintArray(".op");
        q.a = j.readUintArray(".a");
        q.b = j.readUintArray(".b");
        q.x = j.readUintArray(".x");
        q.y = j.readUintArray(".y");
        q.expect = j.readUintArray(".expect");
        q.k = j.readUintArray(".k");
        q.R = j.readUintArray(".R");
        q.Q = j.readUintArray(".Q");
        q.S = j.readUintArray(".S");
        q.avail = j.readUintArray(".avail");
        q.tfo = j.readUintArray(".tfo");
        q.trin = j.readUintArray(".trin");
        q.native = new uint256[][](q.n);
        q.tok = new uint256[][](q.n);
        q.owed = new uint256[][](q.n + 1);
        for (uint256 i; i < q.n; ++i) {
            string memory idx = vm.toString(i);
            q.native[i] = j.readUintArray(string.concat(".native", idx));
            q.tok[i] = j.readUintArray(string.concat(".tok", idx));
            q.owed[i] = j.readUintArray(string.concat(".owed", idx));
        }
        q.owed[q.n] = j.readUintArray(string.concat(".owed", vm.toString(q.n)));
    }

    function runFile(string memory path) external returns (uint256) {
        string memory j = vm.readFile(path);
        Seq memory q = load(j);
        uint256[] memory wallets = j.readUintArray(".wallets");
        BernieFactory factory = new BernieFactory(TREASURY);
        vm.prank(user(0));
        Bernie t = Bernie(factory.create(j.readUint(".P"), uint16(j.readUint(".p")), uint16(j.readUint(".e")), "D", "D", 0));
        for (uint256 i; i < q.n; ++i) vm.deal(user(i), wallets[i]);
        vm.deal(TREASURY, 0);
        uint256 total0 = _total(t, q.n);
        for (uint256 s; s < q.steps; ++s) {
            (bool ok, bytes memory ret) = _exec(t, factory, q, s);
            uint256 mask = q.expect[s];
            string memory where = string.concat(path, " passo ", vm.toString(s));
            if (ok) {
                assertEq(mask, 1, string.concat("atteso errore, riuscito: ", where));
            } else {
                uint256 e = errIndex(ret);
                assertTrue(e < 12 && (mask >> e) & 1 == 1, string.concat("errore diverso: ", where));
            }
            _compare(t, q, s, where);
            assertEq(_total(t, q.n), total0, string.concat("ETH creato o distrutto: ", where));
        }
        return q.steps;
    }

    function _exec(Bernie t, BernieFactory f, Seq memory q, uint256 s) internal returns (bool ok, bytes memory ret) {
        uint256 op = q.op[s];
        address a = user(q.a[s]);
        if (op == 1) {
            vm.prank(a);
            (ok, ret) = address(t).call{value: q.y[s]}(abi.encodeCall(Bernie.mint, (q.x[s])));
        } else if (op == 2) {
            vm.prank(a);
            (ok, ret) = address(t).call(abi.encodeCall(Bernie.redeem, (q.x[s], q.y[s])));
        } else if (op == 3) {
            vm.prank(a);
            (ok, ret) = address(t).call{value: q.x[s]}(abi.encodeCall(Bernie.donate, ()));
        } else if (op == 4) {
            vm.prank(a);
            (ok, ret) = address(t).call(abi.encodeCall(Bernie.sweep, ()));
        } else if (op == 5) {
            vm.prank(a);
            (ok, ret) = address(t).call(abi.encodeCall(t.transfer, (user(q.b[s]), q.x[s])));
        } else if (op == 6) {
            vm.prank(a);
            (ok, ret) = address(t).call(abi.encodeCall(t.transfer, (address(t), q.x[s])));
        } else if (op == 7) {
            address who = q.a[s] == q.n ? TREASURY : a;
            vm.prank(who);
            (ok, ret) = address(t).call(abi.encodeCall(Bernie.claimFees, ()));
        } else {
            address who = q.a[s] == q.n ? TREASURY : a;
            address[] memory l = new address[](1);
            l[0] = address(t);
            vm.prank(user(q.b[s]));
            (ok, ret) = address(f).call(abi.encodeCall(BernieFactory.claimAll, (who, l)));
        }
    }

    function _compare(Bernie t, Seq memory q, uint256 s, string memory where) internal view {
        assertEq(t.k(), q.k[s], string.concat("k: ", where));
        assertEq(t.reserve(), q.R[s], string.concat("R: ", where));
        assertEq(t.residual(), q.Q[s], string.concat("Q: ", where));
        assertEq(t.totalSupply(), q.S[s], string.concat("S: ", where));
        assertEq(address(t).balance, q.avail[s], string.concat("saldo del contratto: ", where));
        assertEq(t.totalFeesOwed(), q.tfo[s], string.concat("totalFeesOwed: ", where));
        assertEq(TREASURY.balance, q.trin[s], string.concat("ETH della tesoreria: ", where));
        uint256 sumTok;
        for (uint256 i; i < q.n; ++i) {
            string memory idx = vm.toString(i);
            assertEq(user(i).balance, q.native[i][s], string.concat("ETH utente ", idx, ": ", where));
            uint256 tok = t.balanceOf(user(i));
            assertEq(tok, q.tok[i][s], string.concat("token utente ", idx, ": ", where));
            assertEq(t.feesOwed(user(i)), q.owed[i][s], string.concat("fee utente ", idx, ": ", where));
            sumTok += tok;
        }
        assertEq(t.feesOwed(TREASURY), q.owed[q.n][s], string.concat("fee tesoreria: ", where));
        assertEq(sumTok, t.totalSupply(), string.concat("somma dei saldi != supply: ", where));
    }

    function _total(Bernie t, uint256 n) internal view returns (uint256 sum) {
        sum = address(t).balance + TREASURY.balance;
        for (uint256 i; i < n; ++i) sum += user(i).balance;
    }
}
