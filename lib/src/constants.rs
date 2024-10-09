//! This module contains constants used throughout the project.

/// Maximum price that can be specified for a bid/offer in basis points (bps)
pub const MAX_BID_PRICE: u64 = 1_000_000; // 10,000% in bps

/// Maximum price that can be specified for an offer in basis points (bps)
pub const MAX_OFFER_PRICE: u64 = 1_000_000; // 10,000% in bps

/// Servicing fee in basis points (bps)
pub const SERVICING_FEE: u64 = 50; // 0.5% in bps
