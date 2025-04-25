#[cfg(test)]
mod tests {
    #[test]
    fn types_work() {
        assert!(crate::integration::test::run_php("types/types.php"));
    }
}
