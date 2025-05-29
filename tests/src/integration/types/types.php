<?php

const TYPES = [
    'testStr' => [['string'], 'string'],
    'testString' => [['string'], 'string'],
    'testBool' => [['bool'], 'bool'],
    'testNumberSigned' => [['int'], 'int'],
    'testNumberUnsigned' => [['int'], 'int'],
    'testNumberFloat' => [['float'], 'float'],
    'testArray' => [['array'], 'array'],
    'testArray' => [['array'], 'array'],
    'testArrayAssoc' => [['array'], 'array'],
    'testBinary' => [['string'], 'string'],
    'testNullable' => [['?string'], '?string'],
    'testObject' => [['object'], 'object'],
    'testClosure' => [[], 'RustClosure'],
    'testClosureOnce' => [['string'], 'RustClosure'],
    'testCallable' => [['callable', 'string'], 'mixed']
];

function toStr(ReflectionNamedType|ReflectionUnionType|ReflectionIntersectionType|null $v): string {
    if ($v === null) {
        return '<null>';
    }
    return match (true) {
        $v instanceof ReflectionNamedType => $v->allowsNull() && $v->getName() !== 'mixed' ? '?'.$v->getName() : $v->getName(),
        $v instanceof ReflectionUnionType => $v->getName(),
        $v instanceof ReflectionIntersectionType => $v->getName(),
    };
}

foreach (TYPES as $func => [$args, $return]) {
    $f = new ReflectionFunction($func);
    $tReturn = toStr($f->getReturnType());
    assert($tReturn === $return, "Wrong return type of $func, expected $return, got $tReturn");
    foreach ($f->getParameters() as $idx => $param) {
        $tParam = toStr($param->getType());
        assert($tParam === $args[$idx], "Wrong arg type $idx of $func, expected {$args[$idx]}, got $tParam");
    }
}
