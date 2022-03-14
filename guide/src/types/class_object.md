# Class Object

A class object is an instance of a Rust struct (which has been registered as a
PHP class) that has been allocated alongside an object. You can think of a class
object as a superset of an object, as a class object contains a Zend object.

| `T` parameter | `&T` parameter        | `T` Return type | `&T` Return type          | PHP representation             |
| ------------- | --------------------- | --------------- | ------------------------- | ------------------------------ |
| No            | `&ZendClassObject<T>` | Yes             | `&mut ZendClassObject<T>` | Zend object and a Rust struct. |

## Examples

### Returning a reference to `self`

```rust
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::{prelude::*, types::ZendClassObject};

#[php_class]
pub struct Example {
    foo: i32,
    bar: i32
}

#[php_impl]
impl Example {
    // Even though this function doesn't have a `self` type, it is still treated as an associated method
    // and not a static method.
    pub fn builder_pattern(#[this] this: &mut ZendClassObject<Example>) -> &mut ZendClassObject<Example> {
        // do something with `this`
        this
    }
}
# #[php_module]
# pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
#     module
# }
```

### Creating a new class instance

```rust
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_class]
pub struct Example {
    foo: i32,
    bar: i32
}

#[php_impl]
impl Example {
    pub fn make_new(foo: i32, bar: i32) -> Example {
        Example { foo, bar }
    }
}
# #[php_module]
# pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
#     module
# }
```
