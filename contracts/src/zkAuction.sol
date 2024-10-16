// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

/// The public values encoded as a struct that can be easily deserialized inside Solidity.
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

/// @title zkAuction.
/// @author zkzoomer
/// @notice This contract implements a simple example of verifying an auction proof.
contract zkAuctionVerifier {
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;

    /// @notice The verification key for the zkAuction program.
    bytes32 public zkAuctionProgramVKey;

    constructor(address _verifier, bytes32 _zkAuctionProgramVKey) {
        verifier = _verifier;
        zkAuctionProgramVKey = _zkAuctionProgramVKey;
    }

    /// @notice The entrypoint for verifying the proof for an auction.
    /// @param _publicValues The encoded public values.
    /// @param _proofBytes The encoded proof.
    function verifyAuctionProof(bytes calldata _publicValues, bytes calldata _proofBytes)
        public
        view
    {
        ISP1Verifier(verifier).verifyProof(zkAuctionProgramVKey, _publicValues, _proofBytes);
    }
}
