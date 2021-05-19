<?php

include __DIR__.'/vendor/autoload.php';

//$y = new \stdClass;
//$y->hello = 'asdf';

$x = new TestClass();
var_dump($x);
$x->prop = 150;
var_dump($x);
