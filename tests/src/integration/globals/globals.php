<?php

assert(testGlobalsHttpGet() === []);
assert(testGlobalsHttpPost() === []);
assert(testGlobalsHttpCookie() === []);
assert(!empty(testGlobalsHttpServer()));
assert(testGlobalsHttpRequest() === []);
assert(testGlobalsHttpFiles() === []);
