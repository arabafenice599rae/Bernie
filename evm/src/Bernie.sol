// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.30;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Clones} from "@openzeppelin/contracts/proxy/Clones.sol";
import {Address} from "@openzeppelin/contracts/utils/Address.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {BernieMath, NothingToClaim, Slippage, TransferToSelf, InvariantViolated} from "./BernieMath.sol";

/// Token Bernie su Robinhood Chain (§3, §6, §11, §13): clone ERC-20 con argomenti
/// immutabili, nessun initializer, nessun admin, niente receive() né fallback().
///
/// Ordine in ogni funzione: nonReentrant; calcolo puro; storage, mint o burn, fee;
/// asserzione di I1–I6; trasferimenti di ETH per ultimi.
contract Bernie is ERC20, ReentrancyGuardTransient {
    /// Tesoreria del protocollo: costante dell'implementazione, letta anche dai cloni.
    address public immutable treasury;

    uint256 private kGrowth; // k = k0 + kGrowth: parte da 0, quindi niente initializer
    uint256 public reserve;
    uint256 public residual;
    mapping(address => uint256) public feesOwed;
    uint256 public totalFeesOwed;

    event Minted(address indexed who, uint256 u, uint256 paid);
    event Redeemed(address indexed who, uint256 u, uint256 out);
    event Donated(address indexed who, uint256 amount);
    event State(uint256 k, uint256 S, uint256 R, uint256 Q);

    constructor(address treasury_) ERC20("", "") {
        treasury = treasury_;
    }

    // ── argomenti immutabili: creator, penaltyBps, entryBps, k0, name, symbol ──

    function _args()
        internal
        view
        returns (address c, uint16 p, uint16 e, uint256 k0_, string memory n, string memory s)
    {
        return abi.decode(Clones.fetchCloneArgs(address(this)), (address, uint16, uint16, uint256, string, string));
    }

    function creator() public view returns (address c) {
        (c,,,,,) = _args();
    }

    function penaltyBps() public view returns (uint16 p) {
        (, p,,,,) = _args();
    }

    function entryBps() public view returns (uint16 e) {
        (,, e,,,) = _args();
    }

    function k0() public view returns (uint256 v) {
        (,,, v,,) = _args();
    }

    function name() public view override returns (string memory n) {
        (,,,, n,) = _args();
    }

    function symbol() public view override returns (string memory s) {
        (,,,,, s) = _args();
    }

    function k() public view returns (uint256) {
        return k0() + kGrowth;
    }

    // ── operazioni ──

    function mint(uint256 u) external payable nonReentrant {
        (address c, , uint16 e, uint256 kInit,,) = _args();
        BernieMath.State memory s0 = _state(kInit);
        (BernieMath.State memory s, uint256 cost, BernieMath.Fees memory f) = BernieMath.mint(s0, e, u);
        uint256 paid = cost + f.total;
        if (msg.value < paid) revert Slippage();
        _store(s, kInit);
        _accrue(c, f);
        _mint(msg.sender, u);
        uint256 refund = msg.value - paid;
        _checkSolvency(s, refund);
        emit Minted(msg.sender, u, paid);
        emit State(s.k, s.S, s.R, s.Q);
        if (refund > 0) Address.sendValue(payable(msg.sender), refund);
    }

    function redeem(uint256 u, uint256 minOut) external nonReentrant {
        (address c, uint16 p,, uint256 kInit,,) = _args();
        BernieMath.State memory s0 = _state(kInit);
        (BernieMath.State memory s,, uint256 out, BernieMath.Fees memory f) = BernieMath.redeem(s0, p, u, minOut);
        _store(s, kInit);
        _accrue(c, f);
        _burn(msg.sender, u);
        _checkSolvency(s, out);
        emit Redeemed(msg.sender, u, out);
        emit State(s.k, s.S, s.R, s.Q);
        Address.sendValue(payable(msg.sender), out);
    }

    function donate() external payable nonReentrant {
        uint256 kInit = k0();
        BernieMath.State memory s = BernieMath.donate(_state(kInit), msg.value);
        _store(s, kInit);
        _checkSolvency(s, 0);
        emit Donated(msg.sender, msg.value);
        emit State(s.k, s.S, s.R, s.Q);
    }

    function claimFees() external nonReentrant {
        _claim(msg.sender);
    }

    /// Chiamabile da chiunque: paga sempre e solo `account`, mai il chiamante. Permette alla
    /// factory di ritirare le fee di un account su più token in una transazione (claimAll).
    function claimFeesFor(address account) external nonReentrant {
        _claim(account);
    }

    /// Chiamabile da chiunque: l'excess va sempre al creator registrato.
    function sweep() external nonReentrant {
        uint256 excess = address(this).balance - (reserve + residual) / BernieMath.SCALE - totalFeesOwed;
        if (excess == 0) revert NothingToClaim();
        Address.sendValue(payable(creator()), excess);
    }

    // ── interni ──

    function _claim(address account) internal {
        uint256 amt = feesOwed[account];
        if (amt == 0) revert NothingToClaim();
        feesOwed[account] = 0;
        totalFeesOwed -= amt;
        Address.sendValue(payable(account), amt);
    }

    function _state(uint256 kInit) internal view returns (BernieMath.State memory) {
        return BernieMath.State(kInit + kGrowth, reserve, residual, totalSupply());
    }

    function _store(BernieMath.State memory s, uint256 kInit) internal {
        kGrowth = s.k - kInit;
        reserve = s.R;
        residual = s.Q;
    }

    function _accrue(address c, BernieMath.Fees memory f) internal {
        feesOwed[c] += f.creator;
        feesOwed[treasury] += f.protocol;
        totalFeesOwed += f.total;
    }

    /// I1 con la supply reale dell'ERC-20 e I2 dopo i trasferimenti ancora da fare.
    function _checkSolvency(BernieMath.State memory s, uint256 pendingOut) internal view {
        if (totalSupply() != s.S) revert InvariantViolated();
        uint256 bal = address(this).balance - pendingOut;
        if (bal * BernieMath.SCALE < s.R + s.Q + totalFeesOwed * BernieMath.SCALE) revert InvariantViolated();
    }

    /// Nessun trasferimento verso il contratto stesso (§7, errore 13).
    function _update(address from, address to, uint256 value) internal override {
        if (to == address(this)) revert TransferToSelf();
        super._update(from, to, value);
    }
}
