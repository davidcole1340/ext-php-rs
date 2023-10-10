// PHP for Windows uses the `vectorcall` calling convention on some functions.
// This is guarded by the `ZEND_FASTCALL` macro, which is set to `__vectorcall`
// on Windows and nothing on other systems.
//
// However, `ZEND_FASTCALL` is only set when compiling with MSVC and the PHP
// source code checks for the __clang__ macro and will not define `__vectorcall`
// if it is set (even on Windows). This is a problem as Bindgen uses libclang to
// generate bindings. To work around this, we include the header file containing
// the `ZEND_FASTCALL` macro but not before undefining `__clang__` to pretend we
// are compiling on MSVC.
#if defined(_MSC_VER) && defined(__clang__)
#undef __clang__
#include "zend_portability.h"
#define __clang__
#endif

#include "php.h"

#include "ext/standard/info.h"
#include "zend_exceptions.h"
#include "zend_inheritance.h"
#include "zend_interfaces.h"
#include "zend_ini.h"

zend_string *ext_php_rs_zend_string_init(const char *str, size_t len, bool persistent);
void ext_php_rs_zend_string_release(zend_string *zs);
bool ext_php_rs_is_known_valid_utf8(const zend_string *zs);
void ext_php_rs_set_known_valid_utf8(zend_string *zs);

const char *ext_php_rs_php_build_id();
void *ext_php_rs_zend_object_alloc(size_t obj_size, zend_class_entry *ce);
void ext_php_rs_zend_object_release(zend_object *obj);
zend_executor_globals *ext_php_rs_executor_globals();