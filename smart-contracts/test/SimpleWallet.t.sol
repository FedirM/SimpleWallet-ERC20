// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
// import "../src/SimpleWallet.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

// Mock ERC20 Token
contract TestToken is ERC20 {
    constructor(uint256 initialSupply) ERC20("TestToken", "TTK") {
        _mint(msg.sender, initialSupply);
    }
}

// Combined Tests for ETH and ERC20
// contract SimpleWalletTest is Test {
//     SimpleWallet public wallet;
//     TestToken public token;
//     address payable public owner;
//     address payable public user1;
//     address payable public user2;
//     address payable public user3;

//     function setUp() public {
//         owner = payable(address(0x123));
//         user1 = payable(address(0x456));
//         user2 = payable(address(0x789));
//         user3 = payable(address(0xABC));

//         token = new TestToken(1000 ether);
//         wallet = new SimpleWallet(owner);

//         // Distribute tokens to users
//         token.transfer(user1, 100 ether);
//         token.transfer(user2, 100 ether);

//         // Approve wallet contract to spend tokens
//         vm.prank(user1);
//         token.approve(address(wallet), 100 ether);

//         vm.prank(user2);
//         token.approve(address(wallet), 100 ether);
//     }

//     // ETH Tests
//     function test_DepositETH() public payable {
//         uint256 initialBalance = address(wallet).balance;
//         uint256 depositAmount = 10 ether;

//         vm.deal(user1, depositAmount);
//         vm.prank(user1);
//         (bool success, ) = address(wallet).call{value: depositAmount}("");
//         require(success, "ETH deposit failed");

//         assertEq(address(wallet).balance, initialBalance + depositAmount);
//     }

//     function test_WithdrawAmountsETH() public {
//         address payable[] memory to = new address payable[](2);
//         to[0] = user2;
//         to[1] = user3;

//         uint256[] memory withdrawAmounts = new uint256[](2);
//         withdrawAmounts[0] = 10 ether;
//         withdrawAmounts[1] = 40 ether;

//         uint256 totalWithdraw = 50 ether;
//         uint256 walletBalance = 1000 ether;
//         // Deposit tokens into the wallet
//         vm.deal(address(wallet), walletBalance);

//         // Expected log emissions
//         vm.expectEmit(true, true, true, true);
//         emit SimpleWallet.Withdrawn(address(0), user2, 10 ether);
//         vm.expectEmit(true, true, true, true);
//         emit SimpleWallet.Withdrawn(address(0), user3, 40 ether);

//         vm.prank(owner);
//         // Perform withdrawal
//         wallet.withdrawAmounts(address(0), to, withdrawAmounts);

//         // Verify final balances
//         assertEq(address(wallet).balance, walletBalance - totalWithdraw);
//         assertEq(user2.balance, withdrawAmounts[0]);
//         assertEq(user3.balance, withdrawAmounts[1]);
//     }

//     // ERC20 Token Tests
//     function test_DepositTokens() public {
//         address[] memory from = new address[](2);
//         from[0] = user1;
//         from[1] = user2;

//         uint256[] memory amounts = new uint256[](2);
//         amounts[0] = 80 ether;
//         amounts[1] = 70 ether;

//         vm.prank(user1);
//         token.approve(address(wallet), 80 ether);
//         vm.prank(user2);
//         token.approve(address(wallet), 70 ether);

//         vm.prank(owner);
//         wallet.deposit(address(token), from, amounts);

//         assertEq(token.balanceOf(address(wallet)), 150 ether);
//         assertEq(token.balanceOf(user1), 20 ether);
//         assertEq(token.balanceOf(user2), 30 ether);
//     }

//     function test_WithdrawAmountsTokens() public {
//         address payable[] memory to = new address payable[](2);
//         to[0] = user2;
//         to[1] = user3;

//         uint256[] memory withdrawAmounts = new uint256[](2);
//         withdrawAmounts[0] = 10 ether;
//         withdrawAmounts[1] = 40 ether;

//         // Approve the wallet to spend tokens
//         vm.prank(user1);
//         token.approve(address(wallet), 10 ether);
//         vm.prank(user2);
//         token.approve(address(wallet), 40 ether);

//         // Deposit tokens into the wallet
//         address[] memory from = new address[](2);
//         from[0] = user1;
//         from[1] = user2;

//         uint256[] memory amounts = new uint256[](2);
//         amounts[0] = 10 ether;
//         amounts[1] = 40 ether;

//         vm.prank(owner);
//         wallet.deposit(address(token), from, amounts);

//         // Expected log emissions
//         vm.prank(owner);
//         vm.expectEmit(true, true, true, true);
//         emit SimpleWallet.Withdrawn(address(token), user2, 10 ether);
//         vm.expectEmit(true, true, true, true);
//         emit SimpleWallet.Withdrawn(address(token), user3, 40 ether);

//         // Perform withdrawal
//         wallet.withdrawAmounts(address(token), to, withdrawAmounts);

//         // Verify final balances
//         assertEq(token.balanceOf(address(wallet)), 0 ether);
//         assertEq(token.balanceOf(user2), 70 ether); // Remaining balance for user2
//         assertEq(token.balanceOf(user3), 40 ether); // Balance for user3
//     }

//     function test_WithdrawPercentagesTokens() public {
//         // Initial setup
//         address payable[] memory to = new address payable[](2);
//         to[0] = user2;
//         to[1] = user3;

//         uint8[] memory percentages = new uint8[](2);
//         percentages[0] = 60; // 60% to user2
//         percentages[1] = 40; // 40% to user3

//         // Approve the wallet to spend tokens
//         vm.prank(user1);
//         token.approve(address(wallet), 100 ether); // Adjust according to actual needs

//         // Deposit tokens into the wallet
//         address[] memory from = new address[](2);
//         from[0] = user1;
//         from[1] = user1;

//         uint256[] memory amounts = new uint256[](2);
//         amounts[0] = 80 ether; // Total tokens deposited
//         amounts[1] = 20 ether;

//         vm.prank(owner);
//         wallet.deposit(address(token), from, amounts);

//         // Calculate expected amounts
//         uint256 totalBalance = token.balanceOf(address(wallet));
//         uint256 expectedAmountUser2 = (totalBalance * percentages[0]) / 100;
//         uint256 expectedAmountUser3 = (totalBalance * percentages[1]) / 100;

//         // Expect correct emissions
//         vm.prank(owner);
//         vm.expectEmit(true, true, true, true);
//         emit SimpleWallet.Withdrawn(address(token), user2, expectedAmountUser2);
//         vm.expectEmit(true, true, true, true);
//         emit SimpleWallet.Withdrawn(address(token), user3, expectedAmountUser3);

//         // Perform withdrawal
//         wallet.withdrawPercentages(address(token), to, percentages);

//         // Verify final balances
//         assertEq(token.balanceOf(address(wallet)), 0 ether);
//         assertEq(token.balanceOf(user2), 100 ether + expectedAmountUser2); // Adjust based on user2's initial balance
//         assertEq(token.balanceOf(user3), expectedAmountUser3); // Balance for user3
//     }
// }
