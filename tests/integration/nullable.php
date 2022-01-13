<?php

echo test_nullable() ?? 'null';
echo ' ';
echo test_nullable('not_null') ?? 'null';
