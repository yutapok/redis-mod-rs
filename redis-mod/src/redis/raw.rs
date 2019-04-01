// Allow dead code in here in case I want to publish it as a crate at some
// point.
#![allow(dead_code)]

extern crate libc;

use libc::{c_int, c_long, c_longlong, c_double, size_t};

// Rust can't link against C macros (#define) so we just redefine them here.
// There's a ~0 chance that any of these will ever change so it's pretty safe.
pub const REDISMODULE_APIVER_1: c_int = 1;

bitflags! {
    pub struct KeyMode: c_int {
        const READ = 1;
        const WRITE = (1 << 1);
    }
}

bitflags! {
    pub struct HashFlags: c_int {
        const NONE = 0;
        const NX = (1 << 0);
        const XX = (1 << 1);
        const CFIELDS = (1 << 2);
        const EXISTS = (1 << 3);
    }
}

#[derive(Debug, PartialEq)]
pub enum ReplyType{
    Unknown = -1,
    String = 0,
    Error = 1,
    Integer = 2,
    Array = 3,
    Nil = 4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Ok = 0,
    Err = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyType{
    Empty = 0,
    String = 1,
    List = 2,
    Hash = 3,
    Set = 4,
    Zset = 5,
    Module = 6,
}


#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleCallReply;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleCtx;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleKey;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RedisModuleString;

pub type RedisModuleCmdFunc = extern "C" fn(
     ctx: *mut RedisModuleCtx,
     argv: *mut *mut RedisModuleString,
     argc: c_int,
 ) -> Status;


//C function wrapper for Rust.
pub fn init(
    ctx: *mut RedisModuleCtx,
    modulename: *const u8,
    module_version: c_int,
    api_version: c_int,
) -> Status {
    unsafe{ Export_RedisModule_Init(ctx, modulename, module_version, api_version) }
}

pub fn call_reply_type(reply: *mut RedisModuleCallReply) -> ReplyType {
    unsafe { RedisModule_CallReplyType(reply) }
}

pub fn free_call_reply(reply: *mut RedisModuleCallReply) {
    unsafe { RedisModule_FreeCallReply(reply) }
}

pub fn call_reply_integer(reply: *mut RedisModuleCallReply) -> c_longlong {
    unsafe { RedisModule_CallReplyInteger(reply) }
}

pub fn call_reply_string_ptr(
    str: *mut RedisModuleCallReply,
    len: *mut size_t,
) -> *const u8 {
    unsafe { RedisModule_CallReplyStringPtr(str, len) }
}



pub fn create_command(
    ctx: *mut RedisModuleCtx,
    name: *const u8,
    cmdfunc: Option<RedisModuleCmdFunc>,
    strflags: *const u8,
    firstkey: c_int,
    lastkey: c_int,
    keystep: c_int,
) -> Status {
    unsafe {
        RedisModule_CreateCommand(
            ctx,
            name,
            cmdfunc,
            strflags,
            firstkey,
            lastkey,
            keystep
        )
    }
}

pub fn open_key(
    ctx: *mut RedisModuleCtx,
    name: *mut RedisModuleString,
    mode: KeyMode
) -> *mut RedisModuleKey {
    unsafe { RedisModule_OpenKey(ctx, name, mode) }
}

pub fn close_key(kp: *mut RedisModuleKey) {
    unsafe { RedisModule_CloseKey(kp) }
}

pub fn key_type(kp: *mut RedisModuleKey) -> KeyType {
    unsafe { RedisModule_KeyType(kp) }
}

pub fn string_set(
    key: *mut RedisModuleKey,
    val: *mut RedisModuleString
) -> Status {
    unsafe{ RedisModule_StringSet(key, val) }
}

pub fn string_dma(
    key: *mut RedisModuleKey,
    len: *mut size_t,
    mode: KeyMode,
) -> *const u8 {
    unsafe { RedisModule_StringDMA(key, len, mode) }
}

pub fn delete_key(key: *mut RedisModuleKey) -> Status {
    unsafe { RedisModule_DeleteKey(key) }
}

pub fn reply_with_array(
    ctx: *mut RedisModuleCtx,
    len: c_long
) -> Status {
    unsafe { RedisModule_ReplyWithArray(ctx, len) }
}

pub fn reply_with_error(
    ctx: *mut RedisModuleCtx,
    err: *const u8
) {
    unsafe { RedisModule_ReplyWithError(ctx, err) }
}

pub fn reply_with_long_long(
    ctx: *mut RedisModuleCtx,
    ll: c_longlong
) -> Status {
    unsafe { RedisModule_ReplyWithLongLong(ctx, ll) }
}

pub fn reply_with_string(
    ctx: *mut RedisModuleCtx,
    str: *mut RedisModuleString,
) -> Status {
    unsafe { RedisModule_ReplyWithString(ctx, str) }
}

pub fn reply_with_simple_string(
    ctx: *mut RedisModuleCtx,
    msg: *const u8
) {
    unsafe { RedisModule_ReplyWithSimpleString(ctx, msg) }
}

pub fn reply_with_null(
    ctx: *mut RedisModuleCtx
){ unsafe { RedisModule_ReplyWithNull(ctx) } }


pub fn free_string(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString) {
    unsafe { RedisModule_FreeString(ctx, str) }
}

pub fn log(ctx: *mut RedisModuleCtx, level: *const u8, fmt: *const u8) {
    unsafe { RedisModule_Log(ctx, level, fmt) }
}

pub fn create_string(
    ctx: *mut RedisModuleCtx,
    ptr: *const u8,
    len: size_t,
) -> *mut RedisModuleString {
    unsafe { RedisModule_CreateString(ctx, ptr, len) }
}

pub fn set_expire(key: *mut RedisModuleKey, expire: c_longlong) -> Status {
    unsafe { RedisModule_SetExpire(key, expire) }
}

pub fn string_ptr_len(str: *mut RedisModuleString, len: *mut size_t) -> *const u8 {
    unsafe { RedisModule_StringPtrLen(str, len) }
}

pub fn list_push(key: *mut RedisModuleKey, place: c_int, ele: *mut RedisModuleString) -> Status {
    unsafe { RedisModule_ListPush(key, place, ele) }
}

pub fn list_pop(key: *mut RedisModuleKey, place: c_int) -> *mut RedisModuleString {
    unsafe { RedisModule_ListPop(key, place) }
}

pub fn callable2(
    ctx: *mut RedisModuleCtx,
    cmdname: *const i8,
    key: *const i8,
    arg0: *const i8,
) -> c_longlong {
    unsafe{ RedisModuleCallable2_ReplyInteger(ctx, cmdname, key, arg0) }
}


//extern function of C
#[allow(improper_ctypes)]
#[link(name = "redis_mod_callable", kind = "static")]
extern "C" {
    pub fn RedisModuleCallable2_ReplyInteger(
        ctx: *mut RedisModuleCtx,
        cmdname: *const i8,
        key: *const i8,
        arg0: *const i8,
    ) -> c_longlong;
}


//extern function of C
#[allow(improper_ctypes)]
#[link(name = "redismodule", kind = "static")]
extern "C" {
    pub fn Export_RedisModule_Init(
        ctx: *mut RedisModuleCtx,
        modulename: *const u8,
        module_version: c_int,
        api_version: c_int,
    ) -> Status;

    static RedisModule_Call: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        args: *const *mut RedisModuleString,
    ) -> *mut RedisModuleCallReply;

    static RedisModule_CallReplyType:
        extern "C" fn(reply: *mut RedisModuleCallReply) -> ReplyType;

    static RedisModule_FreeCallReply: extern "C" fn(reply: *mut RedisModuleCallReply);

    static RedisModule_CallReplyInteger:
        extern "C" fn(reply: *mut RedisModuleCallReply) -> c_longlong;

    static RedisModule_CallReplyStringPtr:
        extern "C" fn(str: *mut RedisModuleCallReply, len: *mut size_t) -> *const u8;

    static RedisModule_CreateCommand:
        extern "C" fn(
            ctx: *mut RedisModuleCtx,
            name: *const u8,
            cmdfunc: Option<RedisModuleCmdFunc>,
            strflags: *const u8,
            firstkey: c_int,
            lastkey: c_int,
            keystep: c_int,
        ) -> Status;

    static RedisModule_OpenKey:
        extern "C" fn(
            ctx: *mut RedisModuleCtx,
            name: *mut RedisModuleString,
            mode: KeyMode
        ) -> *mut RedisModuleKey;

    static RedisModule_CloseKey:
        extern "C" fn(kp: *mut RedisModuleKey);

    static RedisModule_KeyType:
        extern "C" fn(kp: *mut RedisModuleKey) -> KeyType;

    static RedisModule_StringSet:
        extern "C" fn(
            key: *mut RedisModuleKey,
            val: *mut RedisModuleString
        ) -> Status;

    static RedisModule_StringDMA:
        extern "C" fn(
            key: *mut RedisModuleKey,
            len: *mut size_t,
            mode: KeyMode
        ) -> *const u8;

    static RedisModule_DeleteKey:
        extern "C" fn(key: *mut RedisModuleKey) -> Status;

    static RedisModule_ReplyWithArray:
        extern "C" fn(
            ctx: *mut RedisModuleCtx,
            len: c_long
        ) -> Status;

    static RedisModule_ReplyWithError:
        extern "C" fn(
            ctx: *mut RedisModuleCtx,
            err: *const u8
        );

    static RedisModule_ReplyWithLongLong:
        extern "C" fn(
            ctx: *mut RedisModuleCtx,
            ll: c_longlong
        ) -> Status;

    static RedisModule_ReplyWithString:
        extern "C" fn(
            ctx: *mut RedisModuleCtx,
            str: *mut RedisModuleString
    ) -> Status;

    static RedisModule_ReplyWithSimpleString:
        extern "C" fn(
            ctx: *mut RedisModuleCtx,
            msg: *const u8
    );

    static RedisModule_ReplyWithNull:
        extern "C" fn(
            ctx: *mut RedisModuleCtx
    );

    static RedisModule_CreateString:
        extern "C" fn(ctx: *mut RedisModuleCtx, ptr: *const u8, len: size_t)
            -> *mut RedisModuleString;

    static RedisModule_FreeString:
        extern "C" fn(ctx: *mut RedisModuleCtx, str: *mut RedisModuleString);

    static RedisModule_Log:
        extern "C" fn(ctx: *mut RedisModuleCtx, level: *const u8, fmt: *const u8);

    static RedisModule_SetExpire:
        extern "C" fn(key: *mut RedisModuleKey, expire: c_longlong) -> Status;

    static RedisModule_StringPtrLen:
        extern "C" fn(str: *mut RedisModuleString, len: *mut size_t) -> *const u8;

    static RedisModule_ListPush:
        extern "C" fn(key: *mut RedisModuleKey, place: c_int, ele: *mut RedisModuleString) -> Status;

    static RedisModule_ListPop:
        extern "C" fn(key: *mut RedisModuleKey, place: c_int) -> *mut RedisModuleString;


}

pub mod call1 {
    use crate::redis::raw;
    use std::os::raw::c_char;

    pub fn call(
        ctx: *mut raw::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut raw::RedisModuleString,
    ) -> *mut raw::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call: extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut raw::RedisModuleString,
        )
            -> *mut raw::RedisModuleCallReply;
    }
}

pub mod call2 {
    use crate::redis::raw;

    pub fn call(
        ctx: *mut raw::RedisModuleCtx,
        cmdname: *const i8,
        fmt: *const i8,
        arg0: *const i8,
        arg1: *const i8,
    ) -> *mut raw::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call: extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            cmdname: *const i8,
            fmt: *const i8,
            arg0: *const i8,
            arg1: *const i8,
        )
            -> *mut raw::RedisModuleCallReply;
    }
}

pub mod call3 {
    use crate::redis::raw;

    pub fn call(
        ctx: *mut raw::RedisModuleCtx,
        cmdname: *const u8,
        fmt: *const u8,
        arg0: *mut raw::RedisModuleString,
        arg1: *mut raw::RedisModuleString,
        arg2: *mut raw::RedisModuleString,
    ) -> *mut raw::RedisModuleCallReply {
        unsafe { RedisModule_Call(ctx, cmdname, fmt, arg0, arg1, arg2) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_Call: extern "C" fn(
            ctx: *mut raw::RedisModuleCtx,
            cmdname: *const u8,
            fmt: *const u8,
            arg0: *mut raw::RedisModuleString,
            arg1: *mut raw::RedisModuleString,
            arg2: *mut raw::RedisModuleString,
        )
            -> *mut raw::RedisModuleCallReply;
    }
}

pub mod hash {
    use crate::redis::raw;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum HashFlags {
        NONE,
        NX,
        XX,
        CFIELDS,
        EXISTS
    }

    pub fn hash_get(
        key: *mut raw::RedisModuleKey,
        field: *mut raw::RedisModuleString,
        oldval: &*mut raw::RedisModuleString
    ) -> raw::Status {
        let flags = HashFlags::NONE;
        unsafe{ RedisModule_HashGet(key, flags, field, oldval, None) }
    }

    pub fn hash_set(
        key: *mut raw::RedisModuleKey,
        field: *mut raw::RedisModuleString,
        val: *mut raw::RedisModuleString,
    ) -> raw::Status {
        let flags = HashFlags::NONE;
        unsafe{ RedisModule_HashSet(key, flags, field, val, None) }
    }

    #[allow(improper_ctypes)]
    extern "C" {
        pub static RedisModule_HashGet: extern "C" fn(
            key: *mut raw::RedisModuleKey,
            flags: HashFlags,
            field: *mut raw::RedisModuleString,
            oldval: &*mut raw::RedisModuleString,
            nullp: Option<extern "C" fn(i32)>
        ) -> raw::Status;

        pub static RedisModule_HashSet: extern "C" fn(
            key: *mut raw::RedisModuleKey,
            flags: HashFlags,
            field: *mut raw::RedisModuleString,
            val: *mut raw::RedisModuleString,
            nullp: Option<extern "C" fn(i32)>
        ) -> raw::Status;
    }
}


