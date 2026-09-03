/// Fixed-point math helpers (18-decimal, i128-based).

/// Multiply two i128 values with 18-decimal scaling, then divide by a third value.
/// Uses checked operations to avoid panics on overflow.
pub fn mul_div(a: i128, b: i128, c: i128) -> i128 {
    a.checked_mul(b).expect("mul_div: overflow in mul")
        .checked_div(c).expect("mul_div: division by zero")
}

/// Fixed-point scale constant: 1e18 = 1.0 in 18-decimal.
pub const SCALE: i128 = 1_000_000_000_000_000_000i128;