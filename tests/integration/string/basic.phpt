--TEST--
basic string functionality
--FILE--
<?php
var_dump(test_string('hello, world!'));
var_dump(test_str('hello, world!'));
?>
--EXPECT--
string(13) "hello, world!"
string(13) "hello, world!"