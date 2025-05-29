<?php

require(__DIR__ . '/../_utils.php');

// Signed
assert(testNumberSigned(-12) === -12);
assert(testNumberSigned(0) === 0);
assert(testNumberSigned(12) === 12);

// Unsigned
assert(testNumberUnsigned(0) === 0);
assert(testNumberUnsigned(12) === 12);
assert_exception_thrown(fn () => testNumberUnsigned(-12));

// Float
assert(round(testNumberFloat(-1.2), 2) === round(-1.2, 2));
assert(round(testNumberFloat(0.0), 2) === round(0.0, 2));
assert(round(testNumberFloat(1.2), 2) === round(1.2, 2));
