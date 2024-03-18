
#[cfg(test)]
mod buy_tests {
    #[test]
    fn buy_with_funding() {
        assert_eq!(logic::should_buy(-1, 1000, 100), true);
    }

    fn buy_but_no_funding() {
        assert_eq!(logic::should_buy(-1, 1, 100), false);
    }

    fn negative_no_buy_with_funding() {
        assert_eq!(logic::should_buy(-0.0000001, 1000, 100), false);
    }

    fn positive_no_buy_with_funding() {
        assert_eq!(logic::should_buy(9, 1000, 100), false);
    }

    fn positive_no_buy_no_funding() {
        assert_eq!(logic::should_buy(9, 1, 100), false);
    }

    fn negative_no_buy_no_funding() {
        assert_eq!(logic::should_buy(-0.0000001, 10, 100), false);
    }
}