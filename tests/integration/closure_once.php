<?php
$clo = test_closure_once('closure works');

echo $clo();
echo ' ';

try {
    echo test_closure_once('twice');
} catch (\Throwable $th) {
    echo 'once';
}
