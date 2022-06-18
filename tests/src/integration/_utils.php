<?php
// Active assert and make it quiet
assert_options(ASSERT_ACTIVE, 1);
assert_options(ASSERT_WARNING, 0);

function assert_exception_thrown(callable $callback): void
{
    try {
        call_user_func($callback);
    } catch (\Throwable $th) {
        return;
    }
    throw new Exception("Excption was not thrown", 255);
}
