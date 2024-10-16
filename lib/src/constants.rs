//! This module contains constants used throughout the project.

/// Basis points
pub const BPS: u32 = 10_000;

/// Maximum price that can be specified for a bid/offer in basis points (bps)
pub const MAX_BID_PRICE: u32 = 1_000_000; // 10,000% in bps

/// Maximum price that can be specified for an offer in basis points (bps)
pub const MAX_OFFER_PRICE: u32 = 1_000_000; // 10,000% in bps

/// Initial collateral ratio in basis points (bps)
pub const INITIAL_COLLATERAL_RATIO: u32 = 15_000;

/// Servicing fee in basis points (bps)
/// NOTE: The current design does not define *any* fee. This is a placeholder for future fee and protocoldesign.
pub const SERVICING_FEE: u32 = 50; // 0.5% in bps

/// Number of days in a year for 360 day count convention
pub const DAYS_IN_YEAR: u32 = 360;
