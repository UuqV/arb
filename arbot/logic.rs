mod src::logic;

#[cfg(test)]
mod tests {
    #[test]
    fn buy_with_funding() {
        assert_eq!(logic::should_buy(-1, 1000, 100), true);
    }
}