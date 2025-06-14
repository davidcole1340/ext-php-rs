<?php

assert(iterNext([]) === []);
assert(iterNext([1, 2, 3]) === [0, 1, 1, 2, 2, 3]);
assert(iterBack([]) === []);
assert(iterBack([1, 2, 3]) === [2, 3, 1, 2, 0, 1]);

assert(iterNextBack([], 2) === [null, null]);
assert(iterNextBack([1, 2 ,3], 2) === [2, 3, 0, 1, 1, 2, null, null]);
var_dump(iterNextBack([1, 2, 3, 4, 5], 3));
assert(iterNextBack([1, 2, 3, 4, 5], 3) === [4, 5, 0, 1, 1, 2, 3, 4, 2, 3, null, null, null]);
