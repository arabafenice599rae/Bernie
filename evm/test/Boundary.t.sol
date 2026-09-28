// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.30;

import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import "../src/BernieMath.sol";

/// Chiamate esterne alla library: servono try/catch e dati di revert.
contract BoundaryHarness {
    function mint(BernieMath.State memory s, uint256 e, uint256 u)
        external
        pure
        returns (BernieMath.State memory r, uint256 paid, uint256 fc, uint256 fp)
    {
        uint256 c;
        BernieMath.Fees memory f;
        (r, c, f) = BernieMath.mint(s, e, u);
        return (r, c + f.total, f.creator, f.protocol);
    }

    function redeem(BernieMath.State memory s, uint256 p, uint256 u, uint256 minOut)
        external
        pure
        returns (BernieMath.State memory r, uint256 out, uint256 fc, uint256 fp)
    {
        BernieMath.Fees memory f;
        (r,, out, f) = BernieMath.redeem(s, p, u, minOut);
        return (r, out, f.creator, f.protocol);
    }

    function donate(BernieMath.State memory s, uint256 a) external pure returns (BernieMath.State memory) {
        return BernieMath.donate(s, a);
    }

    function absorb(BernieMath.State memory s) external pure returns (BernieMath.State memory) {
        BernieMath.absorb(s);
        return s;
    }
}

/// Vettori di confine EVM (pre-audit, fasi 5–9): `vectors/boundary_evm.json`, generati da
/// `tools/redteam/boundary.py` con l'oracolo dell'Appendice A (SCALE 10¹⁸, uint256).
contract BoundaryTest is Test {
    using stdJson for string;

    BoundaryHarness h;

    function setUp() public {
        h = new BoundaryHarness();
    }

    function errName(bytes memory d) internal pure returns (string memory) {
        bytes4 s = bytes4(d);
        if (s == ZeroAmount.selector) return "ZeroAmount";
        if (s == ExceedsSupply.selector) return "ExceedsSupply";
        if (s == Dust.selector) return "Dust";
        if (s == ZeroPayout.selector) return "ZeroPayout";
        if (s == Slippage.selector) return "Slippage";
        if (s == NoHolders.selector) return "NoHolders";
        if (s == InvariantViolated.selector) return "InvariantViolated";
        if (s == bytes4(keccak256("Panic(uint256)"))) return "Overflow";
        return "?";
    }

    struct Cols {
        uint256[] k0;
        uint256[] R0;
        uint256[] Q0;
        uint256[] S0;
        uint256[] p;
        uint256[] e;
        uint256[] op;
        uint256[] u;
        uint256[] lim;
        uint256[] ok;
        string[] err;
        uint256[] res;
        uint256[] fc;
        uint256[] fp;
        uint256[] k;
        uint256[] R;
        uint256[] Q;
        uint256[] S;
    }

    function test_boundary_vectors() public {
        string memory j = vm.readFile(string.concat(vm.projectRoot(), "/../vectors/boundary_evm.json"));
        Cols memory c;
        c.k0 = j.readUintArray(".k0");
        c.R0 = j.readUintArray(".R0");
        c.Q0 = j.readUintArray(".Q0");
        c.S0 = j.readUintArray(".S0");
        c.p = j.readUintArray(".p");
        c.e = j.readUintArray(".e");
        c.op = j.readUintArray(".op");
        c.u = j.readUintArray(".u");
        c.lim = j.readUintArray(".lim");
        c.ok = j.readUintArray(".ok");
        c.err = j.readStringArray(".err");
        c.res = j.readUintArray(".res");
        c.fc = j.readUintArray(".fc");
        c.fp = j.readUintArray(".fp");
        c.k = j.readUintArray(".k");
        c.R = j.readUintArray(".R");
        c.Q = j.readUintArray(".Q");
        c.S = j.readUintArray(".S");
        for (uint256 i; i < c.op.length; ++i) runCase(c, i);
        assertGt(c.op.length, 500);
    }

    struct Out {
        bool success;
        bytes data;
        BernieMath.State r;
        uint256[3] got;
    }

    function exec(Cols memory c, uint256 i) internal view returns (Out memory o) {
        BernieMath.State memory s0 = BernieMath.State(c.k0[i], c.R0[i], c.Q0[i], c.S0[i]);
        if (c.op[i] == 1) {
            try h.mint(s0, c.e[i], c.u[i]) returns (BernieMath.State memory rr, uint256 paid, uint256 fc, uint256 fp) {
                // slippage come nel modello: pagato > limite ⇒ Slippage
                if (paid > c.lim[i]) o.data = abi.encodeWithSelector(Slippage.selector);
                else (o.success, o.r, o.got) = (true, rr, [paid, fc, fp]);
            } catch (bytes memory d) {
                o.data = d;
            }
        } else if (c.op[i] == 2) {
            try h.redeem(s0, c.p[i], c.u[i], c.lim[i]) returns (BernieMath.State memory rr, uint256 out, uint256 fc, uint256 fp) {
                (o.success, o.r, o.got) = (true, rr, [out, fc, fp]);
            } catch (bytes memory d) {
                o.data = d;
            }
        } else {
            try h.donate(s0, c.u[i]) returns (BernieMath.State memory rr) {
                (o.success, o.r) = (true, rr);
            } catch (bytes memory d) {
                o.data = d;
            }
        }
    }

    function runCase(Cols memory c, uint256 i) internal {
        Out memory o = exec(c, i);
        string memory where = string.concat("caso ", vm.toString(i));
        if (c.ok[i] == 1) {
            assertTrue(o.success, string.concat("atteso ok: ", where, " ", errName(o.data)));
            if (c.op[i] != 3) {
                assertEq(o.got[0], c.res[i], string.concat("importo: ", where));
                assertEq(o.got[1], c.fc[i], string.concat("fc: ", where));
                assertEq(o.got[2], c.fp[i], string.concat("fp: ", where));
            }
            assertEq(o.r.k, c.k[i], string.concat("k: ", where));
            assertEq(o.r.R, c.R[i], string.concat("R: ", where));
            assertEq(o.r.Q, c.Q[i], string.concat("Q: ", where));
            assertEq(o.r.S, c.S[i], string.concat("S: ", where));
        } else {
            assertFalse(o.success, string.concat("atteso errore ", c.err[i], ": ", where));
            // un solo errore previsto: il primo nell'ordine di §7
            assertEq(errName(o.data), c.err[i], string.concat("errore diverso: ", where));
        }
    }

    /// absorb (fase 8): Q = 0, 1, S − 1, S, S + 1, 2S, grande; ripetuto è idempotente.
    function test_absorb_boundaries() public view {
        uint256[6] memory sizes = [uint256(1), 2, 3, 7, 1e18, type(uint128).max];
        for (uint256 i; i < sizes.length; ++i) {
            uint256 S = sizes[i];
            uint256[7] memory qs = [uint256(0), 1, S - 1, S, S + 1, 2 * S, 2 * S + 1];
            for (uint256 j; j < qs.length; ++j) {
                uint256 k = 1e12;
                BernieMath.State memory s = h.absorb(BernieMath.State(k, S * k, qs[j], S));
                uint256 d = qs[j] / S;
                assertEq(s.k, k + d);
                assertEq(s.R, S * (k + d));
                assertEq(s.Q, qs[j] - d * S);
                assertLt(s.Q, S);
                BernieMath.State memory t = h.absorb(s);
                assertEq(abi.encode(t), abi.encode(s), "absorb ripetuto");
            }
        }
    }
}
