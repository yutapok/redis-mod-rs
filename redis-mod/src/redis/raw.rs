// Allow dead code in here in case I want to publish it as a crate at some
// point.
#![allow(dead_code)]

extern crate libc;

use libc::{c_int, c_long, c_longlong, size_t};

// Rust can't link against C macros (#define) so we just redefine them here.
// There's a ~0 chance that any of these will ever change so it's pretty safe.
pub const REDISMODULE_APIVER_1: c_int = 1;
\
bitflags! {
    pub struct KeyMode: c_int {
        const READ = 1;
        const WRITE = (1 << 1);
    }
}


