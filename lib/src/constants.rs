//! This module contains constants used throughout the project.

/// Basis points
pub const BPS: u64 = 10_000;

/// Maximum price that can be specified for a bid/offer in basis points (bps)
pub const MAX_BID_PRICE: u64 = 1_000_000; // 10,000% in bps

/// Maximum price that can be specified for an offer in basis points (bps)
pub const MAX_OFFER_PRICE: u64 = 1_000_000; // 10,000% in bps

/// Initial collateral ratio in basis points (bps)
pub const INITIAL_COLLATERAL_RATIO: u64 = 15_000;

/// Servicing fee in basis points (bps)
pub const SERVICING_FEE: u64 = 50; // 0.5% in bps
