#[test]
fn exception_works() {
    assert!(crate::integration::run_php("exception.php"));
}
