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
}
$f = new Test();

assert(is_a($f, Throwable::class));
assert($f->nonStatic('Rust') === 'Rust - TEST');
assert($f->refToLikeThisClass('TEST', $f) === 'TEST - TEST | TEST - TEST');
