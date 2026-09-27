// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.30;

import {Clones} from "@openzeppelin/contracts/proxy/Clones.sol";
import {Bernie} from "./Bernie.sol";
import {BernieMath} from "./BernieMath.sol";

/// Factory senza owner (§11, §13): cloni deterministici con salt keccak256(msg.sender, salt),
/// enumerazione con count() e tokens(i).
contract BernieFactory {
    address public immutable implementation;
    address[] public tokens;

    event Created(address indexed token, address indexed creator);

    constructor(address treasury) {
        implementation = address(new Bernie(treasury));
    }

    function count() external view returns (uint256) {
        return tokens.length;
    }

    function create(
        uint256 price,
        uint16 penaltyBps,
        uint16 entryBps,
        string calldata name,
        string calldata symbol,
        bytes32 salt
    ) external returns (address token) {
        BernieMath.validate(price, penaltyBps, entryBps, bytes(name), bytes(symbol));
        bytes memory args = abi.encode(msg.sender, penaltyBps, entryBps, price, name, symbol);
        token = Clones.cloneDeterministicWithImmutableArgs(implementation, args, keccak256(abi.encode(msg.sender, salt)));
        tokens.push(token);
        emit Created(token, msg.sender);
    }
}
