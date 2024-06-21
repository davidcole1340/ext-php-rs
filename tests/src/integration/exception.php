<?php

require('_utils.php');

assert_specific_exception_thrown(fn() => throw throw_default_exception(), \Exception::class);

assert_specific_exception_thrown(fn() => throw throw_custom_exception(), \Test\TestException::class);

try {
    throw throw_custom_exception();
} catch (\Throwable $e) {
    // Check if object is initiated
    assert("Not good custom!" === $e->getMessage());
}