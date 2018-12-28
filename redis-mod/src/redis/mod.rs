// `raw` should not be public in the long run. Build an abstraction interface
// instead.
//
// We have to disable a couple Clippy checks here because we'll otherwise have
// warnings thrown from within macros provided by the `bigflags` package.
#[cfg_attr(feature = "cargo-clippy",
           allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod raw;

use error::CellError;
use libc::{c_int, c_long, c_longlong, size_t};
use std::error::Error;
use std::iter;
use std::ptr;
use std::string;

