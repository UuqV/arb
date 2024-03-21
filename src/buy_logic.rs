pub fn should_buy( hist_threshold: f64, hist: f64, roc: f64, usdc: f64, price: f64,) -> bool {
    return check_hist_threshold(hist_threshold, hist) && check_buy_roc(roc) && check_buy_funding(usdc, price);
}

pub fn check_hist_threshold(hist_threshold: f64, hist: f64, ) -> bool {
    return hist > hist_threshold;
}

pub fn check_buy_funding(usdc: f64, price: f64) -> bool {
    return usdc > price;
}

pub fn check_buy_roc(roc: f64) -> bool {
    return roc <= 0.0;
}

#[cfg(test)]
mod buy_tests {
    use super::*;

    #[test]
    fn not_enough_buy_funding() { // Buys should have enough usdc to cover the buy plus 5
        assert_eq!(check_buy_funding(200.0, 200.0), false);
        assert_eq!(check_buy_funding(100.0, 200.0), false);
        assert_eq!(check_buy_funding(208.0, 200.0), true);
    }

    #[test]
    fn negative_buy_roc() { // Buys should happen when the ROC switches positive
        assert_eq!(check_buy_roc(0.01), true);
        assert_eq!(check_buy_roc(-0.01), false);
    }

    #[test]
    fn negative_hist_threshold() { // Buys should happen when histogram value is below a certain NEGATIVE threshold
        assert_eq!(check_hist_threshold(0.01, -0.015), true);
        assert_eq!(check_hist_threshold(0.01, -0.1), true);
        assert_eq!(check_hist_threshold(0.1, 0.1), false);
        assert_eq!(check_hist_threshold(0.01, -0.5), true);
    }


}