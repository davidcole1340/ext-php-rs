<?php

require "_utils.php";

$a = 'a';
$b = 'b';
$c = 'c';

// Passing arguments as references
$args = test_variadic_optional_args();
assert(count($args) === 0, 'Expected no arguments to be returned');

$args = test_variadic_optional_args($a);
assert(count($args) === 1, 'Expected to have 1 argument');

$args = test_variadic_optional_args($a, $b, $c);
assert(count($args) === 3, 'Expected to have 3 argument');
assert($args[1] === $b, 'Expected second argument to have a value of \$b aka "b"');
assert($args[2] === $c, 'Expected third argument to have a value of \$c aka "c"');

// Must have arguments.. so catch ArgumentCountError errors!
assert_exception_thrown('test_variadic_args');

// Values directly passed
test_variadic_add_optional(1, 2, 3); // 1

$count = test_variadic_add_optional(11); // 11
assert($count === 11, 'Allow only one argument');

$numbers = test_variadic_add_required(1, 2, 3, 4);
assert($numbers === [1, 2, 3, 4], 'Must return a array of numbers');

$types = test_variadic_all_types('a', 1, ['abc', 'def', 0.01], true, new stdClass);
assert(gettype(end($types[2])) === 'double', 'Type of argument 2 and its last element should be a float of 0.01');
assert($types[3], 'Arg 4 should be boolean true');
assert($types[4] instanceof stdClass, 'Last argument is an instance of an StdClass');
