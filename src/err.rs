
use std::old_io as io;
use std::fmt;

#[macro_export]
macro_rules! usage_err {
    ($($arg:tt)*) => (
        tlpi_rust::err::usage_err_fmt(format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! err_exit {
    ($errno:expr, $($arg:tt)*) => (
        tlpi_rust::err::err_exit_fmt($errno, format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => (
        tlpi_rust::err::fatal_fmt(format_args!($($arg)*))
    )
}

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

pub type TlpiErr<'a> = Fn(&mut io::Writer) -> () + 'a;

pub fn write_err<'a, F>(fmt: fmt::Arguments, err: F)
    where F: Fn(&mut io::Writer) -> () + 'a
{
    io::stdio::stdout().flush().unwrap();

    let mut stderr = io::stdio::stderr();
    err(&mut stderr);
    stderr.write_fmt(fmt).unwrap();
    write!(&mut stderr, "\n").unwrap();
    stderr.flush().unwrap();
}

pub fn usage_err_fmt(fmt: fmt::Arguments) -> bool {
    write_err(fmt, |&: writer| {
        write!(writer, "Usage: ").unwrap();
    });
    false
}

pub fn err_exit_fmt(errno: Errno, fmt: fmt::Arguments) -> bool {
    let Errno(err) = errno;
    let error_name =
        if err > 0 && err < ENAME.len() { ENAME[err] } else { "?UNKNOWN?" };
    let io_error = io::IoError::from_errno(err, true);
    let detail = match io_error.detail {
        Some(ref d) => format!(" ({})", d),
        _ => String::new()
    };

    write_err(fmt, |&: writer| {
        write!(
            writer, "ERROR [{} ({:?}); {}{}] ", error_name, io_error.kind,
            io_error.desc, detail
        ).unwrap();
    });
    false
}

pub fn fatal_fmt(fmt: fmt::Arguments) -> bool {
    write_err(fmt, |&: writer| {
        write!(writer, "ERROR: ").unwrap();
    });
    false
}

#[derive(Copy, Debug)]
pub struct Errno(usize);

impl Errno {

    pub fn new(value: usize) -> Errno { Errno(value) }

}
