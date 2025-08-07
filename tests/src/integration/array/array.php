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

$arrayKeys = test_array_keys();
assert($arrayKeys[-42] === "foo");
assert($arrayKeys[0] === "bar");
assert($arrayKeys[5] === "baz");
assert($arrayKeys[10] === "qux");
assert($arrayKeys["10"] === "qux");
assert($arrayKeys["quux"] === "quuux");

$assoc_keys = test_array_assoc_array_keys([
    'a' => '1',
    2 => '2',
    '3' => '3',
]);
assert($assoc_keys === [
    'a' => '1',
    2 => '2',
    '3' => '3',
]);
$assoc_keys = test_btree_map([
    'a' => '1',
    2 => '2',
    '3' => '3',
]);
assert($assoc_keys === [
    2 => '2',
    '3' => '3',
    'a' => '1',
]);

$assoc_keys = test_array_assoc_array_keys(['foo', 'bar', 'baz']);
assert($assoc_keys === [
    0 => 'foo',
    1 => 'bar',
    2 => 'baz',
]);
assert(test_btree_map(['foo', 'bar', 'baz']) === [
    0 => 'foo',
    1 => 'bar',
    2 => 'baz',
]);
