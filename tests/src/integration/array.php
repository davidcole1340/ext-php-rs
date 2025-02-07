<?php

require('_utils.php');

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

// Test ZendHashtable drop logic
$immutable = test_zend_hashtable();
assert(!$immutable);

// Test immutable ZendHashtable drop logic
$immutable = test_immutable_zend_hashtable();
assert($immutable);

// Test that an immutable ZendHashtable is transparent to userland
$immutable = test_immutable_zend_hashtable_ret();
$immutable[] = 'fpp';
assert(count($immutable) === 1);

// Test empty array -> Vec -> array conversion
$empty = test_array( [] );
assert(is_array($empty));
assert(count($empty) === 0);

// Test empty array -> HashMap -> array conversion
$empty_assoc = test_array_assoc( [] );
assert(is_array($empty_assoc));
assert(count($empty_assoc) === 0);
