// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;


contract IZKAuction {
    error ZKAuction__InsufficientCollateral();
    error ZKAuction__InsufficientPurchaseTokens();

    /// @dev The `AuctionParameters` struct is used to store the parameters of an auction.
    struct AuctionParameters {
        /// The purchase token address
        address purchaseToken;
        /// The oracle price of the purchase token at proof verification time
        uint256 purchasePrice;
        /// The collateral token address
        address collateralToken;
        /// The oracle price of the collateral token at proof verification time
        uint256 collateralPrice;
        // Number of days between auction and maturity dates, used to compute servicing fees and repurchase prices
        uint256 dayCount;
    }

    /// @dev The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        /// Address of the prover
        address proverAddress;
        /// Reconstructed hash chain of all bids placed and revealed onchain
        bytes32 accBidsHash;
        /// Reconstructed hash chain of all offers placed and revealed onchain
        bytes32 accOffersHash;
        /// Hashed together information on the tokens involved
        bytes32 auctionParametersHash;
        /// The root of the auction results tree
        bytes32 auctionResultRoot;
    }
}
