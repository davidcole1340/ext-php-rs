#include "wrapper.h"

zend_string *ext_php_rs_zend_string_init(const char *str, size_t len, bool persistent)
{
    return zend_string_init(str, len, persistent);
}

void ext_php_rs_zend_string_release(zend_string *zs)
{
    zend_string_release(zs);
}

const char *ext_php_rs_php_build_id()
{
    return ZEND_MODULE_BUILD_ID;
}

void *ext_php_rs_zend_object_alloc(size_t obj_size, zend_class_entry *ce)
{
    return zend_object_alloc(obj_size, ce);
}
