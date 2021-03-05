#[macro_export]
macro_rules! error {
    ($message:expr) => {
        RModError::generic($message)
    };
    ($message:expr, $($arg:tt)*) => {
        RModError::generic(format!($message, $($arg)+).as_str())
    }
}

//macro_rules! log_debug {
//    ($logger:expr, $target:expr) => {
//        if cfg!(debug_assertions) {
//            $logger.log_debug($target)
//        }
//    };
//    ($logger:expr, $target:expr, $($arg:tt)*) => {
//        if cfg!(debug_assertions) {
//            $logger.log_debug(format!($target, $($arg)+).as_str())
//        }
//    }
//}

#[macro_export]
macro_rules! bultin_command {
    ($name: ident, $command: ident) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[no_mangle]
        pub extern "C" fn $name(
            ctx: *mut raw::RedisModuleCtx,
            argv: *mut *mut raw::RedisModuleString,
            argc: c_int,
        ) -> raw::Status {
            Command::harness(&$command {}, ctx, argv, argc)
        }
    }
}

#[macro_export]
macro_rules! rmod_load {
    ( $( ($builtin: ident ,$command: ident)),*) => {

        $(
            bultin_command!($builtin, $command);
        )*

        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[no_mangle]
        pub extern "C" fn RedisModule_OnLoad(
            ctx: *mut raw::RedisModuleCtx,
            argv: *mut *mut raw::RedisModuleString,
            argc: c_int,
        ) -> raw::Status {
            if RedisModuleInitializer::new(
              ctx,
              MODULE_NAME,
              MODULE_VERSION
            ).run() == raw::Status::Err
            {
                return raw::Status::Err;
            }

            $(

                let command = $command {};
                if raw::create_command(
                    ctx,
                    format!("{}\0", command.name()).as_ptr(),
                    Some($builtin),
                    format!("{}\0", command.str_flags()).as_ptr(),
                    0,
                    0,
                    0,
                 ) == raw::Status::Err
                 {
                     return raw::Status::Err;
                 }

            )*

            raw::Status::Ok


        }
    }
}
