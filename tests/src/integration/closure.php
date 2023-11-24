<?php

require('_utils.php');

$v = test_closure();

// Closure
assert($closure('works') === 'works');

// Closure once
$closure = test_closure_once('test');

assert(call_user_func($closure) === 'test');
assert_exception_thrown($closure);
