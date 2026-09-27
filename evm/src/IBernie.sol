// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// Interfaccia normativa di §11 (Robinhood Chain). tools/abi_check.py la compila con solc
/// e confronta selettori e topic con l'Appendice B del README.

interface IBernieFactory {
    event Created(address indexed token, address indexed creator);

    function count() external view returns (uint256);
    function tokens(uint256 i) external view returns (address);
    function create(
        uint256 price,
        uint16 penaltyBps,
        uint16 entryBps,
        string calldata name,
        string calldata symbol,
        bytes32 salt
    ) external returns (address);
}

interface IBernie {
    event Minted(address indexed who, uint256 u, uint256 paid);
    event Redeemed(address indexed who, uint256 u, uint256 out);
    event Donated(address indexed who, uint256 amount);
    event State(uint256 k, uint256 S, uint256 R, uint256 Q);

    // Letture ERC-20
    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
    function totalSupply() external view returns (uint256);
    function balanceOf(address who) external view returns (uint256);

    // Letture Bernie
    function k() external view returns (uint256);
    function k0() external view returns (uint256);
    function reserve() external view returns (uint256);
    function residual() external view returns (uint256);
    function penaltyBps() external view returns (uint16);
    function entryBps() external view returns (uint16);
    function creator() external view returns (address);
    function feesOwed(address who) external view returns (uint256);
    function totalFeesOwed() external view returns (uint256);

    // Scritture
    function mint(uint256 u) external payable;
    function redeem(uint256 u, uint256 minOut) external;
    function donate() external payable;
    function claimFees() external;
    function sweep() external;
}
