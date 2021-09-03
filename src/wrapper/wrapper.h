#include "php.h"
#include "ext/standard/info.h"
#include "zend_exceptions.h"
#include "zend_inheritance.h"

zend_string *ext_php_rs_zend_string_init(const char *str, size_t len, bool persistent);
void ext_php_rs_zend_string_release(zend_string *zs);
const char *ext_php_rs_php_build_id();
void *ext_php_rs_zend_object_alloc(size_t obj_size, zend_class_entry *ce);
void ext_php_rs_zend_object_release(zend_object *obj);
zend_executor_globals *ext_php_rs_executor_globals();