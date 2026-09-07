use liquidation_engine::calculator::{apply_close_factor, calculate_collateral_to_seize};

const SCALE: i128 = 1_000_000_000_000_000_000;

#[test]
fn test_calculate_collateral_to_seize_with_bonus() {
    // debt_amount = 10 tokens (18 dec), debt_price = 1e18 (1 USD),
    // collateral_price = 1e18 (1 USD), bonus = 500 bp (5%).
    let debt = 10 * SCALE;
    let debt_price = SCALE;
    let collateral_price = SCALE;
    let bonus = 500;

    let seize = calculate_collateral_to_seize(debt, debt_price, collateral_price, bonus);
    // expect: 10 * 1e18 * 10500 / (1e18 * 10000) = 10.5 tokens
    assert_eq!(seize, 10 * SCALE * 1050 / 1000);
}

#[test]
fn test_calculate_collateral_to_seize_no_bonus() {
    let debt = 5 * SCALE;
    let seize = calculate_collateral_to_seize(debt, SCALE, SCALE, 0);
    assert_eq!(seize, 5 * SCALE);
}

#[test]
fn test_calculate_collateral_to_seize_scales_price_difference() {
    // Debt priced at 2 USD; collateral at 1 USD, no bonus.
    // Seize = 4e18 * 2e18 / (1e18) = 8e18.
    let debt = 4 * SCALE;
    let debt_price = 2 * SCALE;
    let collateral_price = SCALE;
    let seize = calculate_collateral_to_seize(debt, debt_price, collateral_price, 0);
    assert_eq!(seize, 8 * SCALE);
}

#[test]
#[should_panic]
fn test_calculate_collateral_to_seize_zero_collateral_price() {
    calculate_collateral_to_seize(SCALE, SCALE, 0, 500);
}

#[test]
fn test_apply_close_factor_half() {
    assert_eq!(apply_close_factor(100), 50);
    assert_eq!(apply_close_factor(0), 0);
}

#[test]
fn test_apply_close_factor_odd_debt() {
    // Integer division: 99 / 2 = 49.
    assert_eq!(apply_close_factor(99), 49);
}