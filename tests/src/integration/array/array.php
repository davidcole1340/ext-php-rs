<?php

// Tests sequential arrays
$array = test_array(['a', 'b', 'c', 'd']);
unset($array[2]);

assert(is_array($array));
assert(count($array) === 3);
assert(in_array('a', $array));
assert(in_array('b', $array));
assert(in_array('d', $array));

// Tests associative arrays
$assoc = test_array_assoc([
    'a' => '1',
    'b' => '2',
    'c' => '3'
]);

assert(array_key_exists('a', $assoc));
assert(array_key_exists('b', $assoc));
assert(array_key_exists('c', $assoc));
assert(in_array('1', $assoc));
assert(in_array('2', $assoc));
assert(in_array('3', $assoc));
