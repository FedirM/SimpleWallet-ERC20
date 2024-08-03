// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract SimpleWallet is Ownable {
    event Deposited(IERC20 indexed token, address indexed from, uint256 amount);
    event Withdrawn(IERC20 indexed token, address indexed to, uint256 amount);

    receive() external payable {
        emit Deposited(IERC20(address(0)), msg.sender, msg.value);
    }

    fallback() external payable {
        emit Deposited(IERC20(address(0)), msg.sender, msg.value);
    }

    constructor(address _initialOwner) Ownable(_initialOwner) {}

    function collectETH(address[] memory from, address to) public {
        for (uint256 i = 0; i < from.length; i++) {
            payable(to).transfer(from[i].balance);
        }
    }

    function collectERC20(
        IERC20 token,
        address[] memory from,
        address to
    ) external {
        for (uint256 i = 0; i < from.length; i++) {
            uint256 balance = token.balanceOf(from[i]);
            require(
                token.transferFrom(from[i], to, balance),
                "Transfer failed"
            );
        }
    }

    function withdrawETH(
        address payable[] memory to,
        uint256[] memory amounts
    ) public onlyOwner {
        require(
            to.length == amounts.length,
            "Arrays must have the same length"
        );

        for (uint256 i = 0; i < to.length; i++) {
            (bool success, ) = to[i].call{value: amounts[i]}("");
            require(success, "Transfer failed");
            emit Withdrawn(IERC20(address(0)), to[i], amounts[i]);
        }
    }

    function withdrawERC20(
        IERC20 token,
        address payable[] memory to,
        uint256[] memory amounts
    ) public onlyOwner {
        require(
            to.length == amounts.length,
            "Arrays must have the same length"
        );

        for (uint256 i = 0; i < to.length; i++) {
            token.transfer(to[i], amounts[i]);
            emit Withdrawn(token, to[i], amounts[i]);
        }
    }
}
