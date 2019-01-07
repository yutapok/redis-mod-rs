#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate time;

#[macro_use]
mod macros;

pub mod error;
mod redis;

use std::str;
use crate::redis::Command;
use crate::redis::raw;
use crate::error::RModError;
use libc::c_int;


const MODULE_NAME: &str = "rmod-diffsync-rs";
const MODULE_VERSION: c_int = 1;

pub struct DiffSyncDeqCommand {}

impl Command for DiffSyncDeqCommand {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str {
        "diffsync.deq"
    }

    // Run the command.
    fn run(&self, r: redis::Redis, args: &[&str]) -> Result<(), RModError> {
        if args.len() != 2 {
            return Err(error!(
                "Usage: {} <key> <accoutid>",
                self.name()
            ));
        }

        let key = args[1];
        let restr = r.open_key(&key).read().unwrap().unwrap();
        r.reply_string(&restr);

        Ok(())
    }
    fn str_flags(&self) -> &'static str {
        "write"
    }
}

pub struct DiffSyncEnqCommand {}
impl Command for DiffSyncEnqCommand {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str {
        "diffsync.enq"
    }

    // Run the command.
    fn run(&self, r: redis::Redis, args: &[&str]) -> Result<(), RModError> {
        if args.len() != 3 {
            return Err(error!(
                "Usage: {} <key> init:<unix> upd:<unix>",
                self.name()
            ));
        }

        let key = args[1];
        let init_unix = args[2];
        let upd_unix = args[3];

        let val = format!("{}, {}", init_unix, upd_unix);

        r.open_key_writable(key).write(&val).unwrap();
        r.reply_string("ok");

        Ok(())
    }

    fn str_flags(&self) -> &'static str {
        "write"
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn DiffSyncDeq_RedisCommand(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> raw::Status {
    Command::harness(&DiffSyncDeqCommand {}, ctx, argv, argc)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn DiffSyncEnq_RedisCommand(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> raw::Status {
    Command::harness(&DiffSyncEnqCommand {}, ctx, argv, argc)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RedisModule_OnLoad(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> raw::Status {
    if raw::init(
        ctx,
        format!("{}\0", MODULE_NAME).as_ptr(),
        MODULE_VERSION,
        raw::REDISMODULE_APIVER_1,
    ) == raw::Status::Err
    {
        return raw::Status::Err;
    }

    let get_command = DiffSyncDeqCommand {};
    let set_command = DiffSyncEnqCommand {};
    if raw::create_command(
        ctx,
        format!("{}\0", get_command.name()).as_ptr(),
        Some(DiffSyncDeq_RedisCommand),
        format!("{}\0", get_command.str_flags()).as_ptr(),
        0,
        0,
        0,
    ) == raw::Status::Err
    {
        return raw::Status::Err;
    }

    if raw::create_command(
        ctx,
        format!("{}\0", set_command.name()).as_ptr(),
        Some(DiffSyncEnq_RedisCommand),
        format!("{}\0", set_command.str_flags()).as_ptr(),
        0,
        0,
        0,
    ) == raw::Status::Err
    {
        return raw::Status::Err;
    }

    raw::Status::Ok
}
