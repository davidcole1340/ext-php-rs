<?php

assert(testDefaultsInteger() === 42);
assert(testDefaultsInteger(12) === 12);
assert(testDefaultsNullableString() === null);
assert(testDefaultsNullableString('test') === 'test');
