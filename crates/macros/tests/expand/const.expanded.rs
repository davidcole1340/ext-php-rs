#[macro_use]
extern crate ext_php_rs_derive;
const MY_CONST: &str = "Hello, world!";
#[allow(non_upper_case_globals)]
const _internal_const_docs_MY_CONST: &[&str] = &[];
#[allow(non_upper_case_globals)]
const _internal_const_name_MY_CONST: &str = "MY_CONST";
fn main() {
    (_internal_const_name_MY_CONST, MY_CONST, _internal_const_docs_MY_CONST);
}
