# `#[php_impl]`

You can export an entire `impl` block to PHP. This exports all methods as well
as constants to PHP on the class that it is implemented on. This requires the
`#[php_class]` macro to already be used on the underlying struct. Trait
implementations cannot be exported to PHP.

If you do not want a function exported to PHP, you should place it in a separate
`impl` block.

If you want to use async Rust, use `#[php_async_impl]`, instead: see [here &raquo;](#async) for more info.

## Methods

Methods basically follow the same rules as functions, so read about the
[`php_function`] macro first. The primary difference between functions and
methods is they are bounded by their class object.

Class methods can take a `&self` or `&mut self` parameter. They cannot take a
consuming `self` parameter. Static methods can omit this `self` parameter.

To access the underlying Zend object, you can take a reference to a
`ZendClassObject<T>` in place of the self parameter, where the parameter is
annotated with the `#[this]` attribute. This can also be used to return a
reference to `$this`.

By default, all methods are renamed in PHP to the camel-case variant of the Rust
method name. This can be changed on the `#[php_impl]` attribute, by passing one
of the following as the `rename_methods` option:

- `"none"` - does not rename the methods.
- `"camelCase"` - renames all methods to camel case (default).
- `"snake_case"` - renames all methods to snake case.

For example, to disable renaming, change the `#[php_impl]` attribute to
`#[php_impl(rename_methods = "none")]`.

The rest of the options are passed as separate attributes:

- `#[defaults(i = 5, b = "hello")]` - Sets the default value for parameter(s).
- `#[optional(i)]` - Sets the first optional parameter. Note that this also sets
  the remaining parameters as optional, so all optional parameters must be a
  variant of `Option<T>`.
- `#[public]`, `#[protected]` and `#[private]` - Sets the visibility of the
  method.
- `#[rename("method_name")]` - Renames the PHP method to a different identifier,
  without renaming the Rust method name.

The `#[defaults]` and `#[optional]` attributes operate the same as the
equivalent function attribute parameters.

### Constructors

By default, if a class does not have a constructor, it is not constructable from
PHP. It can only be returned from a Rust function to PHP.

Constructors are Rust methods which can take any amount of parameters and
returns either `Self` or `Result<Self, E>`, where `E: Into<PhpException>`. When
the error variant of `Result` is encountered, it is thrown as an exception and
the class is not constructed.

Constructors are designated by either naming the method `__construct` or by
annotating a method with the `#[constructor]` attribute. Note that when using
the attribute, the function is not exported to PHP like a regular method.

Constructors cannot use the visibility or rename attributes listed above.

### Async

Using `#[php_async_impl]` instead of `#[php_impl]` allows us to expose any async Rust library to PHP, using [PHP fibers](https://www.php.net/manual/en/language.fibers.php), [php-tokio](https://github.com/danog/php-tokio) and the [PHP Revolt event loop](https://revolt.run) under the hood to handle async interoperability.  

This allows full compatibility with [amphp](https://amphp.org), [PSL](https://github.com/azjezz/psl), [reactphp](https://reactphp.org) and any other async PHP library based on [Revolt](https://revolt.run).

Traits annotated with `#[php_impl]` can freely expose any async function, using `await` and any async Rust library.  

Make sure to also expose the `php_tokio::EventLoop::init` and `php_tokio::EventLoop::wakeup` functions to PHP in order to initialize the event loop, as specified in the full example [here &rauquo;](#async-example).

Also, make sure to invoke `EventLoop::shutdown` in the request shutdown handler to clean up the tokio event loop before finishing the request.

See [here &rauquo;](#async-example) for the full example.

## Constants

Constants are defined as regular Rust `impl` constants. Any type that implements
`IntoZval` can be used as a constant. Constant visibility is not supported at
the moment, and therefore no attributes are valid on constants.

## Property getters and setters

You can add properties to classes which use Rust functions as getters and/or
setters. This is done with the `#[getter]` and `#[setter]` attributes. By
default, the `get_` or `set_` prefix is trimmed from the start of the function
name, and the remainder is used as the property name.

If you want to use a different name for the property, you can pass a `rename`
option to the attribute which will change the property name.

Properties do not necessarily have to have both a getter and a setter, if the
property is immutable the setter can be omitted, and vice versa for getters.

The `#[getter]` and `#[setter]` attributes are mutually exclusive on methods.
Properties cannot have multiple getters or setters, and the property name cannot
conflict with field properties defined on the struct.

As the same as field properties, method property types must implement both
`IntoZval` and `FromZval`.

## Example

Continuing on from our `Human` example in the structs section, we will define a
constructor, as well as getters for the properties. We will also define a
constant for the maximum age of a `Human`.

```rust,no_run
# #![cfg_attr(windows, feature(abi_vectorcall))]
# extern crate ext_php_rs;
# use ext_php_rs::{prelude::*, types::ZendClassObject};
# #[php_class]
# #[derive(Debug, Default)]
# pub struct Human {
#     name: String,
#     age: i32,
#     #[prop]
#     address: String,
# }
#[php_impl]
impl Human {
    const MAX_AGE: i32 = 100;

    // No `#[constructor]` attribute required here - the name is `__construct`.
    pub fn __construct(name: String, age: i32) -> Self {
        Self { name, age, address: String::new() }
    }

    #[getter]
    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    #[setter]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[getter]
    pub fn get_age(&self) -> i32 {
        self.age
    }

    pub fn introduce(&self) {
        println!("My name is {} and I am {} years old. I live at {}.", self.name, self.age, self.address);
    }

    pub fn get_raw_obj(#[this] this: &mut ZendClassObject<Human>) {
        dbg!(this);   
    }

    pub fn get_max_age() -> i32 {
        Self::MAX_AGE
    }
}
# #[php_module]
# pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
#     module
# }
# fn main() {}
```

Using our newly created class in PHP:

```php
<?php

$me = new Human('David', 20);

$me->introduce(); // My name is David and I am 20 years old.
var_dump(Human::get_max_age()); // int(100)
var_dump(Human::MAX_AGE); // int(100)
```

### Async example

In this example, we're exposing an async Rust HTTP client library called [reqwest](https://docs.rs/reqwest/latest/reqwest/) to PHP, using [PHP fibers](https://www.php.net/manual/en/language.fibers.php), [php-tokio](https://github.com/danog/php-tokio) and the [PHP Revolt event loop](https://revolt.run) under the hood to handle async interoperability.  

This allows full compatibility with [amphp](https://amphp.org), [PSL](https://github.com/azjezz/psl), [reactphp](https://reactphp.org) and any other async PHP library based on [Revolt](https://revolt.run).  

Currently, only Linux is supported by php-tokio.

```rust,no_run
# #![cfg(linux)]
# extern crate ext_php_rs;
# use ext_php_rs::prelude::*;
use php_tokio::EventLoop;

#[php_class]
struct Client {}

#[php_async_impl]
impl Client {
    pub fn init() -> PhpResult<u64> {
        EventLoop::init()
    }
    pub fn wakeup() -> PhpResult<()> {
        EventLoop::wakeup()
    }
    pub async fn get(url: &str) -> anyhow::Result<String> {
        Ok(reqwest::get(url).await?.text().await?)
    }
}

pub extern "C" fn request_shutdown(_type: i32, _module_number: i32) -> i32 {
    EventLoop::shutdown();
    0
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module.request_shutdown_function(request_shutdown)
}

# fn main() {}
```

Here's the async PHP code we use to interact with the Rust class we just exposed.  

The `Client::init` method needs to be called only once in order to initialize the Revolt event loop and link it to the Tokio event loop, as shown by the following code.

See [here &raquo;](https://amphp.org) for more info on async PHP using [amphp](https://amphp.org) + [revolt](https://revolt.run).  

```php
<?php declare(strict_types=1);

namespace Reqwest;

use Revolt\EventLoop;

use function Amp\async;
use function Amp\Future\await;

final class Client
{
    private static ?string $id = null;

    public static function init(): void
    {
        if (self::$id !== null) {
            return;
        }

        $f = \fopen("php://fd/".\Client::init(), 'r+');
        \stream_set_blocking($f, false);
        self::$id = EventLoop::onReadable($f, fn () => \Client::wakeup());
    }

    public static function reference(): void
    {
        EventLoop::reference(self::$id);
    }
    public static function unreference(): void
    {
        EventLoop::unreference(self::$id);
    }

    public static function __callStatic(string $name, array $args): mixed
    {
        return \Client::$name(...$args);
    }
}


Client::init();

function test(int $delay): void
{
    $url = "https://httpbin.org/delay/$delay";
    $t = time();
    echo "Making async reqwest to $url that will return after $delay seconds...".PHP_EOL;
    Client::get($url);
    $t = time() - $t;
    echo "Got response from $url after ~".$t." seconds!".PHP_EOL;
};

$futures = [];
$futures []= async(test(...), 5);
$futures []= async(test(...), 5);
$futures []= async(test(...), 5);

await($futures);
```

Result:

```
Making async reqwest to https://httpbin.org/delay/5 that will return after 5 seconds...
Making async reqwest to https://httpbin.org/delay/5 that will return after 5 seconds...
Making async reqwest to https://httpbin.org/delay/5 that will return after 5 seconds...
Got response from https://httpbin.org/delay/5 after ~5 seconds!
Got response from https://httpbin.org/delay/5 after ~5 seconds!
Got response from https://httpbin.org/delay/5 after ~5 seconds!
```

[`php_function`]: ./function.md
