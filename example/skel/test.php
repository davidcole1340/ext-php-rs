<?php

include __DIR__.'/vendor/autoload.php';

//$y = new \stdClass;
//$y->hello = 'asdf';

$x = ['hello' => 'world'];
var_dump(skel_unpack($x));

$x = new TestClass();
var_dump($x->call([$x, 'get']));
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

