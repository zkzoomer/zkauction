use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        /// Reconstructed hash chain of all bids placed and revealed onchain
        bytes32 accBidsHash;
        /// Reconstructed hash chain of all offers placed and revealed onchain
        bytes32 accOffersHash;
        /// Hashed together information on the tokens involved
        bytes32 tokensHash;
        /// The root of the auction results tree
        bytes32 auctionResultRoot;
    }

    /// A `BidSubmission` represents a bid submission to borrow an amount of money for a specific interest rate
    #[derive(Serialize, Deserialize)]
    struct BidSubmission {
        /// The address of the bidder
        address bidder;
        /// Defines, alongside the `bidder`, a unique identifier for the bid
        uint96 id;
        /// Hash of the offered price as a percentage of the initial loaned amount vs amount returned at maturity. This stores 9 decimal places
        bytes32 bidPriceHash;
        /// The maximum amount of purchase tokens that can be borrowed
        uint256 amount;
        /// The amount of collateral tokens that were locked onchain
        uint256 collateralAmount;
        /// The address of the ERC20 purchase token
        address purchaseToken;
        /// The addresses of the collateral ERC20 token in the bid
        address collateralToken;
    }

    /// An `OfferSubmission` represents an offer submission to lend an amount of money for a specific interest rate
    #[derive(Serialize, Deserialize)]
    struct OfferSubmission {
        /// The address of the offeror
        address offeror;
        /// Defines, alongside the `offeror`, a unique identifier for the offer
        uint96 id;
        /// Hash of the offered price as a percentage of the initial loaned amount vs amount returned at maturity. This stores 9 decimal places
        bytes32 offerPriceHash;
        /// The maximum amount of purchase tokens that can be lent
        uint256 amount;
        /// The address of the ERC20 purchase token
        address purchaseToken;
    }


    /// A `BidReveal` represents the bid reveal process that was carried out onchain
    #[derive(Serialize, Deserialize)]
    struct BidReveal {
        /// The ID of the bid that was revealed
        uint256 orderId;
        /// The price of the bid that was revealed
        uint256 price;
        /// Nonce value that was used to generate the bid price hash
        uint256 nonce;
    }

    /// An `OfferReveal` represents the offer reveal process that was carried out onchain
    #[derive(Serialize, Deserialize)]
    struct OfferReveal {
        /// The ID of the offer that was revealed
        uint256 orderId;
        /// The price of the offer that was revealed
        uint256 price;
        /// Nonce value that was used to generate the offer price hash
        uint256 nonce;
    }

    /// A `TokenInformation` represents a given ERC-20 token address and its oracle price at proof verification time
    #[derive(Serialize, Deserialize)]
    struct TokenInformation {
        /// The address of the ERC-20 token
        address tokenAddress;
        /// The oracle price of the token at proof verification time
        uint256 price;
    }
}
