<?php

declare(strict_types=1);

require(__DIR__ . '/../_utils.php');

// Tests constructor
$class = test_class('lorem ipsum', 2022);
assert($class instanceof Foo\TestClass);

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

// Call regular from object
assert($class->staticCall('Php') === 'Hello Php');

// Call static from object
assert($class::staticCall('Php') === 'Hello Php');

// Call static from class
assert(Foo\TestClass::staticCall('Php') === 'Hello Php');

assert_exception_thrown(fn() => $class->private_string = 'private2');
assert_exception_thrown(fn() => $class->private_string);
