#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate time;

#[macro_use]
pub mod macros;

pub mod redis;
pub use crate::redis::{raw, Command};
pub mod error;
pub use crate::error::RModError;

use libc::c_int;

const MODULE_NAME: &str = "rmod";
const MODULE_VERSION: c_int = 1;

pub struct RedisModuleInitializer {
    module_name: &'static str,
    module_version: c_int
}

impl RedisModuleInitializer {
    pub fn new(
        mod_name: &'static str,
        mod_ver: c_int
    ) -> Self {
      module_name: mod_name,
      module_version: mod_ver
    }

    pub fn run(&self,
        ctx: *mut raw::RedisModuleCtx,
        argv: *mut *mut raw::RedisModuleString,
        argc: c_int
    ) -> raw::Status {
        if raw::init(
            ctx,
            format!("{}\0", self.module_name).as_ptr(),
            self.module_version,
            raw::REDISMODULE_APIVER_1,
        ) == raw::Status::Err
        {
            return raw::Status::Err;
        }

        redis::enable_redis_allocator();

        return raw::Status::Ok;

    }
}
