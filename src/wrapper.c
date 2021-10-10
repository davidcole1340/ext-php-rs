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

void ext_php_rs_zend_object_release(zend_object *obj)
{
    zend_object_release(obj);
}

zend_executor_globals *ext_php_rs_executor_globals()
{
#ifdef ZTS
# ifdef ZEND_ENABLE_STATIC_TSRMLS_CACHE
    return TSRMG_FAST_BULK_STATIC(executor_globals_offset, zend_executor_globals);
# else
    return TSRMG_FAST_BULK(executor_globals_offset, zend_executor_globals *);
# endif
#else
    return &executor_globals;
#endif
}
