// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.30;

/// Errori di §7 (i codici numerici sono quelli del programma Solana).
error ZeroAmount(); //         1
error ExceedsSupply(); //      2
error Dust(); //               3
error ZeroPayout(); //         4
error Slippage(); //           5
error NoHolders(); //          6
error PenaltyOutOfRange(); //  8
error PriceOutOfRange(); //    9
error InvariantViolated(); // 11
error NothingToClaim(); //    12
error TransferToSelf(); //    13
error MetadataTooLong(); //   16

/// Aritmetica di Bernie v1.6 (§5, §6) con SCALE = 10¹⁸: funzioni pure, l'unico codice
/// che Halmos deve dimostrare (§13). Stesse formule, stesso ordine dei controlli e stessi
/// arrotondamenti del modello di riferimento (Appendice A); i vettori lo verificano.
library BernieMath {
    uint256 internal constant SCALE = 1e18;
    uint256 internal constant BPS = 10_000;
    uint256 internal constant FEE_C = 20;
    uint256 internal constant FEE_P = 20;
    uint256 internal constant PEN_MIN = 100;
    uint256 internal constant PEN_MAX = 1_000;
    uint256 internal constant MIN_PRICE = 1e12;
    uint256 internal constant MAX_PRICE = 1e24;

    /// k, R, Q in sotto-wei; S in unità base.
    struct State {
        uint256 k;
        uint256 R;
        uint256 Q;
        uint256 S;
    }

    struct Fees {
        uint256 total;
        uint256 creator;
        uint256 protocol;
    }

    function cdiv(uint256 a, uint256 b) internal pure returns (uint256) {
        return a == 0 ? 0 : (a - 1) / b + 1;
    }

    /// fees(base): un solo arrotondamento sulla fee totale.
    function fees(uint256 base) internal pure returns (Fees memory f) {
        f.total = base * (FEE_C + FEE_P) / BPS;
        f.protocol = f.total * FEE_P / (FEE_C + FEE_P);
        f.creator = f.total - f.protocol;
    }

    function absorb(State memory s) internal pure {
        if (s.S == 0) return;
        uint256 d = s.Q / s.S;
        s.k += d;
        s.R += d * s.S;
        s.Q -= d * s.S;
    }

    /// I1, I3, I4, I5, I6. I2 dipende dal saldo ed è verificata dal token.
    function check(State memory s, uint256 kPrev) internal pure {
        checkAt(s, kPrev, SCALE);
    }

    function checkAt(State memory s, uint256 kPrev, uint256 scale) internal pure {
        if (
            s.R != s.S * s.k || s.k < kPrev || (s.S == 0 && s.Q != 0) || (s.R + s.Q) % scale != 0
                || (s.S != 0 && s.Q >= s.S)
        ) revert InvariantViolated();
    }

    /// mint(u): ritorna il nuovo stato, c (backing + epen, in wei) e le fee su ⌈full/SCALE⌉.
    function mint(State memory s0, uint256 e, uint256 u)
        internal
        pure
        returns (State memory s, uint256 c, Fees memory f)
    {
        return mintAt(s0, e, u, SCALE);
    }

    /// Come mint, con SCALE come parametro: gli harness Halmos usano SCALE = 10 come Kani.
    function mintAt(State memory s0, uint256 e, uint256 u, uint256 scale)
        internal
        pure
        returns (State memory s, uint256 c, Fees memory f)
    {
        if (u == 0) revert ZeroAmount();
        s = State(s0.k, s0.R, s0.Q, s0.S);
        uint256 epen = s.S == 0 ? 0 : cdiv(u * s.k * e, BPS);
        s.Q += epen;
        absorb(s); // solo gli holder esistenti
        uint256 full = u * s.k;
        c = cdiv(full + epen, scale);
        f = fees(cdiv(full, scale));
        s.Q += c * scale - full - epen;
        s.S += u;
        s.R += full;
        absorb(s);
        checkAt(s, s0.k, scale);
    }

    /// redeem(u, minOut): ritorna il nuovo stato, il lordo g, l'uscita out e le fee su g.
    function redeem(State memory s0, uint256 p, uint256 u, uint256 minOut)
        internal
        pure
        returns (State memory s, uint256 g, uint256 out, Fees memory f)
    {
        return redeemAt(s0, p, u, minOut, SCALE);
    }

    function redeemAt(State memory s0, uint256 p, uint256 u, uint256 minOut, uint256 scale)
        internal
        pure
        returns (State memory s, uint256 g, uint256 out, Fees memory f)
    {
        if (u == 0) revert ZeroAmount();
        if (u > s0.S) revert ExceedsSupply();
        s = State(s0.k, s0.R, s0.Q, s0.S);
        uint256 full = u * s.k;
        uint256 pen = cdiv(full * p, BPS);
        if (full <= pen) revert Dust();
        g = (full - pen) / scale;
        f = fees(g);
        out = g - f.total;
        if (out == 0) revert ZeroPayout();
        if (out < minOut) revert Slippage();
        s.S -= u;
        s.R -= full;
        if (s.S == 0) {
            s.Q = 0; // penalità e resti diventano excess
        } else {
            s.Q += full - g * scale;
            absorb(s);
        }
        checkAt(s, s0.k, scale);
    }

    function donate(State memory s0, uint256 a) internal pure returns (State memory s) {
        return donateAt(s0, a, SCALE);
    }

    function donateAt(State memory s0, uint256 a, uint256 scale) internal pure returns (State memory s) {
        if (a == 0) revert ZeroAmount();
        if (s0.S == 0) revert NoHolders();
        s = State(s0.k, s0.R, s0.Q + a * scale, s0.S);
        absorb(s);
        checkAt(s, s0.k, scale);
    }

    /// Parametri di create (§6, §20).
    function validate(uint256 price, uint256 p, uint256 e, bytes memory name, bytes memory symbol) internal pure {
        if (price < MIN_PRICE || price > MAX_PRICE) revert PriceOutOfRange();
        if (p < PEN_MIN || p > PEN_MAX || e > p) revert PenaltyOutOfRange();
        if (name.length == 0 || name.length > 32 || symbol.length == 0 || symbol.length > 10) {
            revert MetadataTooLong();
        }
    }
}
