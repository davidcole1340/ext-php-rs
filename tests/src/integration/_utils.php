<?php
// Active assert and make it quiet
assert_options(ASSERT_ACTIVE, 1);
assert_options(ASSERT_WARNING, 0);
// Set up the callback
assert_options(ASSERT_CALLBACK, fn () => exit(1));

function assert_exception_thrown(callable $callback): void
{
    try {
        call_user_func($callback);
        exit(1);
    } catch (\Throwable $th) {
    }
}
