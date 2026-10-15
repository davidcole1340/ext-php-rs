<?php

function assert_exception_thrown(callable $callback): void
{
    try {
        call_user_func($callback);
    } catch (\Throwable $th) {
        return;
    }
    throw new Exception('Exception was not thrown', 255);
}

function assert_type_error(callable $callback): void
{
    try {
        call_user_func($callback);
    } catch (\TypeError) {
        return;
    }
    throw new Exception('TypeError was not thrown', 255);
}
