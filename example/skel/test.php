<?php

include __DIR__.'/vendor/autoload.php';

//$y = new \stdClass;
//$y->hello = 'asdf';

$x = new TestClass();
var_dump($x);
$x->get();
var_dump($x);
