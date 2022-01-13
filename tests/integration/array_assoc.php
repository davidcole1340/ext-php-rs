<?php

foreach (test_array_assoc([
    'first' => '1',
    'second' => '2',
    'third' => '3'
]) as $key => $value) {
    $output .= "{$key}={$value} ";
}

echo trim($output);
