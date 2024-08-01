// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {SimpleWallet} from "../src/SimpleWallet.sol";

contract SimpleWalletScript is Script {
    SimpleWallet public wallet;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        wallet = new SimpleWallet(address(this));

        vm.stopBroadcast();
    }
}
