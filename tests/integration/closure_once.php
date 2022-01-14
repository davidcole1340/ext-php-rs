<?php

echo test_closure_once('closure works')();
echo ' ';

try {
    echo test_closure_once('twice');
} catch (\Throwable $th) {
    echo 'once';
}
