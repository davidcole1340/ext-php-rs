#[test]
fn globals_works() {
    assert!(crate::integration::run_php("globals.php"));
}
