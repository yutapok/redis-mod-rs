// `raw` should not be public in the long run. Build an abstraction interface
// instead.
//
// We have to disable a couple Clippy checks here because we'll otherwise have
// warnings thrown from within macros provided by the `bigflags` package.
#[cfg_attr(feature = "cargo-clippy",
           allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod raw;

use crate::error::RModError;
use libc::{c_int, c_long, c_longlong, size_t};
use std::ptr;
use std::string;
use time;
use std::ffi::CString;
use std::alloc::{GlobalAlloc, Layout};



/// `LogLevel` is a level of logging to be specified with a Redis log directive.
#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Debug,
    Notice,
    Verbose,
    Warning,
}

/// Reply represents the various types of a replies that we can receive after
/// executing a Redis command.
#[derive(Debug)]
pub enum Reply {
    Array,
    Error,
    Integer(i64),
    Nil,
    String(String),
    Unknown,
}

pub trait Command {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str;

    // Run the command.
    fn run(&self, r: Redis, args: &[&str]) -> Result<(), RModError>;

    // Should return any flags to be registered with the name as a string
    // separated list. See the latest Redis module API documentation for a complete
    // list of the ones that are available. 

    /// flags to be registered are ...
    ///     "write": The command may modify the data set (it may also read from it).
    ///     "readonly": The command returns data from keys but never writes.
    ///     "admin": The command is an administrative command (may change replication or perform similar tasks).
    ///     "deny-oom": The command may use additional memory and should be denied during out of memory conditions.
    ///     "deny-script": Don't allow this command in Lua scripts.
    ///     "allow-loading": Allow this command while the server is loading data. 
    ///     "pubsub": The command publishes things on Pub/Sub channels.
    ///     "random": The command may have different outputs even starting from the same input arguments and key values.
    ///     "allow-stale": The command is allowed to run on slaves that don't serve stale data. Don't use if you don't know what this means.
    ///     "no-monitor": Don't propagate the command on monitor. Use this if the command has sensible data among the arguments.
    ///     "fast": The command time complexity is not greater than O(log(N)) where N is the size of the collection or anything else representing the normal scalability issue with the command.
    ///     "getkeys-api": The command implements the interface to return the arguments that are keys. Used when start/stop/step is not enough because of the command syntax.
    ///     "no-cluster": The command should not register in Redis Cluster since is not designed to work with it. 
    fn str_flags(&self) -> &'static str;  
}

impl dyn Command {
    /// Provides a basic wrapper for a command's implementation that parses
    /// arguments to Rust data types and handles the OK/ERR reply back to Redis.    
    pub fn harness(
        command: &dyn Command,
        ctx: *mut raw::RedisModuleCtx,
        argv: *mut *mut raw::RedisModuleString,
        argc: c_int,
    ) -> raw::Status {
        let r = Redis { ctx };
        let args = parse_args(argv, argc).unwrap();
        let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match command.run(r, str_args.as_slice()) {
            Ok(_) => raw::Status::Ok,
            Err(e) => {
                raw::reply_with_error(
                    ctx,
                    format!("RMod error: {}\0", e.to_string()).as_ptr(),
                );
                raw::Status::Err
            }
        }
    }
}

/// Redis is a structure that's designed to give us a high-level interface to
/// the Redis module API by abstracting away the raw C FFI calls.
pub struct Redis {
    ctx: *mut raw::RedisModuleCtx,
}

impl Redis {
        pub fn call2_reply_int(&self, cmdname: &str, args0: &str, args1: &str) -> c_longlong {
            let cmdname = CString::new(cmdname).expect("CString::new(cmdname) failed");
            let key = CString::new(args0).expect("CString::new(key) failed");
            let arg0 = CString::new(args1).expect("CString::new(arg0) failed");
            raw::callable2_reply_int(self.ctx, cmdname.as_ptr(), key.as_ptr(), arg0.as_ptr())
        }

        pub fn call1_reply_integer(&self, cmdname: &str, arg0 : &str) -> Result<i64, RModError> {
            let cmdname = CString::new(cmdname).expect("CString::new(cmdname) failed");
            let arg = CString::new(arg0).expect("CString::new(arg) failed");
            let reply = RedisCallReply::create(raw::call1_reply(self.ctx, cmdname.as_ptr(),arg.as_ptr()));
            reply.to_integer()
        }

        pub fn call2_reply_integer(&self, cmdname: &str, arg0 : &str, arg1 : &str) -> Result<i64, RModError> {
            let cmdname = CString::new(cmdname).expect("CString::new(cmdname) failed");
            let arg0 = CString::new(arg0).expect("CString::new(arg) failed");
            let arg1 = CString::new(arg1).expect("CString::new(arg) failed");
            let reply = RedisCallReply::create(raw::call2_reply(self.ctx, cmdname.as_ptr(),arg0.as_ptr(), arg1.as_ptr()));
            reply.to_integer()
        }

        pub fn call3_reply_integer(&self, cmdname: &str, arg0 : &str, arg1 : &str, arg2 : &str) -> Result<i64, RModError> {
            let cmdname = CString::new(cmdname).expect("CString::new(cmdname) failed");
            let arg0 = CString::new(arg0).expect("CString::new(arg) failed");
            let arg1 = CString::new(arg1).expect("CString::new(arg) failed");
            let arg2 = CString::new(arg2).expect("CString::new(arg) failed");
            let reply = RedisCallReply::create(raw::call3_reply(self.ctx, cmdname.as_ptr(),arg0.as_ptr(), arg1.as_ptr(), arg2.as_ptr()));
            reply.to_integer()
        }

        pub fn call1_reply_string(&self, cmdname: &str, arg0 : &str) -> Result<String, RModError> {
            let cmdname = CString::new(cmdname).expect("CString::new(cmdname) failed");
            let arg = CString::new(arg0).expect("CString::new(arg) failed");
            let reply = RedisCallReply::create(raw::call1_reply(self.ctx, cmdname.as_ptr(),arg.as_ptr()));
            reply.to_string()
        }

        pub fn call2_reply_string(&self, cmdname: &str, arg0 : &str, arg1 : &str) -> Result<String, RModError> {
            let cmdname = CString::new(cmdname).expect("CString::new(cmdname) failed");
            let arg0 = CString::new(arg0).expect("CString::new(arg) failed");
            let arg1 = CString::new(arg1).expect("CString::new(arg) failed");
            let reply = RedisCallReply::create(raw::call2_reply(self.ctx, cmdname.as_ptr(), arg0.as_ptr(), arg1.as_ptr()));
            reply.to_string()
        }

        pub fn call3_reply_string(&self, cmdname: &str, arg0 : &str, arg1 : &str, arg2 : &str) -> Result<String, RModError> {
            let cmdname = CString::new(cmdname).expect("CString::new(cmdname) failed");
            let arg0 = CString::new(arg0).expect("CString::new(arg) failed");
            let arg1 = CString::new(arg1).expect("CString::new(arg) failed");
            let arg2 = CString::new(arg2).expect("CString::new(arg) failed");
            let reply = RedisCallReply::create(raw::call3_reply(self.ctx, cmdname.as_ptr(),arg0.as_ptr(), arg1.as_ptr(), arg2.as_ptr()));
            reply.to_string()
        }


        pub fn call_keys(&self, arg: &str) -> Result<Vec<String>, RModError> {
            let arg = CString::new(arg).expect("CString::new(arg) failed");
            let cmd = CString::new("keys").expect("CString::new(keys) failed");
            let reply = RedisCallReply::create(raw::call1_reply(self.ctx, cmd.as_ptr(), arg.as_ptr()));
            let size = reply.check_length() as u64;
            let mut vec_keys: Vec<String> = Vec::with_capacity(size as usize);
            for idx in 0..size {
                let ele_str = match reply.reply_array_element(idx as usize){
                    Ok(reply2) => reply2.to_string(),
                    Err(_) => return Err(error!("Failed to take element from reply array"))
                };
                match ele_str {
                    Ok(s) => vec_keys.insert(idx as usize, s),
                    Err(msg) =>  vec_keys.insert(idx as usize, msg.to_string())
                }
            }

            Ok(vec_keys)
        }


    /// Coerces a Redis string as an integer.size_t///
    /// Redis is pretty dumb about data types. It nominally supports strings
    /// versus integers, but an integer set in the store will continue to look
    /// like a string (i.e. "1234") until some other operation like INCR forces
    /// its coercion.
    ///
    /// This method coerces a Redis string that looks like an integer into an
    /// integer response. All other types of replies are passed through
    /// unmodified.
    pub fn coerce_integer(
        &self,
        reply_res: Result<Reply, RModError>,
    ) -> Result<Reply, RModError> {
        match reply_res {
            Ok(Reply::String(s)) => match s.parse::<i64>() {
                Ok(n) => Ok(Reply::Integer(n)),
                _ => Ok(Reply::String(s)),
            },
            _ => reply_res,
        }
    }

    pub fn create_string(&self, s: &str) -> RedisString {
        RedisString::create(self.ctx, s)
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        raw::log(
            self.ctx,
            format!("{:?}\0", level).to_lowercase().as_ptr(),
            format!("{}\0", message).as_ptr(),
        );
    }

    pub fn log_debug(&self, message: &str) {
        // Note that we log our debug messages as notice level in Redis. This
        // is so that they'll show up with default configuration. Our debug
        // logging will get compiled out in a release build so this won't
        // result in undue noise in production.
        self.log(LogLevel::Notice, message);
    }

    /// Opens a Redis key for read access.
    pub fn open_key(&self, key: &str) -> RedisKey {
        RedisKey::open(self.ctx, key)
    }

    /// Opens a Redis key for read and write access.
    pub fn open_key_writable(&self, key: &str) -> RedisKeyWritable {
        RedisKeyWritable::open(self.ctx, key)
    }

    /// Tells Redis that we're about to reply with an (Redis) array.
    /// Used by invoking once with the expected length and then calling any
    /// combination of the other reply_* methods exactly that number of times.
    pub fn reply_array(&self, len: i64) -> Result<(), RModError> {
        handle_status(
            raw::reply_with_array(self.ctx, len as c_long),
            "Could not reply with long",
        )
    }

    pub fn reply_integer(&self, integer: i64) -> Result<(), RModError> {
        handle_status(
            raw::reply_with_long_long(self.ctx, integer as c_longlong),
            "Could not reply with longlong",
        )
    }

    pub fn reply_string(&self, message: &str) -> Result<(), RModError> {
        let redis_str = self.create_string(message);
        handle_status(
            raw::reply_with_string(self.ctx, redis_str.str_inner),
            "Could not reply with string",
        )
    }

    pub fn reply_with_simple_string(&self, message: &str) {
        raw::reply_with_simple_string(
            self.ctx,
            format!("{}\0",message).as_ptr()
        )
    }

    pub fn reply_ok(&self){
        raw::reply_with_simple_string(
            self.ctx,
            format!("OK\0").as_ptr()
        )
    }

    pub fn reply_null(&self) {
        raw::reply_with_null(self.ctx);
    }

    pub fn replicate_verbatim(&self) {
        raw::replicate_verbatim(self.ctx);
    }

}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyMode {
    Read,
    ReadWrite,
}

/// `RedisKey` is an abstraction over a Redis key that allows readonly
/// operations.
///
/// Its primary function is to ensure the proper deallocation of resources when
/// it goes out of scope. Redis normally requires that keys be managed manually
/// by explicitly freeing them when you're done. This can be a risky prospect,
/// especially with mechanics like Rust's `?` operator, so we ensure fault-free
/// operation through the use of the Drop trait.
#[derive(Debug)]
pub struct RedisKey {
    ctx:       *mut raw::RedisModuleCtx,
    key_inner: *mut raw::RedisModuleKey,
    key_str:   RedisString,
}

impl RedisKey {
    fn open(ctx: *mut raw::RedisModuleCtx, key: &str) -> RedisKey {
        let key_str = RedisString::create(ctx, key);
        let key_inner = raw::open_key(ctx, key_str.str_inner, to_raw_mode(KeyMode::Read));
        RedisKey {
            ctx,
            key_inner,
            key_str,
        }
    }

    /// Detects whether the key pointer given to us by Redis is null.
    pub fn is_null(&self) -> bool {
        let null_key: *mut raw::RedisModuleKey = ptr::null_mut();
        self.key_inner == null_key
    }

    pub fn read(&self) -> Result<Option<String>, RModError> {
        let val = if self.is_null() {
            None
        } else {
            Some(read_key(self.key_inner)?)
        };
        Ok(val)
    }

}


impl Drop for RedisKey {
// Frees resources appropriately as a RedisKey goes out of scope.
    fn drop(&mut self) {
        raw::close_key(self.key_inner);
    }
}

/// `RedisKeyWritable` is an abstraction over a Redis key that allows read and
/// write operations.
pub struct RedisKeyWritable {
    ctx:       *mut raw::RedisModuleCtx,
    key_inner: *mut raw::RedisModuleKey,

    // The Redis string
    //
    // This field is needed on the struct so that its Drop implementation gets
    // called when it goes out of scope.
    #[allow(dead_code)]
    key_str: RedisString,
}


impl RedisKeyWritable {
    fn open(ctx: *mut raw::RedisModuleCtx, key: &str) -> RedisKeyWritable {
        let key_str = RedisString::create(ctx, key);
        let key_inner =
            raw::open_key(ctx, key_str.str_inner, to_raw_mode(KeyMode::ReadWrite));
        RedisKeyWritable {
            ctx,
            key_inner,
            key_str,
        }
    }

    /// Detects whether the value stored in a Redis key is empty.
    ///
    /// Note that an empty key can be reliably detected by looking for a null
    /// as you open the key in read mode, but when asking for write Redis
    /// returns a non-null pointer to allow us to write to even an empty key,
    /// so we have to check the key's value instead.
    pub fn is_empty(&self) -> Result<bool, RModError> {
        match self.read()? {
            Some(s) => match s.as_str() {
                "" => Ok(true),
                _ => Ok(false),
            },
            _ => Ok(false),
        }
    }

    pub fn read(&self) -> Result<Option<String>, RModError> {
        Ok(Some(read_key(self.key_inner)?))
    }

    pub fn set_expire(&self, expire: time::Duration) -> Result<(), RModError> {
        match raw::set_expire(self.key_inner, expire.num_milliseconds()) {
            raw::Status::Ok => Ok(()),

            // Error may occur if the key wasn't open for writing or is an
            // empty key.
            raw::Status::Err => Err(error!("Error while setting key expire")),
        }
    }

    pub fn write(&self, val: &str) -> Result<(), RModError> {
        let val_str = RedisString::create(self.ctx, val);
        match raw::string_set(self.key_inner, val_str.str_inner) {
            raw::Status::Ok => Ok(()),
            raw::Status::Err => Err(error!("Error while setting key")),
        }
    }

    pub fn erace(&self) -> Result<(), RModError> {
        match raw::delete_key(self.key_inner){
            raw::Status::Ok => Ok(()),
            raw::Status::Err => Err(error!("Error while eracing key"))
        }
    }

    pub fn rpush(&self, ele: &str) -> Result<(), RModError> {
        let ele_str = RedisString::create(self.ctx, ele);
        let place: c_int = -1;
        match raw::list_push(self.key_inner,place,ele_str.str_inner) {
            raw::Status::Ok => Ok(()),
            raw::Status::Err => Err(error!("Error while rpush to key, tried to the wrong type"))
        }
    }

    pub fn lpush(&self, ele: &str) -> Result<(), RModError> {
        let ele_str = RedisString::create(self.ctx, ele);
        let place: c_int = 0;
        match raw::list_push(self.key_inner,place,ele_str.str_inner) {
            raw::Status::Ok => Ok(()),
            raw::Status::Err => Err(error!("Error while lpush to key, tried to the wrong type"))
        }
    }

    pub fn rpop(&self) -> Result<Option<String>, RModError> {
        match raw::key_type(self.key_inner) {
            raw::KeyType::Empty => return Ok(None),
            raw::KeyType::List  => (),
            _ => return Err(error!("Error while lpop to key, not List structure")),
        }
        let place: c_int = -1;
        let redis_str = raw::list_pop(self.key_inner,place);
        match manifest_redis_string(redis_str){
            Ok(re_str) => Ok(Some(re_str)),
            Err(_) => Err(error!("Error while rpop to key, tried to the wrong type"))
        }
    }

    pub fn lpop(&self) -> Result<Option<String>, RModError> {
        match raw::key_type(self.key_inner) {
            raw::KeyType::Empty => return Ok(None),
            raw::KeyType::List  => (),
            _ => return Err(error!("Error while lpop to key, not List structure")),
        }

        let place: c_int = 0;
        let redis_str = raw::list_pop(self.key_inner,place);
        match manifest_redis_string(redis_str){
            Ok(re_str) => Ok(Some(re_str)),
            Err(_) => Err(error!("Error while lpop to key, tried to the wrong type"))
        }
    }


    pub fn rm_hget(&self, field: &str) -> Option<String> {
        let fld_str = RedisString::create(self.ctx, field);
        let val_str = raw::rm_hash_get(self.key_inner, fld_str.str_inner);
        match manifest_redis_string(val_str){
            Ok(re_str) => Some(re_str),
            Err(_) => None
        }
    }

    pub fn rm_hset(&self, field: &str, val: &str) -> Result<(), RModError> {
        let fld_str = RedisString::create(self.ctx, field);
        let val_str = RedisString::create(self.ctx, val);
        match raw::rm_hash_set(
            self.key_inner,
            fld_str.str_inner,
            val_str.str_inner
        ){
            raw::Status::Ok => Ok(()),
            raw::Status::Err => Err(error!(
                "Error while hset key value, sth of err occured inside redismodule api"
            ))
        }
    }
}

impl Drop for RedisKeyWritable {
    // Frees resources appropriately as a RedisKey goes out of scope.
    fn drop(&mut self) {
        raw::close_key(self.key_inner);
    }
}

/// `RedisString` is an abstraction over a Redis string.
///
/// Its primary function is to ensure the proper deallocation of resources when
/// it goes out of scope. Redis normally requires that strings be managed
/// manually by explicitly freeing them when you're done. This can be a risky
/// prospect, especially with mechanics like Rust's `?` operator, so we ensure
/// fault-free operation through the use of the Drop trait.
#[derive(Debug)]
pub struct RedisString {
    ctx:       *mut raw::RedisModuleCtx,
    str_inner: *mut raw::RedisModuleString,
}

impl RedisString {
    fn create(ctx: *mut raw::RedisModuleCtx, s: &str) -> RedisString {
        let str_inner = raw::create_string(ctx, format!("{}\0", s).as_ptr(), s.len());
        RedisString { ctx, str_inner }
    }
}

impl Drop for RedisString {
    // Frees resources appropriately as a RedisString goes out of scope.
    fn drop(&mut self) {
        raw::free_string(self.ctx, self.str_inner);
    }
}


#[derive(Debug)]
pub struct RedisCallReply {
    reply: *mut raw::RedisModuleCallReply
}

impl RedisCallReply {
    fn create(reply: *mut raw::RedisModuleCallReply) -> RedisCallReply {
        RedisCallReply{ reply }
    }

    fn check_type(&self) -> raw::ReplyType {
        raw::call_reply_type(self.reply)
    }

    fn to_integer(&self) -> Result<i64, RModError> {
        if self.check_type() != raw::ReplyType::Integer {
            return Err(error!("Invalid type of CallReply, not Integer"))
        }
        Ok(raw::call_reply_integer(self.reply) as i64)
    }

    fn to_string(&self) -> Result<String, RModError> {
        if self.check_type() != raw::ReplyType::String {
            return Err(error!("Invalid type of CallReply, not String"))
        }

        let mut length: size_t = 0;
        let char_ptr = raw::call_reply_string_ptr(self.reply, &mut length);
        match from_byte_string(char_ptr, length){
            Ok(s) => Ok(s),
            Err(_) => Err(error!("failed to parse char pointer"))
        }
    }

    fn check_length(&self) -> size_t {
        raw::call_reply_length(self.reply)
    }

    fn reply_array_element(&self, idx: size_t) -> Result<RedisCallReply, RModError> {
        if self.check_type() != raw::ReplyType::Array {
            return Err(error!("Invalid type of CallReply, not Array"))
        }
        Ok(RedisCallReply::create(raw::call_reply_array_element(self.reply, idx)))
    }
}

impl Drop for RedisCallReply {
    fn drop(&mut self) {
        raw::free_call_reply(self.reply);
    }
}


pub struct RedisAlloc;
unsafe impl GlobalAlloc for RedisAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = (layout.size() + layout.align() - 1) & (!(layout.align() - 1));
        raw::rm_alloc(size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        raw::rm_free(ptr);
    }
}

fn handle_status(status: raw::Status, message: &str) -> Result<(), RModError> {
    match status {
        raw::Status::Ok => Ok(()),
        raw::Status::Err => Err(error!(message)),
    }
}



fn manifest_redis_string(
    redis_str: *mut raw::RedisModuleString,
) -> Result<String, string::FromUtf8Error> {
    let mut length: size_t = 0;
    let bytes = raw::string_ptr_len(redis_str, &mut length);
    from_byte_string(bytes, length)
}

fn parse_args(
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> Result<Vec<String>, string::FromUtf8Error> {
    let mut args: Vec<String> = Vec::with_capacity(argc as usize);
    for i in 0..argc {
        let redis_str = unsafe { *argv.offset(i as isize) };
        args.push(manifest_redis_string(redis_str)?);
    }
    Ok(args)
}

fn from_byte_string(
    byte_str: *const u8,
    length: size_t,
) -> Result<String, string::FromUtf8Error> {
    let mut vec_str: Vec<u8> = Vec::with_capacity(length as usize);
    for j in 0..length {
        let byte: u8 = unsafe { *byte_str.offset(j as isize) };
        vec_str.insert(j, byte);
    }

    String::from_utf8(vec_str)
}

fn read_key(key: *mut raw::RedisModuleKey) -> Result<String, string::FromUtf8Error> {
    let mut length: size_t = 0;
    from_byte_string(
        raw::string_dma(key, &mut length, raw::KeyMode::READ),
        length,
    )
}

fn to_raw_mode(mode: KeyMode) -> raw::KeyMode {
    match mode {
        KeyMode::Read => raw::KeyMode::READ,
        KeyMode::ReadWrite => raw::KeyMode::READ | raw::KeyMode::WRITE,
    }
}
