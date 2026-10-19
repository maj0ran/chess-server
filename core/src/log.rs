pub const PREFIX_SEND: &str = "\x1B[35m[SEND]\x1B[39m";
pub const PREFIX_RECV: &str = "\x1B[36m[RECV]\x1B[39m";
pub const PREFIX_PROC: &str = "\x1B[94m[PROC]\x1B[39m";

#[macro_export]
macro_rules! log_send {
    ($level:ident, $($arg:tt)+) => {
        log::$level!("{} {}", $crate::PREFIX_SEND, format_args!($($arg)+))
    };
}
#[macro_export]
macro_rules! log_recv {
    ($level:ident, $($arg:tt)+) => {
        log::$level!("{} {}", $crate::PREFIX_RECV, format_args!($($arg)+))
    };
}
#[macro_export]
macro_rules! log_proc {
    ($level:ident, $($arg:tt)+) => {
        log::$level!("{} {}", $crate::PREFIX_PROC, format_args!($($arg)+))
    };
}

