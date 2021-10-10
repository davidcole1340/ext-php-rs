# Object

An object is any object type in PHP. This can include a PHP class and PHP
`stdClass`. A Rust struct registered as a PHP class is a [class object], which
contains an object.

Objects are valid as parameters but only as an immutable or mutable reference.
You cannot take ownership of an object as objects are reference counted, and
multiple zvals can point to the same object. You can return a boxed owned
object.

| `T` parameter | `&T` parameter | `T` Return type    | `&T` Return type  | PHP representation |
| ------------- | -------------- | ------------------ | ----------------- | ------------------ |
| No            | Yes            | `ZBox<ZendObject>` | Yes, mutable only | Zend object.       |

## Examples

### Taking an object reference

```rust
# extern crate ext_php_rs;
use ext_php_rs::{prelude::*, types::ZendObject};

// Take an object reference and also return it.
#[php_function]
pub fn take_obj(obj: &mut ZendObject) -> &mut ZendObject {
    let _ = obj.set_property("hello", 5);
    dbg!(obj)
}
# #[php_module]
# pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
#     module
# }
```

### Creating a new object

```rust
# extern crate ext_php_rs;
use ext_php_rs::{prelude::*, types::ZendObject, boxed::ZBox};

// Create a new `stdClass` and return it.
#[php_function]
pub fn make_object() -> ZBox<ZendObject> {
    let mut obj = ZendObject::new_stdclass();
    let _ = obj.set_property("hello", 5);
    obj
}
# #[php_module]
# pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
#     module
# }
```

[class object]: ./class_object.md
