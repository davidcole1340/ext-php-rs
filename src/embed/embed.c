#include "embed.h"

// We actually use the PHP embed API to run PHP code in test
// At some point we might want to use our own SAPI to do that
void* ext_php_rs_embed_callback(int argc, char** argv, void* (*callback)(void *), void *ctx) {
  void *result = NULL;

  PHP_EMBED_START_BLOCK(argc, argv)

  result = callback(ctx);

  PHP_EMBED_END_BLOCK()

  return result;
}

void ext_php_rs_sapi_startup() {
  #if defined(SIGPIPE) && defined(SIG_IGN)
    signal(SIGPIPE, SIG_IGN);
  #endif

  #ifdef ZTS
    php_tsrm_startup();
    #ifdef PHP_WIN32
      ZEND_TSRMLS_CACHE_UPDATE();
    #endif
  #endif

  zend_signal_startup();
}

void ext_php_rs_php_error(int type, const char *format, ...) {
  va_list args;
  va_start(args, format);
  php_error(type, format, args);
  vprintf(format, args);
  va_end(args);
}
