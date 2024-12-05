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
// try {
//     $args = test_variadic_args();
// } catch (ArgumentCountError $e) {
//     var_dump($e->getMessage());
// }
