pub fn should_buy(hist: f64, funding: f64, price: f64) -> bool {
    return hist < -0.1 && funding > price;
}

pub fn should_sell(hist: f64, buys: u64) -> bool {
    return hist > 0.1 && buys > 0;
}