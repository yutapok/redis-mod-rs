// `raw` should not be public in the long run. Build an abstraction interface
// instead.
//
// We have to disable a couple Clippy checks here because we'll otherwise have
// warnings thrown from within macros provided by the `bigflags` package.
#[cfg_attr(feature = "cargo-clippy",
           allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod raw;

use std::error::Error;
use crate::error::RModError;
use libc::{c_int, c_long, c_longlong, size_t};
use std::ptr;
use std::string;
use time;

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

impl Command {
    /// Provides a basic wrapper for a command's implementation that parses
    /// arguments to Rust data types and handles the OK/ERR reply back to Redis.    
    pub fn harness(
        command: &Command,
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
                    format!("RMod error: {}\0", e.description()).as_ptr(),
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

    pub fn reply_with_simple_string(&self, str: &str) -> Result<(), RModError> {
        let msg = str.as_ptr();
        handle_status(
            raw::reply_with_simple_string(self.ctx, msg),
            "Could not reply with simple string",
        )
    }

    pub fn reply_ok(&self){
        self.reply_with_simple_string("Ok").unwrap_or(());
    }

    pub fn reply_null(&self) {
        raw::reply_with_null(self.ctx);
    }

    pub fn reply_null_with_ok(&self, msg: &str) -> Result<(), RModError>{
        self.log(LogLevel::Warning, msg);
        Ok(self.reply_null())
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
        let place: c_int = -1;
        let redis_str = raw::list_pop(self.key_inner,place);
        match manifest_redis_string(redis_str){
            Ok(re_str) => Ok(Some(re_str)),
            Err(_) => Err(error!("Error while rpop to key, tried to the wrong type"))
        }
    }

    //pub fn lpop(&self, ele: &str) -> Result<(), RModError> {
    //    let place: c_int = 0;
    //    match raw::list_pop(self.key_inner,place) {
    //        raw::Status::Ok => Ok(()),
    //        raw::Status::Err => Err(error!("Error while lpop to key, tried to the wrong type"))
    //    }
    //}
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
