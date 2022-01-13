<?php

try {
    echo test_number_unsigned(-12);
} catch (\Throwable $th) {
    echo "thrown";
    echo ' ';
}

echo test_number_unsigned(0);
echo ' ';
echo test_number_unsigned(12);
