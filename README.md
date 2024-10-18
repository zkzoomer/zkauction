# zkauction

This project presents a proof of concept to infinitely scale onchain auctions by use of a ZK-powered architecture. You can find the original improvement proposal this work is based on [here](https://hackmd.io/@zkzoomer/B1bk2pWAA).

> [!CAUTION]
> This is an ongoing proof of concept and is **severely infested with critical bugs**. If you are so "*brave*" to use it in production, you will be left feeling **poor and very stupid**.

## Overview

To say that [fixed-income markets](https://www.investopedia.com/fixed-income-essentials-4689775) are the backbone of the modern economy is not an understatement. Fixed-income products enable institutions and individuals to predict future cash flows and make investments according to their risk appetite. However, such products are currently absent in DeFi, effectively preventing the development of more advanced financial products like forwards, futures, and options.

In an ideal world, we should be able to plot Bitcoin's [yield curve](https://www.investopedia.com/terms/y/yieldcurve.asp) by use of decentralized and permissionless smart contracts. But such implementation quickly runs into a scalability problem: executing the logic for an auction is not cheap, and having to do so for *every order that was placed* inevitably grows past the block gas limit.

The motivation behind this work was to implement a **ZK-based, fixed gas-cost model to clear and settle auctions** to i) effectively remove the block gas limit constraint, and ii) increase protocol revenue and lower fees.

### Specification

This proof of concept demonstrates non-custodial, fixed-rate lending using sealed bids and offers to determine a market-clearing interest rate. Successful participants receive loans or bond tokens, while others get their funds returned.

By moving computations offchain, we achieve infinite scalability for auctions with fixed gas costs. Orders are not stored onchain, reducing gas fees further. A hash chain is used to commit the entire order history in a single value.

The general outline of the auction proof is as follows:
1. All placed bids/offers are loaded.
2. All revealed prices for placed bids/offers are loaded.
3. Bids are validated to be overcollateralized.
3. The hash chain value that is stored onchain is reconstructed.
4. A clearing price that clears the market is computed.
5. Bids and offers are either fully assigned, partially assigned, or left on the table and unlocked.
6. A single cryptographic commitment is computed, encoding the the entire auction results into the root of a [lean incremental Merkle tree](https://zkkit.pse.dev/classes/_zk_kit_lean_imt.LeanIMT.html).

### Performance Metrics

#### Cycle Count

As the auction size and volume can vary greatly, so does the cycle count for verifying it. Here is a very rough estimate:

| **Total Orders** | **Cycle Count** |
|:----------------:|:---------------:|
|        0         | ~20,000 |
|       10         | ~170,000 |
|       100        | ~1,700,000 |
|      1,000       | ~17,500,000 |

The actual number of cycles can vary greatly: orders can get either fully filled, partially filled, or be left on the table.

#### Time Constraints

Bids need to be validated by checking that their purchase amount is overcollateralized, else the protocol can incur in bad debt. To this end, we rely on onchain oracles as our price feed, meaning **the current onchain oracle price must be an input to the proof**. The protocol can become effectively halted if the oracle updates **faster than we can generate proofs**. A quick look at Chainlink's [Data Feeds](https://data.chain.link/feeds) shows that price updates are not overly consistent, meaning our proving time should ideally be in the *minutes*.

Since this application does not need frequent proving, maintaining a server to generate these proofs is not ideal. It would therefore rely on third parties like [Succinct's prover network](https://docs.succinct.xyz/generating-proofs/prover-network.html) to generate them. Although proving time on the prover network varies greatly, all tests made have resulted in times **faster than 5 minutes**, which should be acceptable for this use case.

#### Gas Savings

We can clearly see the significant gas cost reductions from this ZK-powered architecture:

| **Action** | **Fully Onchain Cost** | **ZK-Powered Cost** | **Cost Reduction** |
|:----------------:|:---------------:|:----------------:|:---------------:|
| `lockBid` | 460,000 | 45,000 | **10x** |
| `lockOffer` | 260,000 | 45,000 | 5x |
| `revealBid` | 77,000 | 27,000 | 3x |
| `revealOffer` | 77,000 | 27,000 | 3x |
| `settleAuction` | >1,250,000 | ~450,000 | **3-infinite**x

There are two key takeaways from this gas cost breakdown:
1. The gas costs to place and reveal orders are *directly incurred* by users. It is easy to see how this ZK-powered architecture would make interacting with the protocol more attractive to users, just by virtue of lowering their transaction fees.
2. The gas cost for settling the auction is incurred by "the protocol", and so it must be amortized by the accrued fees. Lowering this cost, as well as making it predictable by fixing it, can help reduce overall protocol fees.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)

## Running the Project

There are four main ways to run this project: build a program, execute a program, generate a core proof, and
generate an EVM-compatible proof.

### Build the Program

To build the program, run the following command:

```sh
cd program
cargo prove build
```

### Execute the Program

To run the program without generating a proof:

```sh
cd script
cargo run --release -- --execute
```

This will execute the program and display the output.

### Generate a Core Proof

To generate a core proof for your program:

```sh
cd script
cargo run --release -- --prove
```

### Generate an EVM-Compatible Proof

> [!WARNING]
> You will need at least 128GB RAM to generate a Groth16 or PLONK proof.

To generate a proof that is small enough to be verified on-chain and verifiable by the EVM:

```sh
cd script
cargo run --release --bin evm -- --system groth16
```

this will generate a Groth16 proof. If you want to generate a PLONK proof, run the following command:

```sh
cargo run --release --bin evm -- --system plonk
```

These commands will also generate fixtures that can be used to test the verification of SP1 zkVM proofs
inside Solidity.

### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command:

```sh
cargo prove vkey --elf elf/riscv32im-succinct-zkvm-elf
```

## Using the Prover Network

We highly recommend using the Succinct prover network for any non-trivial programs or benchmarking purposes. For more information, see the [setup guide](https://docs.succinct.xyz/generating-proofs/prover-network.html).

To get started, copy the example environment file:

```sh
cp .env.example .env
```

Then, set the `SP1_PROVER` environment variable to `network` and set the `SP1_PRIVATE_KEY`
environment variable to your whitelisted private key.

For example, to generate an EVM-compatible proof using the prover network, run the following
command:

```sh
SP1_PROVER=network SP1_PRIVATE_KEY=... cargo run --release --bin evm
```

## License

This work is licensed under [CC BY-NC-ND 4.0](https://creativecommons.org/licenses/by-nc-nd/4.0/).
