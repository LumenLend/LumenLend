//! Fixed-point arithmetic helpers (18-decimal, i128-based).
//!
//! All protocol rates, utilization ratios and prices are represented as
//! 18-decimal fixed-point integers (e.g. `1e18` == `1.0`, `0.5e18` == `0.5`).
//! Configuration parameters (base rate, slopes, optimal utilization,
//! reserve factor) are expressed in basis points (`1/10000`).

/// Fixed-point scale: `1.0` in 18-decimal fixed point.
pub const SCALE: i128 = 1_000_000_000_000_000_000;

/// Number of basis points in `1.0`.
const BASIS_POINTS: i128 = 10_000;

/// Converts a basis-point value (`1/10000`) into 18-decimal fixed point.
///
/// e.g. `from_basis_points(200)` == `0.02e18` (2%).
pub fn from_basis_points(bp: u32) -> i128 {
    (bp as i128) * SCALE / BASIS_POINTS
}

/// Computes `(a * b) / denom` in 18-decimal fixed point using checked
/// arithmetic.
///
/// Panics on division by zero or integer overflow — callers must never
/// observe a silently truncated result on an overflow path.
pub fn mul_div(a: i128, b: i128, denom: i128) -> i128 {
    if denom == 0 {
        panic!("InterestRateModel: division by zero");
    }
    match a
        .checked_mul(b)
        .and_then(|product| product.checked_div(denom))
    {
        Some(result) => result,
        None => panic!("InterestRateModel: fixed-point arithmetic overflow"),
    }
}
