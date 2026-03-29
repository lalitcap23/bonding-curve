use std::ops::{Div, Mul};

pub fn convert_to_float(value: u64, decimals: u8) -> f64 {
    (value as f64).div(f64::powf(10.0, decimals as f64))
}

pub fn convert_from_float(value: f64, decimals: u8) -> u64 {
    value.mul(f64::powf(10.0, decimals as f64)) as u64
}

pub fn calculate_price_at_point(
    initial_sol_reserve: u64,
    additional_sol: u64,
    token_supply: u64,
) -> f64 {
    const CRR: f64 = 0.2; // Constant Reserve Ratio
    const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;
    
    let total_sol_reserve = initial_sol_reserve as f64 + additional_sol as f64;
    let token_reserve = token_supply as f64;
    
    // Price in SOL = reserve_sol / (reserve_tokens * CRR)
    let price_in_sol = total_sol_reserve / (token_reserve * CRR);
    
    price_in_sol / LAMPORTS_PER_SOL
}

pub fn get_curve_points(
    initial_sol_reserve: u64,
    curve_limit: u64,
    token_supply: u64,
) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let steps = 10; // Calculate 10 points along the curve
    
    for i in 0..=steps {
        let progress = i as f64 / steps as f64;
        let additional_sol = ((curve_limit - initial_sol_reserve) as f64 * progress) as u64;
        let price = calculate_price_at_point(initial_sol_reserve, additional_sol, token_supply);
        let sol_amount = (initial_sol_reserve as f64 + additional_sol as f64) / 1_000_000_000.0;
        points.push((sol_amount, price));
    }
    
    points
}

pub fn calculate_reserves_for_target_sol(
    target_sol_amount: f64,
    token_supply: u64,
    token_reserve_percentage: f64,
) -> (u64, u64, u64) {
    const CRR: f64 = 0.2; // Constant Reserve Ratio
    const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;
    
    // Calculate token reserve (80% of supply)
    let token_reserve = (token_supply as f64 * token_reserve_percentage) as u64;
    
    // Convert target SOL to lamports
    let target_lamports = target_sol_amount * LAMPORTS_PER_SOL;
    
    // Using Bancor formula:
    // For a smooth curve where we want exactly target_sol_amount when complete,
    // we set initial_reserve = target_amount / 4 (this gives us a good curve shape)
    let initial_sol_reserve = (target_lamports / 4.0) as u64;
    
    // Calculate the initial price in lamports that will achieve this curve
    // Using the Bancor pricing formula: price = sol_reserve / (token_reserve * CRR)
    let initial_price = ((initial_sol_reserve as f64 * LAMPORTS_PER_SOL) / (token_reserve as f64 * CRR)) as u64;
    
    // The curve limit will be our target SOL amount in lamports
    let curve_limit = (target_lamports) as u64;
    
    (initial_price, initial_sol_reserve, curve_limit)
}
