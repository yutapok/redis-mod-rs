#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate time;

#[macro_use]
pub mod macros;

pub mod redis;
pub use crate::redis::{raw, Command, RedisAlloc};
pub mod error;
pub use crate::error::RModError;

use std::sync::atomic::AtomicBool;


#[global_allocator]
static ALLOC: RedisAlloc = RedisAlloc {
    use_redis_ab: AtomicBool::new(false)
};


