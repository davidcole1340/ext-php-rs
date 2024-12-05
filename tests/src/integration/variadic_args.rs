#[test]
fn test_variadic_optional_args() {
    assert!(crate::integration::run_php("variadic_args.php"));
}
