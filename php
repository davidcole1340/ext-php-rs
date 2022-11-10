#!/bin/bash

cargo build --example hello_world
php -dextension=target/debug/examples/libhello_world.dylib $@
