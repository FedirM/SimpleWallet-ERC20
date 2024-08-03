// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/SimpleWallet.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// A simple ERC20 token for testing
contract TestERC20 is ERC20 {
    constructor() ERC20("TestToken", "TTK") {
        _mint(msg.sender, 10000 * 10 ** 18);
    }
}

contract SimpleWalletTest is Test {
    SimpleWallet public wallet;
    TestERC20 public token;
    address public owner;
    address public user1;
    address public user2;
    address public user3;

    function setUp() public {
        owner = address(this);
        user1 = address(1);
        user2 = address(2);
        user3 = address(3);

        wallet = new SimpleWallet(owner);
        token = new TestERC20();

        token.transfer(user1, 1000 * 10 ** 18);
        token.transfer(user2, 1000 * 10 ** 18);

        vm.startPrank(user1);
        token.approve(address(wallet), 1000 * 10 ** 18);
        deal(user1, 1 ether);
        vm.stopPrank();

        vm.startPrank(user2);
        token.approve(address(wallet), 1000 * 10 ** 18);
        deal(user2, 1 ether);
        vm.stopPrank();
    }

    function test_CollectERC20() public {
        address[] memory from = new address[](2);
        from[0] = user1;
        from[1] = user2;

        wallet.collectERC20(IERC20(address(token)), from, address(wallet));

        assertEq(token.balanceOf(address(wallet)), 2000 * 10 ** 18);
    }

    function test_WithdrawETH() public {
        address payable[] memory to = new address payable[](2);
        to[0] = payable(user1);
        to[1] = payable(user2);

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = 0.5 ether;
        amounts[1] = 0.5 ether;

        vm.deal(address(wallet), 1 ether);

        vm.prank(owner);
        wallet.withdrawETH(to, amounts);

        assertEq(user1.balance, 1.5 ether);
        assertEq(user2.balance, 1.5 ether);
    }

    function test_WithdrawERC20() public {
        address payable[] memory to = new address payable[](2);
        to[0] = payable(user1);
        to[1] = payable(user2);

        address[] memory from = new address[](2);
        from[0] = user1;
        from[1] = user2;

        uint256[] memory amounts = new uint256[](2);
        amounts[0] = 500 * 10 ** 18;
        amounts[1] = 500 * 10 ** 18;

        wallet.collectERC20(IERC20(address(token)), from, address(wallet));

        assertEq(token.balanceOf(address(wallet)), 2000 * 10 ** 18);

        vm.prank(owner);
        wallet.withdrawERC20(IERC20(address(token)), to, amounts);

        assertEq(token.balanceOf(user1), amounts[0]);
        assertEq(token.balanceOf(user2), amounts[1]);
    }
}
