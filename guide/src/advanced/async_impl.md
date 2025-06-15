# `#[php_async_impl]` Attribute

Using `#[php_async_impl]` instead of `#[php_impl]` allows us to expose any async Rust library to PHP, using [PHP fibers](https://www.php.net/manual/en/language.fibers.php), [php-tokio](https://github.com/danog/php-tokio) and the [PHP Revolt event loop](https://revolt.run) under the hood to handle async interoperability.

This allows full compatibility with [amphp](https://amphp.org), [PSL](https://github.com/azjezz/psl), [reactphp](https://reactphp.org) and any other async PHP library based on [Revolt](https://revolt.run).

Traits annotated with `#[php_async_impl]` can freely expose any async function, using `await` and any async Rust library.

Make sure to also expose the `php_tokio::EventLoop::init` and `php_tokio::EventLoop::wakeup` functions to PHP in order to initialize the event loop, as specified in the full example [here &raquo;](#async-example).

Also, make sure to invoke `EventLoop::shutdown` in the request shutdown handler to clean up the tokio event loop before finishing the request.

## Async example

In this example, we're exposing an async Rust HTTP client library called [reqwest](https://docs.rs/reqwest/latest/reqwest/) to PHP, using [PHP fibers](https://www.php.net/manual/en/language.fibers.php), [php-tokio](https://github.com/danog/php-tokio) and the [PHP Revolt event loop](https://revolt.run) under the hood to handle async interoperability.

This allows full compatibility with [amphp](https://amphp.org), [PSL](https://github.com/azjezz/psl), [reactphp](https://reactphp.org) and any other async PHP library based on [Revolt](https://revolt.run).

Make sure to require [php-tokio](https://github.com/danog/php-tokio) as a dependency before proceeding.

<!-- Must ignore because of circular dependency with php_tokio. Otherwise, this _should_ work. -->
```rust,no_run,ignore
# extern crate ext_php_rs;
# extern crate php_tokio;
# extern crate reqwest;
use ext_php_rs::prelude::*;
use php_tokio::{php_async_impl, EventLoop};

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
