<?php

$x = new PhpFuture();
var_dump($x->obj()->get_str());
$x->then(function ($h) {
    var_dump($h);
});

$x->now();
exit;

var_dump('program starting');
$x = new Test();
$x->set_str('Hello World');
var_dump($x->get_str());
var_dump($x->get());
# $x->test = 'hello world';
var_dump($x->get());
var_dump($x->get_str());
// var_dump($x);
var_dump('program done');
// var_dump($x->get_str());
