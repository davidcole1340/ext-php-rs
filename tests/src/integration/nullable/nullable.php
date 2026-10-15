<?php

require __DIR__ . '/../_utils.php';

assert(is_null(test_nullable()));
assert(!is_null(test_nullable('value')));

$param = ( new ReflectionFunction('test_nullable') )->getParameters()[0];
assert($param->allowsNull());
assert($param->isOptional());
assert($param->isDefaultValueAvailable());
assert($param->getDefaultValue() === null);

assert(test_nullable_int(5) === 5);
assert(test_nullable_int(null) === null);
assert(test_nullable_int() === null);
assert_type_error(fn() => test_nullable_int('5'));
assert_type_error(fn() => test_nullable_int([]));
assert(test_nullable_float(1) === 1.0);

$null = null;
assert(test_nullable_ints([1, null, &$null]) === [1, null, null]);
assert_type_error(fn() => test_nullable_ints([1, 'two']));
