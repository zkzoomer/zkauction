// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {ZKAuction} from "../src/ZKAuction.sol";
import {SP1VerifierGateway} from "@sp1-contracts/SP1VerifierGateway.sol";

struct SP1ProofFixtureJson {
    bytes proof;
    bytes publicValues;
    bytes32 vkey;
}

contract ZKAuctionTest is Test {
    using stdJson for string;

    address verifier;
    address purchaseToken = address(0x350);
    address collateralToken = address(0x250);
    uint256 dayCount = 100;
    ZKAuction public zkAuction;

    /* function loadFixture() public view returns (SP1ProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/src/fixtures/plonk-fixture.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (SP1ProofFixtureJson));
    } */

    function setUp() public {
        //SP1ProofFixtureJson memory fixture = loadFixture();
        verifier = address(new SP1VerifierGateway(address(1)));
        zkAuction = new ZKAuction(purchaseToken, collateralToken, dayCount, verifier, bytes32(0));
    }

    /* function test_ValidAuctionProof() public {
        SP1ProofFixtureJson memory fixture = loadFixture();

        vm.mockCall(verifier, abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector), abi.encode(true));

        zkAuction.verifyAuctionProof(fixture.proof);
    }

    function testFail_InvalidAuctionProof() public view {
        SP1ProofFixtureJson memory fixture = loadFixture();

        // Create a fake proof.
        bytes memory fakeProof = new bytes(fixture.proof.length);

        zkAuction.verifyAuctionProof(fakeProof);
    } */

    function test_LockBid() public {
        zkAuction.lockBid(1, bytes32(0), 100, 100);
        zkAuction.lockBid(1, bytes32(0), 100, 100);
    }

    function test_UnlockBid() public {
        zkAuction.lockBid(1, bytes32(0), 100, 100);
        zkAuction.unlockBid(1, bytes32(0), 100, 100);
    }

    function test_RevealBid() public {
        zkAuction.lockBid(1, bytes32(0), 100, 100);
        zkAuction.revealBid(1, 100, 100);
    }

    function test_LockOffer() public {
        zkAuction.lockOffer(1, bytes32(0), 100);
        zkAuction.lockOffer(1, bytes32(0), 100);
    }

    function test_UnlockOffer() public {
        zkAuction.lockOffer(1, bytes32(0), 100);
        zkAuction.unlockOffer(1, bytes32(0), 100);
    }

    function test_RevealOffer() public {
        zkAuction.lockOffer(1, bytes32(0), 100);
        zkAuction.revealOffer(1, 100, 100);
    }
}
