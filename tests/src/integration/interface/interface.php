<?php

declare(strict_types=1);

assert(interface_exists('ExtPhpRs\Interface\EmptyObjectInterface'), 'Interface not exist');

assert(is_a('ExtPhpRs\Interface\EmptyObjectInterface', Throwable::class, true), 'Interface could extend Throwable');


final class Test extends Exception implements ExtPhpRs\Interface\EmptyObjectInterface
{
	public static function void(): void
	{
	}

	public function nonStatic(string $data): string
	{
		return sprintf('%s - TEST', $data);
	}

	public function refToLikeThisClass(
		string $data,
		ExtPhpRs\Interface\EmptyObjectInterface $other,
	): string {
		return sprintf('%s | %s', $this->nonStatic($data), $other->nonStatic($data));
	}

    public function setValue(?int $value = 0) {

    }
}
$f = new Test();

assert(is_a($f, Throwable::class));
assert($f->nonStatic('Rust') === 'Rust - TEST');
assert($f->refToLikeThisClass('TEST', $f) === 'TEST - TEST | TEST - TEST');
assert(ExtPhpRs\Interface\EmptyObjectInterface::STRING_CONST === 'STRING_CONST');
assert(ExtPhpRs\Interface\EmptyObjectInterface::USIZE_CONST === 200);
