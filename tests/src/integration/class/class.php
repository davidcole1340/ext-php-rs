<?php

require(__DIR__ . '/../_utils.php');

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

// Call regular from object
assert($class->staticCall('Php') === 'Hello Php');

// Call static from object
assert($class::staticCall('Php') === 'Hello Php');

// Call static from class
assert(TestClass::staticCall('Php') === 'Hello Php');

$ex = new TestClassExtends();
assert_exception_thrown(fn() => throw $ex);
assert_exception_thrown(fn() => throw_exception());

$arrayAccess = new TestClassArrayAccess();
assert_exception_thrown(fn() => $arrayAccess[0] = 'foo');
assert_exception_thrown(fn() => $arrayAccess['foo']);
assert($arrayAccess[0] === true);
assert($arrayAccess[1] === false);
