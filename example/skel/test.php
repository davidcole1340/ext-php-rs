<?php

$x = new TestClass();

var_dump($x->call(function ($v1, $v2) {
    // var_dump($v1, $v2);
    // echo "Hello, world! I'm a callable.".PHP_EOL;
    // return "Ok rust";
    return 0;
}));