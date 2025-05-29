<?php

require(__DIR__ . '/../_utils.php');

$v = testClosure();

// Closure
assert($v('works') === 'works');

// Closure once
$closure = testClosureOnce('test');

assert(call_user_func($closure) === 'test');
assert_exception_thrown($closure);

function take(\stdClass $rs): void { }

try {
    take($closure);
} catch (\TypeError $e) {
    assert(str_starts_with($e->getMessage(), 'take(): Argument #1 ($rs) must be of type stdClass, RustClosure given, called in '));
}
