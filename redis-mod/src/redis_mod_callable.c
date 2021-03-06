#include <string.h>
#include "redismodule.h"

//Here is to handle method of RedisModule_Call, RedisModule_HashSet.. for unsupported variable argument on Rust function.
//
//Notice:
//    Method which is similar to this is already prepared, but segmentation fault occur at C ffi
//    while calling RedisModule_Call, RedisModule_HashSet... function extern on Rust side.
//    This reason probably is when parsing variable arguments transferd by Rust inside RedisModule_Call, RedisModule_HashSet...
//    and then something error with unexpecetd incoming.

//RedisModuleCallReply *RedisModule_Callable1(RedisModuleCtx *ctx, const char *cmdname, const char *key, const char *arg0) {
//    return RedisModule_Call(ctx, cmdname, "cc", key, arg0);
//}

static RedisModuleCallReply *RedisModule_Callable2(RedisModuleCtx *ctx, const char *cmdname, const char *key, const char *arg0) {
    return RedisModule_Call(ctx, cmdname, "cc", key, arg0);
}

RedisModuleCallReply *RedisModule_Call1(RedisModuleCtx *ctx, const char *cmdname, const char *key) {
    return RedisModule_Call(ctx, cmdname, "c", key);
}

RedisModuleCallReply *RedisModule_Call2(RedisModuleCtx *ctx, const char *cmdname, const char *key, const char *arg0) {
    return RedisModule_Call(ctx, cmdname, "cc", key, arg0);
}

RedisModuleCallReply *RedisModule_Call3(RedisModuleCtx *ctx, const char *cmdname, const char *key, const char *arg0, const char *arg1) {
    return RedisModule_Call(ctx, cmdname, "ccc", key, arg0, arg1);
}

RedisModuleCallReply *RedisModule_CallKeys(RedisModuleCtx *ctx, const char *arg0){
    return RedisModule_Call(ctx, "keys", "c", arg0);
}

long long RedisModuleCallable2_ReplyInteger(RedisModuleCtx *ctx, const char *cmdname, const char *key, const char *arg0){
    RedisModuleCallReply *resp = RedisModule_Callable2(ctx, cmdname, key, arg0);
    if (RedisModule_CallReplyType(resp) != REDISMODULE_REPLY_INTEGER){
        RedisModule_FreeCallReply(resp); resp = NULL;
        return -1;
    }

    long long reply_int = RedisModule_CallReplyInteger(resp);
    RedisModule_FreeCallReply(resp); resp = NULL;

    return reply_int;
}



RedisModuleString *RedisModuleHash_Get(RedisModuleKey *key, RedisModuleString *field){
    RedisModuleString *oldval;
    RedisModule_HashGet(key, REDISMODULE_HASH_NONE, field, &oldval, NULL);
    return oldval;
}

int RedisModuleHash_Set(RedisModuleKey *key, RedisModuleString *field, RedisModuleString *val){
    RedisModule_HashSet(key, REDISMODULE_HASH_NONE, field, val, NULL);
    return REDISMODULE_OK;
}


