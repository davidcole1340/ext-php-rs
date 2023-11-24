<?php

function create_generator() {
    yield 1;
    yield 2;
    yield 3;
}

class TestIterator implements \Iterator {
    private $count = 0;

    public function current()
    {
        return match ($this->count) {
            0 => 'foo',
            1 => 'bar',
            2 => 'baz',
            default => null,
        };
    }

    public function next()
    {
        $this->count++;
    }

    public function key()
    {
        return match ($this->count) {
            0 => 'key',
            1 => 10,
            2 => 2,
            default => null,
        };
    }

    public function valid()
    {
        return $this->count < 3;
    }

    public function rewind()
    {
        $this->count = 0;
    }
}

$generator = create_generator();
$iterator = new TestIterator();
