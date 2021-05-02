<?php

include __DIR__.'/vendor/autoload.php';

$x = pack('f*', 1234, 5678, 9012);
var_dump(unpack('l*', skel_unpack($x)));
dd($x);
