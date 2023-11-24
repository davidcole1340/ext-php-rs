<?php

require('_utils.php');

assert(is_null(test_nullable()));
assert(!is_null(test_nullable('value')));
