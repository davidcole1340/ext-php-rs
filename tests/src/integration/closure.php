<?php

require('_utils.php');

// Closure
assert(test_closure()('once') === 'once');
assert(test_closure()('twice') === 'twice');

// Closure once
$closure = test_closure_once('test');

assert($closure() === 'test');
assert_exception_thrown(fn () => $closure());
