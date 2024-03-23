pub fn should_sell(hist_threshold: f64, hist: f64, roc: f64, sol: f64) -> bool {
    return check_hist_threshold(hist_threshold, hist) && check_sell_roc(roc) && check_sell_funding(sol);
}

pub fn check_hist_threshold(hist_threshold: f64, hist: f64) -> bool {
    return hist > hist_threshold;
}

pub fn check_sell_funding(sol: f64) -> bool {
    return sol > 1.01;
}

pub fn check_sell_roc(roc: f64) -> bool {
    return roc.abs() < 0.01;
}

#[cfg(test)]
mod sell_tests {
    use super::*;

    #[test]
    fn not_enough_sell_funding() { // Sells should have at least one sol plus a little to cover
        assert_eq!(check_sell_funding(2.0), true);
        assert_eq!(check_sell_funding(1.0), false);
        assert_eq!(check_sell_funding(0.5), false);
    }

    #[test]
    fn negative_sell_roc() { // Sells should happen when the ROC is sufficiently low
        assert_eq!(check_sell_roc(0.01), false);
        assert_eq!(check_sell_roc(-0.002), true);
        assert_eq!(check_sell_roc(0.002), true);
        assert_eq!(check_sell_roc(-0.01), false);
        assert_eq!(check_sell_roc(0.01), false);
    }

    #[test]
    fn positive_hist_threshold() { // Sells should happen when histogram value is above a certain threshold
        assert_eq!(check_hist_threshold(0.01, 0.01), false);
        assert_eq!(check_hist_threshold(0.01, -0.1), false);
        assert_eq!(check_hist_threshold(0.1, 0.05), false);
        assert_eq!(check_hist_threshold(0.01, 0.05), true);
    }


}