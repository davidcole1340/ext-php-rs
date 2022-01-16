<?php

require('_utils.php');

assert(test_callable(fn (string $a) => $a, 'test') === 'test');
