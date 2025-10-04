#[macro_use]
extern crate ext_php_rs_derive;

/// Doc comments for MyInterface.
/// This is a basic interface example.
#[php_interface]
trait MyInterface {
    /// Doc comments for my_method.
    /// This method does something.
    fn my_method(&self, arg: i32) -> String;
}
