<?php

include __DIR__.'/vendor/autoload.php';

//$y = new \stdClass;
//$y->hello = 'asdf';

$x = new TestClass();
var_dump($x);
$x->get();
$x->asdf = 10;
$x->hello = 'asdf';
var_dump($x);
$x->get();

$y = new \stdClass;
$y->hello = 'world';
$y->world = 'hello';

$x->debug($y);
var_dump($y);

