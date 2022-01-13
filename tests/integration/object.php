<?php

$obj = new stdClass;
$obj->first = 1;
$obj->second = 2;
$obj->third = 3;

foreach (test_object($obj) as $key => $value) {
    $output .= "{$key}={$value} ";
}

echo trim($output);
