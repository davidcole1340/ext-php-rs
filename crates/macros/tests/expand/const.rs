#[macro_use]
extern crate ext_php_rs_derive;

#[php_const]
const MY_CONST: &str = "Hello, world!";

fn main() {
    wrap_constant!(MY_CONST);
}
