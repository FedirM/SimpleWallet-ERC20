// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract SimpleWallet is Ownable {
    event Deposited(
        address indexed token,
        address indexed from,
        uint256 amount
    );
    event Withdrawn(address indexed token, address indexed to, uint256 amount);

    receive() external payable {
        emit Deposited(address(0), msg.sender, msg.value);
    }

    fallback() external payable {
        emit Deposited(address(0), msg.sender, msg.value);
    }

    constructor(address _initialOwner) Ownable(_initialOwner) {}

    function deposit(
        address token,
        address[] memory from,
        uint256[] memory amounts
    ) external payable {
        require(
            from.length == amounts.length,
            "Arrays must have the same length"
        );

        for (uint256 i = 0; i < from.length; i++) {
            uint256 amount = amounts[i];
            require(amount > 0, "Amount must be greater than zero");

            uint256 allowance = IERC20(token).allowance(from[i], address(this));
            require(allowance >= amount, "Insufficient allowance");

            IERC20(token).transferFrom(from[i], address(this), amount);
            emit Deposited(token, from[i], amount);
        }
    }

    function withdrawETHAmounts(
        address payable[] memory to,
        uint256[] memory amounts
    ) public payable onlyOwner {
        require(
            to.length == amounts.length,
            "Arrays must have the same length"
        );

        uint256 totalBalance = address(this).balance;
        require(totalBalance > 0, "No ETH available");

        uint256 totalAmount = 0;
        for (uint256 i = 0; i < amounts.length; i++) {
            totalAmount += amounts[i];
        }

        require(totalBalance >= totalAmount, "Not enough ETH for withdraw");

        for (uint256 i = 0; i < to.length; i++) {
            (bool success, ) = to[i].call{value: amounts[i]}("");
            require(success, "Transfer failed");
            emit Withdrawn(address(0), to[i], amounts[i]);
        }
    }

    function withdrawAmounts(
        address token,
        address payable[] memory to,
        uint256[] memory amounts
    ) public payable onlyOwner {
        require(
            to.length == amounts.length,
            "Arrays must have the same length"
        );

        if (token == address(0)) {
            uint256 totalBalance = address(this).balance;
            require(totalBalance > 0, "No ETH available");

            uint256 totalAmount = 0;
            for (uint256 i = 0; i < amounts.length; i++) {
                totalAmount += amounts[i];
            }

            require(totalBalance >= totalAmount, "Not enough ETH for withdraw");

            for (uint256 i = 0; i < to.length; i++) {
                (bool success, ) = to[i].call{value: amounts[i]}("");
                require(success, "Transfer failed");
                emit Withdrawn(address(0), to[i], amounts[i]);
            }
        } else {
            uint256 totalBalance = IERC20(token).balanceOf(address(this));
            require(totalBalance > 0, "No tokens available");

            uint256 totalAmount = 0;
            for (uint256 i = 0; i < amounts.length; i++) {
                totalAmount += amounts[i];
            }

            require(
                totalBalance >= totalAmount,
                "Not enough tokens for withdraw"
            );

            for (uint256 i = 0; i < to.length; i++) {
                IERC20(token).transfer(to[i], amounts[i]);
                emit Withdrawn(token, to[i], amounts[i]);
            }
        }
    }

    function withdrawPercentages(
        address token,
        address payable[] memory to,
        uint8[] memory percentages
    ) public payable onlyOwner {
        require(
            to.length == percentages.length,
            "Arrays must have the same length"
        );

        if (token == address(0)) {
            // ETH Withdrawal by Percentage
            uint256 totalBalance = address(this).balance;
            require(totalBalance > 0, "No ETH available");

            uint8 totalPercentage = 0;
            uint256 len = percentages.length;
            for (uint256 i = 0; i < len; ) {
                totalPercentage += percentages[i];
                unchecked {
                    ++i;
                }
            }

            require(totalPercentage <= 100, "Invalid percentage summary");

            for (uint256 i = 0; i < to.length; ) {
                uint256 amount = (totalBalance * percentages[i]) / 100;
                require(amount > 0, "Invalid percentage");
                payable(to[i]).transfer(amount);
                emit Withdrawn(token, to[i], amount);

                unchecked {
                    ++i;
                }
            }
        } else {
            // ERC20 Token Withdrawal by Percentage
            uint256 totalBalance = IERC20(token).balanceOf(address(this));
            require(totalBalance > 0, "No tokens available");

            uint8 totalPercentage = 0;
            uint256 len = percentages.length;
            for (uint256 i = 0; i < len; ) {
                totalPercentage += percentages[i];
                unchecked {
                    ++i;
                }
            }

            require(totalPercentage <= 100, "Invalid percentage summary");

            for (uint256 i = 0; i < to.length; ) {
                uint256 amount = (totalBalance * percentages[i]) / 100;
                require(amount > 0, "Invalid percentage");
                IERC20(token).transfer(to[i], amount);
                emit Withdrawn(token, to[i], amount);

                unchecked {
                    ++i;
                }
            }
        }
    }
}
