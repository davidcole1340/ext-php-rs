# Object

Objects can be returned from functions as instances or references. You can only
return a reference when you are returning an immutable reference to the object
the method is implemented on.

| `T` parameter | `&T` parameter | `T` Return type | `&T` Return type      | PHP representation               |
| ------------- | -------------- | --------------- | --------------------- | -------------------------------- |
| No            | No             | Yes             | Yes, as `ClassRef<T>` | A Rust struct and a Zend object. |

## Examples

### Returning a reference to `self`

```rust
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;
use ext_php_rs::php::types::object::ClassRef;

#[php_class]
#[derive(Default)]
pub struct Example {
    foo: i32,
    bar: i32
}

#[php_impl]
impl Example {
    pub fn builder_pattern(&self) -> ClassRef<Example> {
        // As long as you return `self` from a method, you can unwrap this option.
        ClassRef::from_ref(self).unwrap()
    }
}
```

### Creating a new class instance

```rust
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[php_class]
#[derive(Default)]
pub struct Example {
    foo: i32,
    bar: i32
}

#[php_impl]
impl Example {
    pub fn make_new(&self, foo: i32, bar: i32) -> Example {
        Example { foo, bar }
    }
}
```
