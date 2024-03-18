pub fn should_buy(hist: f64, funding: f64, price: f64) -> bool {
    return hist < -(funding / 10000.0) && funding > price;
}

pub fn should_sell(hist: f64, funding: f64, buys: u64) -> bool {
    return hist > (funding / 10000.0) && buys > 0;
}


#[cfg(test)]
mod buy_tests {
    use super::*;

    #[test]
    fn buy_with_funding() {
        assert_eq!(should_buy(-1.0, 1000.0, 100.0), true);
    }

    #[test]
    fn buy_but_no_funding() {
        assert_eq!(should_buy(-1.0, 1.0, 100.0), false);
    }

    #[test]
    fn negative_no_buy_with_funding() {
        assert_eq!(should_buy(-0.0000001, 1000.0, 100.0), false);
    }

    #[test]
    fn positive_no_buy_with_funding() {
        assert_eq!(should_buy(9.0, 1000.0, 100.0), false);
    }

    #[test]
    fn positive_no_buy_no_funding() {
        assert_eq!(should_buy(9.0, 1.0, 100.0), false);
    }

    #[test]
    fn negative_no_buy_no_funding() {
        assert_eq!(should_buy(-0.0000001, 10.0, 100.0), false);
    }
}