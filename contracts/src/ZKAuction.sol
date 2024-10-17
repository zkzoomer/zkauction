// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import {IZKAuction} from "./IZKAuction.sol";

/// @title ZKAuction.
/// @author zkzoomer
/// @notice This contract implements a simple example of verifying an auction proof.
contract ZKAuction is IZKAuction{
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;

    /// @notice The accumulated bids hash
    bytes32 public accBidsHash = bytes32(0x0000000000000000000000000000000000000000000000000000000000000001);
    /// @notice Mapping of bid IDs to the amount of collateral locked, indexed by `abi.encodePacked(bidder, orderId)`
    mapping(bytes32 => uint256) lockedBids;
    /// @notice The accumulated offers hash
    bytes32 public accOffersHash = bytes32(0x0000000000000000000000000000000000000000000000000000000000000001);
    /// @notice Mapping of offer IDs to the amount of purchase tokens locked, indexed by `abi.encodePacked(offeror, orderId)`
    mapping(bytes32 => uint256) lockedOffers;

    /// @notice The purchase token address
    address public purchaseToken;
    /// @notice The collateral token address
    address public collateralToken;
    /// @notice The number of days between auction and maturity dates
    uint256 public dayCount;

    /// @notice The root of the auction results tree
    bytes32 public auctionResultRoot;

    /// @notice The verification key for the zkAuction program.
    bytes32 public zkAuctionProgramVKey;

    constructor(address _purchaseToken, address _collateralToken, uint256 _dayCount, address _verifier, bytes32 _zkAuctionProgramVKey) {
        verifier = _verifier;
        zkAuctionProgramVKey = _zkAuctionProgramVKey;
        purchaseToken = _purchaseToken;
        collateralToken = _collateralToken;
        dayCount = _dayCount;
    }

    /// @notice Submits a bid to borrow an amount of money for a specific interest rate by locking the collateral amount.
    /// @param _id The ID of the bid.
    /// @param _bidPriceHash The hash of the bid price.
    /// @param _amount The amount of purchase tokens to borrow.
    /// @param _collateralAmount The amount of collateral to lock.
    function lockBid(uint96 _id, bytes32 _bidPriceHash, uint256 _amount, uint256 _collateralAmount) public {
        // Lock the collateral amount
        uint256 lockedAmount = lockedBids[bytes32(abi.encodePacked(msg.sender, _id))];
        uint256 newLockedAmount = lockedAmount + _collateralAmount;
        lockedBids[bytes32(abi.encodePacked(msg.sender, _id))] = newLockedAmount;
        // Update the hash chain
        accBidsHash = keccak256(
            abi.encodePacked(
                accBidsHash,
                msg.sender,
                _id,
                _bidPriceHash,
                _amount,
                newLockedAmount
            )
        );
    }

    /// @notice Unlocks collateral from a bid and updates the hash chain.
    /// @param _id The ID of the bid.
    /// @param _bidPriceHash The hash of the bid price.
    /// @param _amount The amount of purchase tokens to borrow.
    /// @param _unlockCollateralAmount The amount of collateral to unlock.
    function unlockBid(uint96 _id, bytes32 _bidPriceHash, uint256 _amount, uint256 _unlockCollateralAmount) public {
        // Unlock the collateral amount
        uint256 lockedAmount = lockedBids[bytes32(abi.encodePacked(msg.sender, _id))];
        if (_unlockCollateralAmount > lockedAmount) revert ZKAuction__InsufficientCollateral();
        uint256 newLockedAmount = lockedAmount - _unlockCollateralAmount;
        lockedBids[bytes32(abi.encodePacked(msg.sender, _id))] = newLockedAmount;
        // Update the hash chain
        accBidsHash = keccak256(
            abi.encodePacked(
                accBidsHash,
                msg.sender,
                _id,
                _bidPriceHash,
                _amount,
                newLockedAmount
            )
        );
    }

    /// @notice Reveals a bid by updating the hash chain.
    /// @param _id The ID of the bid.
    /// @param _price The price that was specified for the bid.
    /// @param _nonce The nonce that was specified for the bid.
    function revealBid(uint96 _id, uint256 _price, uint256 _nonce) public {
        // Update the hash chain
        accBidsHash = keccak256(
            abi.encodePacked(
                accBidsHash,
                msg.sender,
                _id,
                _price,
                _nonce
            )
        );
    }

    /// @notice Submits an offer to lend an amount of money for a specific interest rate by locking the purchase amount.
    /// @param _id The ID of the offer.
    /// @param _offerPriceHash The hash of the offer price.
    /// @param _amount The amount of purchase tokens to sell.
    function lockOffer(uint96 _id, bytes32 _offerPriceHash, uint256 _amount) public {
        // Lock the purchase amount
        uint256 lockedAmount = lockedOffers[bytes32(abi.encodePacked(msg.sender, _id))];
        uint256 newLockedAmount = lockedAmount + _amount;
        lockedOffers[bytes32(abi.encodePacked(msg.sender, _id))] = newLockedAmount;
        // Update the hash chain
        accOffersHash = keccak256(
            abi.encodePacked(
                accBidsHash,
                msg.sender,
                _id,
                _offerPriceHash,
                newLockedAmount
            )
        );
    }

    /// @notice Unlocks purchase tokens from an offer and updates the hash chain.
    /// @param _id The ID of the offer.
    /// @param _offerPriceHash The hash of the offer price.
    /// @param _unlockPurchaseAmount The amount of purchase tokens to unlock.
    function unlockOffer(uint96 _id, bytes32 _offerPriceHash, uint256 _unlockPurchaseAmount) public {
        // Unlock the purchase amount
        uint256 lockedAmount = lockedOffers[bytes32(abi.encodePacked(msg.sender, _id))];
        if (_unlockPurchaseAmount > lockedAmount) revert ZKAuction__InsufficientPurchaseTokens();
        uint256 newLockedAmount = lockedAmount - _unlockPurchaseAmount;
        lockedOffers[bytes32(abi.encodePacked(msg.sender, _id))] = newLockedAmount;
        // Update the hash chain
        accOffersHash = keccak256(
            abi.encodePacked(
                accOffersHash,
                msg.sender,
                _id,
                _offerPriceHash,
                newLockedAmount
            )
        );
    }
    
    /// @notice Reveals an offer by updating the hash chain.
    /// @param _id The ID of the offer.
    /// @param _price The price that was specified for the offer.
    /// @param _nonce The nonce that was specified for the offer.
    function revealOffer(uint96 _id, uint256 _price, uint256 _nonce) public {
        // Update the hash chain
        accOffersHash = keccak256(
            abi.encodePacked(
                accOffersHash,
                msg.sender,
                _id,
                _price,
                _nonce
            )
        );
    }

    /// @notice The entrypoint for verifying the proof for an auction.
    /// @param _proofBytes The encoded proof.
    function verifyAuctionProof(bytes calldata _proofBytes)
        public
        view
    {
        PublicValuesStruct memory publicValues = PublicValuesStruct(
            msg.sender,
            accBidsHash,
            accOffersHash,
            _getAuctionParametersHash(),
            auctionResultRoot
        );

        ISP1Verifier(verifier).verifyProof(zkAuctionProgramVKey, abi.encode(publicValues), _proofBytes);
    }

    function _getAuctionParametersHash() private view returns (bytes32) {
        return keccak256(abi.encode(AuctionParameters(
            purchaseToken,
            _getPurchaseTokenPrice(),
            collateralToken,
            _getCollateralTokenPrice(),
            dayCount
        )));
    }

    function _getPurchaseTokenPrice() private pure returns (uint256) {
        // This should fetch the price from an oracle
        return 99996000;
    }

    function _getCollateralTokenPrice() private pure returns (uint256) {
        // This should fetch the price from an oracle
        return 99996000;
    }
}
