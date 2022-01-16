#[test]
fn callable_works() {
    assert!(crate::integration::run_php("callable.php"));
}