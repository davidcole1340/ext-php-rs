<?php

include __DIR__.'/vendor/autoload.php';

function test(array $x)
{
    $x['hello'] = 'not world';
    return $x;
}

var_dump(hello_world("David"));

$x = ['hello' => 'world'];
var_dump(test($x));
var_dump($x);

//$y = new \stdClass;
//$y->hello = 'asdf';

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

