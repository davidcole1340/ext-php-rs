#include "embed.h"

// We actually use the PHP embed API to run PHP code in test
// At some point we might want to use our own SAPI to do that
void* ext_php_rs_embed_callback(int argc, char** argv, void* (*callback)(void *), void *ctx) {
  void *result;

  PHP_EMBED_START_BLOCK(argc, argv)

  result = callback(ctx);

  PHP_EMBED_END_BLOCK()

  return result;
}
