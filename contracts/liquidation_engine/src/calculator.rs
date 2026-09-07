/// Close-factor and liquidation bonus calculations.
///
/// All amounts and rates are expressed in 18-decimal fixed point
/// (`SCALE = 1e18` == `1.0`). Basis-point parameters are `1/10000`.

/// Fixed-point scale: `1.0` in 18-decimal fixed point.
pub const SCALE: i128 = 1_000_000_000_000_000_000;

/// Compute the amount of collateral a liquidator is entitled to receive.
///
/// Formula (matches README):
/// ```text
/// collateral_to_seize = debt_amount * debt_price * (10000 + liquidation_bonus)
///                     / (collateral_price * 10000)
/// ```
///
/// All prices are in 18-decimal fixed point; the result is an 18-decimal
/// fixed-point collateral amount.
pub fn calculate_collateral_to_seize(
    debt_amount: i128,
    debt_price: i128,
    collateral_price: i128,
    liquidation_bonus: u32,
) -> i128 {
    if debt_amount <= 0 {
        panic!("LiquidationEngine: debt amount must be positive");
    }
    if collateral_price <= 0 {
        panic!("LiquidationEngine: collateral price must be positive");
    }

    let bonus: i128 = liquidation_bonus as i128;
    let numerator = debt_amount
        .checked_mul(debt_price)
        .and_then(|v| v.checked_mul(10_000 + bonus))
        .expect("LiquidationEngine: seize numerator overflow");

    numerator / (collateral_price * 10_000)
}

/// Apply the close factor to a borrower's total outstanding debt.
///
/// Returns `total_debt / 2` (i.e. a 50% close factor — a liquidator may
/// close at most half of a borrower's debt in a single transaction).
pub fn apply_close_factor(total_debt: i128) -> i128 {
    if total_debt <= 0 {
        return 0;
    }
    total_debt / 2
}