<?php


$bin = test_binary(pack('L*', 1, 2, 3, 4, 5));

echo implode(' ', unpack('L*', $bin));
