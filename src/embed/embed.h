#include "zend.h"
#include "sapi/embed/php_embed.h"

void* ext_php_rs_embed_callback(int argc, char** argv, void* (*callback)(void *), void *ctx);

void ext_php_rs_sapi_startup();
