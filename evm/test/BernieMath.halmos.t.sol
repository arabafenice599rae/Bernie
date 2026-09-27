// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import "../src/BernieMath.sol";

/// Harness Halmos su BernieMath (§17): gli stessi di Kani su state.rs, con SCALE = 10 e
/// input fino a 8 bit (fino a 32 per le fee). È bounded model checking su quel dominio;
/// la corrispondenza con SCALE = 10¹⁸ è coperta dai vettori differenziali.
///
/// Halmos non considera fallimento un revert con custom error: le operazioni sono quindi
/// chiamate con this.* dentro try/catch, e l'unico revert ammesso è un errore di dominio,
/// mai InvariantViolated.
contract BernieMathHalmos {
    uint256 constant SC = 10;

    function _canonical(uint8 k8, uint8 s8, uint8 q8) internal pure returns (BernieMath.State memory s) {
        uint256 k = k8;
        uint256 S = s8 & 0x1f;
        uint256 Q = q8 & 0x1f;
        require(k >= 1);
        require(S == 0 ? Q == 0 : Q < S);
        require((k * S + Q) % SC == 0);
        s = BernieMath.State(k, k * S, Q, S);
    }

    function _isCanonical(BernieMath.State memory s) internal pure returns (bool) {
        return s.S == 0 ? (s.Q == 0 && s.R == 0) : s.Q < s.S;
    }

    function _notInvariant(bytes memory reason) internal pure {
        assert(bytes4(reason) != InvariantViolated.selector);
    }

    // Punti d'ingresso esterni, per poter osservare il motivo dei revert.
    function doMint(BernieMath.State memory s, uint256 e, uint256 u)
        external
        pure
        returns (BernieMath.State memory, uint256, BernieMath.Fees memory)
    {
        return BernieMath.mintAt(s, e, u, SC);
    }

    function doRedeem(BernieMath.State memory s, uint256 p, uint256 u)
        external
        pure
        returns (BernieMath.State memory, uint256, uint256, BernieMath.Fees memory)
    {
        return BernieMath.redeemAt(s, p, u, 0, SC);
    }

    function doDonate(BernieMath.State memory s, uint256 a) external pure returns (BernieMath.State memory) {
        return BernieMath.donateAt(s, a, SC);
    }

    function check_absorb(uint8 k8, uint8 s8, uint8 q8, uint16 extra) public pure {
        BernieMath.State memory s = _canonical(k8, s8, q8);
        s.Q += extra;
        uint256 before = s.R + s.Q;
        BernieMath.absorb(s);
        assert(s.R + s.Q == before);
        assert(s.R == s.S * s.k);
        assert(s.S == 0 || s.Q < s.S);
    }

    /// Penalità fisse (riferimento 2% / 1%): con p ed e simbolici i prodotti a 256 bit non
    /// chiudono in 40 minuti. Tutte le combinazioni di p ed e sono coperte dai vettori e da Kani.
    uint256 constant P_REF = 200;
    uint256 constant E_REF = 100;

    function check_mint(uint8 k8, uint8 s8, uint8 q8, uint8 u8) public view {
        BernieMath.State memory s0 = _canonical(k8, s8, q8);
        uint256 e = E_REF;
        uint256 u = u8 & 0x1f;
        require(u >= 1);
        try this.doMint(s0, e, u) returns (BernieMath.State memory s, uint256 c, BernieMath.Fees memory f) {
            assert(_isCanonical(s));
            assert(s.k >= s0.k);
            assert(s.R + s.Q == s0.R + s0.Q + c * SC); // entra solo c·SCALE, le fee restano fuori
            uint256 full = u * s.k;
            assert(BernieMath.cdiv(full, SC) <= c); // b ≤ c
            assert(f.creator + f.protocol == f.total);
        } catch (bytes memory reason) {
            _notInvariant(reason);
        }
    }

    function check_redeem(uint8 k8, uint8 s8, uint8 q8, uint8 u8) public view {
        BernieMath.State memory s0 = _canonical(k8, s8, q8);
        uint256 p = P_REF;
        uint256 u = u8 & 0x1f;
        require(u >= 1);
        try this.doRedeem(s0, p, u) returns (BernieMath.State memory s, uint256 g, uint256 out, BernieMath.Fees memory) {
            assert(_isCanonical(s));
            assert(s.k >= s0.k);
            assert(out <= g);
            uint256 full = u * s0.k;
            assert(g * SC <= full - BernieMath.cdiv(full * p, BernieMath.BPS));
            if (s.S > 0) assert(s.R + s.Q == s0.R + s0.Q - g * SC);
        } catch (bytes memory reason) {
            _notInvariant(reason);
        }
    }

    function check_donate(uint8 k8, uint8 s8, uint8 q8, uint8 a) public view {
        BernieMath.State memory s0 = _canonical(k8, s8, q8);
        try this.doDonate(s0, a) returns (BernieMath.State memory s) {
            assert(_isCanonical(s));
            assert(s.k >= s0.k);
            assert(s.R + s.Q == s0.R + s0.Q + uint256(a) * SC);
        } catch (bytes memory reason) {
            _notInvariant(reason);
        }
    }

    function check_fees(uint32 base32) public pure {
        uint256 base = base32;
        BernieMath.Fees memory a = BernieMath.fees(base);
        BernieMath.Fees memory b = BernieMath.fees(base + 1);
        assert(b.total - a.total <= 1);
        assert(a.creator + a.protocol == a.total);
        assert(base + 1 - b.total >= base - a.total); // P7
    }
}
