<?php

echo test_number_unsigned(0);
echo ' ';
echo test_number_unsigned(12);

try {
    echo test_number_unsigned(-12);
} catch (\Throwable $th) {
    echo ' invalid';
}