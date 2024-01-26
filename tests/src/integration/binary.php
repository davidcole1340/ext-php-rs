<?php

require('_utils.php');

$bin = test_binary(pack('L*', 1, 2, 3, 4, 5));
$result = unpack('L*', $bin);

assert(count($result) === 5);
assert(in_array(1, $result));
assert(in_array(2, $result));
assert(in_array(3, $result));
assert(in_array(4, $result));
assert(in_array(5, $result));