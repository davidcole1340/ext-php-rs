# Functions & methods

PHP functions and methods are represented by the `Function` struct.

You can use the `try_from_function` and `try_from_method` methods to obtain a Function struct corresponding to the passed function or static method name.
It's heavily recommended you reuse returned `Function` objects, to avoid the overhead of looking up the function/method name.

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_module]
mod module {
    use ext_php_rs::zend::Function;

    #[php_function]
    pub fn test_function() -> () {
        let var_dump = Function::try_from_function("var_dump").unwrap();
        let _ = var_dump.try_call(vec![&"abc"]);
    }

    #[php_function]
    pub fn test_method() -> () {
        let f = Function::try_from_method("ClassName", "staticMethod").unwrap();
        let _ = f.try_call(vec![&"abc"]);
    }
}
# fn main() {}
```
