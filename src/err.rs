
//! Utilities for error handling.

extern crate libc;

use std::old_io as io;
use std::fmt;

// libc provides no doc comments for these; it's clearer
// if they are just mentioned as reexports in the docs
#[doc(no_inline)]
pub use libc::{EXIT_SUCCESS, EXIT_FAILURE};

/// The error value generated by libc functions.
#[derive(Copy, Debug)]
pub struct Errno(i32);

impl Errno {

    /// Create an `Errno` from its raw value.
    pub fn new(value: i32) -> Errno { Errno(value) }

}

/// Result type that has trivial error information.
///
/// It's preferable to `Option` because the compiler will warn if
/// values of `Result` type are not used.
pub type TlpiResult<T> = Result<T, ()>;

/// Reports command-line argument usage errors.
///
/// Expects a format string and arguments, like `println!`. The prefix
/// `"Usage: "` will be added to the format string before display on
/// standard error. Returns an indication of program failure.
#[macro_export]
macro_rules! usage_err {
    ($($arg:tt)*) => (
        tlpi_rust::err::usage_err_fmt(format_args!($($arg)*))
    )
}

/// Reports errors specified by the libc `errno` mechanism.
///
/// Expects an `Errno` value, followed by a format string and
/// arguments, like `println!`. Prints the formatted string to
/// standard error, prefixed by the text `ERROR` and the following
/// diagnostic information for the given `Errno` value:
///
/// - the name of its libc constant;
/// - the name of the equivalent Rust `std::old_io::IoErrorKind`
///   element;
/// - its system-provided short description;
/// - its detail message, if provided.
///
/// Returns an indication of program failure.
#[macro_export]
macro_rules! err_exit {
    ($errno:expr, $($arg:tt)*) => (
        tlpi_rust::err::err_exit_fmt($errno, format_args!($($arg)*))
    )
}

/// Reports generic program errors that don't have an associated
/// `errno` value.
///
/// Expects a format string and arguments, like `println!`. The prefix
/// `"ERROR: "` will be added to the format string before display on
/// standard error. Returns an indication of program failure.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => (
        tlpi_rust::err::fatal_fmt(format_args!($($arg)*))
    )
}

/// Reports command-line argument interpretation errors.
///
/// For example, when an integer argument cannot be parsed.
///
/// Expects a format string and arguments, like `println!`. The prefix
/// `"Command-line usage error: "` will be added to the format string
/// before display on standard error. Returns an indication of program
/// failure.
#[macro_export]
macro_rules! cmd_line_err {
    ($($arg:tt)*) => (
        tlpi_rust::err::cmd_line_err_fmt(format_args!($($arg)*))
    )
}

/// Sets the exit status of the program from an expression.
///
/// The expression must be of type `Result`: `Ok` indicates success;
/// `Err` indicates failure.
#[macro_export]
macro_rules! set_exit_status {
    ($result:expr) => (
        {
            use ::tlpi_rust::err::{EXIT_SUCCESS, EXIT_FAILURE};
            let status =
                if $result.is_ok() { EXIT_SUCCESS } else { EXIT_FAILURE };
            env::set_exit_status(status);
        }
    )
}

/// Helper macro that is used by the other `*_fmt` functions.
///
/// Expects an already-created `fmt::Arguments` value, followed by
/// another format string and arguments, as with `println!`.
macro_rules! write_err {
    ($fmt:ident, $($arg:tt)*) => (
        write_err_fmt(format_args!($($arg)*), $fmt)
    )
}

/// Performs the same function as `usage_err!`, but takes a
/// pre-existing `fmt::Arguments` value.
///
/// This is mainly an implementation detail, but it might be useful
/// for other purposes.
pub fn usage_err_fmt<T>(fmt: fmt::Arguments) -> TlpiResult<T> {
    write_err!(fmt, "Usage: ")
}

/// Performs the same function as `err_exit!`, but takes a
/// pre-existing `fmt::Arguments` value.
///
/// This is mainly an implementation detail, but it might be useful
/// for other purposes.
pub fn err_exit_fmt<T>(errno: Errno, fmt: fmt::Arguments) -> TlpiResult<T> {
    let Errno(err) = errno;
    let err_in_bounds = err > 0 && (err as usize) < ENAME.len();
    let error_name =
        if err_in_bounds { ENAME[err as usize] } else { "?UNKNOWN?" };
    let io_error = io::IoError::from_errno(err, true);
    let detail = match io_error.detail {
        Some(ref d) => format!(" ({})", d),
        _ => String::new()
    };

    write_err!(
        fmt, "ERROR [{} ({:?}); {}{}] ", error_name, io_error.kind,
        io_error.desc, detail
    )
}

/// Performs the same function as `fatal!`, but takes a
/// pre-existing `fmt::Arguments` value.
///
/// This is mainly an implementation detail, but it might be useful
/// for other purposes.
pub fn fatal_fmt<T>(fmt: fmt::Arguments) -> TlpiResult<T> {
    write_err!(fmt, "ERROR: ")
}

/// Performs the same function as `cmd_line_err!`, but takes a
/// pre-existing `fmt::Arguments` value.
///
/// This is mainly an implementation detail, but it might be useful
/// for other purposes.
pub fn cmd_line_err_fmt<T>(fmt: fmt::Arguments) -> TlpiResult<T> {
    write_err!(fmt, "Command-line usage error: ")
}

/// Performs the same function as `write_err!`, but takes a
/// pre-existing `fmt::Arguments` value.
///
/// This is mainly an implementation detail, but it might be useful
/// for other purposes.
fn write_err_fmt<T>(
    prefix_fmt: fmt::Arguments, message_fmt: fmt::Arguments
) -> TlpiResult<T> {
    io::stdio::stdout().flush().unwrap();

    let mut stderr = io::stdio::stderr();
    stderr.write_fmt(prefix_fmt).unwrap();
    stderr.write_fmt(message_fmt).unwrap();
    stderr.write_char('\n').unwrap();
    stderr.flush().unwrap();

    Err(())
}

/// Names for the various documented `errno` values, as defined on an
/// x86-64 architecture with a Linux 3.18 kernel.
///
/// This was generated by the `lib/Build_ename.sh` script provided
/// in the source code distribution for _The Linux Programming
/// Interface_.
static ENAME: [&'static str; 134] = [
    "",
    "EPERM", "ENOENT", "ESRCH", "EINTR", "EIO", "ENXIO",
    "E2BIG", "ENOEXEC", "EBADF", "ECHILD",
    "EAGAIN/EWOULDBLOCK", "ENOMEM", "EACCES", "EFAULT",
    "ENOTBLK", "EBUSY", "EEXIST", "EXDEV", "ENODEV",
    "ENOTDIR", "EISDIR", "EINVAL", "ENFILE", "EMFILE",
    "ENOTTY", "ETXTBSY", "EFBIG", "ENOSPC", "ESPIPE",
    "EROFS", "EMLINK", "EPIPE", "EDOM", "ERANGE",
    "EDEADLK/EDEADLOCK", "ENAMETOOLONG", "ENOLCK", "ENOSYS",
    "ENOTEMPTY", "ELOOP", "", "ENOMSG", "EIDRM", "ECHRNG",
    "EL2NSYNC", "EL3HLT", "EL3RST", "ELNRNG", "EUNATCH",
    "ENOCSI", "EL2HLT", "EBADE", "EBADR", "EXFULL", "ENOANO",
    "EBADRQC", "EBADSLT", "", "EBFONT", "ENOSTR", "ENODATA",
    "ETIME", "ENOSR", "ENONET", "ENOPKG", "EREMOTE",
    "ENOLINK", "EADV", "ESRMNT", "ECOMM", "EPROTO",
    "EMULTIHOP", "EDOTDOT", "EBADMSG", "EOVERFLOW",
    "ENOTUNIQ", "EBADFD", "EREMCHG", "ELIBACC", "ELIBBAD",
    "ELIBSCN", "ELIBMAX", "ELIBEXEC", "EILSEQ", "ERESTART",
    "ESTRPIPE", "EUSERS", "ENOTSOCK", "EDESTADDRREQ",
    "EMSGSIZE", "EPROTOTYPE", "ENOPROTOOPT",
    "EPROTONOSUPPORT", "ESOCKTNOSUPPORT",
    "EOPNOTSUPP/ENOTSUP", "EPFNOSUPPORT", "EAFNOSUPPORT",
    "EADDRINUSE", "EADDRNOTAVAIL", "ENETDOWN", "ENETUNREACH",
    "ENETRESET", "ECONNABORTED", "ECONNRESET", "ENOBUFS",
    "EISCONN", "ENOTCONN", "ESHUTDOWN", "ETOOMANYREFS",
    "ETIMEDOUT", "ECONNREFUSED", "EHOSTDOWN", "EHOSTUNREACH",
    "EALREADY", "EINPROGRESS", "ESTALE", "EUCLEAN",
    "ENOTNAM", "ENAVAIL", "EISNAM", "EREMOTEIO", "EDQUOT",
    "ENOMEDIUM", "EMEDIUMTYPE", "ECANCELED", "ENOKEY",
    "EKEYEXPIRED", "EKEYREVOKED", "EKEYREJECTED",
    "EOWNERDEAD", "ENOTRECOVERABLE", "ERFKILL", "EHWPOISON"
];
