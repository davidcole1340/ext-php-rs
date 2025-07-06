<?php

declare(strict_types=1);

$enum_variant = TestEnum::Variant1;
var_dump($enum_variant);
assert($enum_variant === TestEnum::Variant1);
assert($enum_variant !== TestEnum::Variant2);

$backed = IntBackedEnum::Variant1;
assert($backed->value === 1);
