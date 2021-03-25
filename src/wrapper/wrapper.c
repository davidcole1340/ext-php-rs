#include "wrapper.h"

zend_string *php_rs_zend_string_init(const char *str, size_t len, bool persistent)
{
    return zend_string_init(str, len, persistent);
}

void php_rs_zend_string_release(zend_string *zs)
{
    zend_string_release(zs);
}

const char *php_rs_php_build_id()
{
    return ZEND_MODULE_BUILD_ID;
}

void *php_rs_zend_object_alloc(size_t obj_size, zend_class_entry *ce)
{
    return zend_object_alloc(obj_size, ce);
}

void php_rs_zend_object_std_init(zend_object *object, zend_class_entry *ce)
{
    zend_object_std_init(object, ce);
}