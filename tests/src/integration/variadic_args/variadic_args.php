<?php

require __DIR__ . "/../_utils.php";

$a = 'a';
$b = 'b';
$c = 'c';

$array = [$b, 'a','c'];

// Passing arguments as references
$args = testVariadicArgs();
assert($args === [], 'Expected no arguments to be returned');

$args = testVariadicArgs($a);
assert($args === ['a'], 'Expected to return argument $a');

$args = testVariadicArgs($a, $b, $c);
assert($args === ['a', 'b', 'c'], 'Expected to return arguments $a, $b and $c');

$args = testVariadicArgs(...$array);
assert($args === ['b', 'a', 'c'], 'Expected to return an array with the array $array');

assert_exception_thrown('testVariadicAddRequired');

// Values directly passed
$sum = testVariadicAddRequired(1, 2, 3); // 1
assert($sum === 6, 'Expected to return 6');

$count = testVariadicAddRequired(11); // 11
assert($count === 11, 'Allow only one argument');

$types = testVariadicArgs('a', 1, ['abc', 'def', 0.01], true, new stdClass);
assert(gettype(end($types[2])) === 'double', 'Type of argument 2 and its last element should be a float of 0.01');
assert($types[3], 'Arg 4 should be boolean true');
assert($types[4] instanceof stdClass, 'Last argument is an instance of an StdClass');
