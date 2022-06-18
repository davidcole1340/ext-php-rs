<?php

require('_utils.php');

// Tests constructor
$class = test_class('lorem ipsum', 2022);
assert($class instanceof TestClass);

// Tests getter/setter
assert($class->getString() === 'lorem ipsum');
$class->setString('dolor et');
assert($class->getString() === 'dolor et');

assert($class->getNumber() === 2022);
$class->setNumber(2023);
assert($class->getNumber() === 2023);

// Tests #prop decorator
assert($class->boolean);
$class->boolean = false;
assert($class->boolean === false);
