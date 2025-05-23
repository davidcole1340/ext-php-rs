<?php

assert(test_defaults_integer() === 42);
assert(test_defaults_integer(12) === 12);
assert(test_defaults_nullable_string() === null);
assert(test_defaults_nullable_string('test') === 'test');
