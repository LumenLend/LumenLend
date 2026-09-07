/// Close-factor and liquidation bonus calculations.
///
/// All token amounts and USD prices are 18-decimal fixed-point
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
/// All amounts are in 18-decimal fixed point; the 18-decimal precision
/// cancels between the USD-denominated numerator and denominator, leaving a
/// collateral token amount in 18-decimal fixed point. Intermediate products
/// are scaled via `mul_div` to avoid i128 overflow.
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
    if debt_price <= 0 {
        panic!("LiquidationEngine: debt price must be positive");
    }

    // (debt_amount * debt_price) / collateral_price  ->  collateral units
    // scaled into 18-decimal fixed point.
    let value = mul_div(debt_amount, debt_price, collateral_price);

    // Apply the bonus: value * (10000 + bonus) / 10000
    let bonus: i128 = liquidation_bonus as i128;
    let scaled = mul_div(value, 10_000 + bonus, 10_000);

    scaled
}

/// Multiply `a * b / c` with 18-decimal-checked arithmetic.
fn mul_div(a: i128, b: i128, c: i128) -> i128 {
    if c == 0 {
        panic!("LiquidationEngine: division by zero");
    }
    a.checked_mul(b)
        .and_then(|p| p.checked_div(c))
        .expect("LiquidationEngine: fixed-point arithmetic overflow")
}

/// Apply the close factor to a borrower's total outstanding debt.
///
/// Returns `total_debt / 2` (a 50% close factor — a liquidator may close at
/// most half of a borrower's debt in a single transaction).
pub fn apply_close_factor(total_debt: i128) -> i128 {
    if total_debt <= 0 {
        return 0;
    }
    total_debt / 2
}