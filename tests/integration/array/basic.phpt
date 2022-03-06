--TEST--
basic array functionality
--FILE--
<?php

$array = test_array(['a', 'b', 'c']);

var_dump(is_array($array));
var_dump(count($array));
var_dump($array);

$assoc = test_assoc_array([
    'a' => 1,
    'b' => 2,
    'c' => 3,
]);

var_dump(is_array($assoc));
var_dump(count($assoc));

// Hashmap order not guaranteed
var_dump($assoc['a']);
var_dump($assoc['b']);
var_dump($assoc['c']);
?>
--EXPECT--
bool(true)
int(3)
array(3) {
  [0]=>
  string(1) "a"
  [1]=>
  string(1) "b"
  [2]=>
  string(1) "c"
}
bool(true)
int(3)
int(1)
int(2)
int(3)