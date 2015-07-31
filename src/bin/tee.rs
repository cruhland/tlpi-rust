
#![feature(libc, slice_splits)]

#[macro_use]
extern crate tlpi_rust;

extern crate getopts;
use getopts::Options;

use tlpi_rust::err::*;
use tlpi_rust::fd::*;
use std::env;

fn main() {
    exit_with_status!(main_with_result());
}

fn main_with_result() -> TlpiResult<()> {
    let (output_path, write_mode) = try!(parse_args());

    let path = output_path.clone();
    let flags = O_WRONLY | O_CREAT | write_mode;
    let perms = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH; // rw-r--r--
    let dest_fd = match FileDescriptor::open(path, flags, perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "open() on file {}", output_path),
    };

    let mut buf = [0u8; 1 << 16]; // 64k buffer
    loop {
        let bytes_read = match STDIN.read(&mut buf) {
            Ok(0) => break,
            Ok(count) => count,
            Err(errno) => return err_exit!(errno, "read() on stdin"),
        };

        try!(write_buf(&dest_fd, &buf[..bytes_read], &output_path[..]));
        try!(write_buf(&STDOUT, &buf[..bytes_read], "stdout"));
    }

    dest_fd.close().or_else(|errno| {
        err_exit!(errno, "close() on file {}", output_path)
    })
}

fn write_buf(
    fd: &FileDescriptor, buf: &[u8], dest_name: &str
) -> TlpiResult<()> {
    match fd.write(buf) {
        Ok(count) if count == buf.len() => Ok(()),
        Ok(_) => fatal!("incomplete write to {}", dest_name),
        Err(errno) => err_exit!(errno, "write() to {}", dest_name),
    }
}

fn parse_args() -> TlpiResult<(String, OpenFlags)> {
    let argv: Vec<_> = env::args().collect();
    let opts = build_options();

    let argv_tail = match argv.split_first() {
        Some((_, tail)) => tail,
        _ => return cmd_line_err!("No program name provided!?"),
    };

    // Mutable so we can move out the output path
    let mut matches = match opts.parse(argv_tail) {
        Ok(m) => m,
        Err(f) => {
            let usage = opts.usage(&f.to_string());
            return cmd_line_err!("{}", usage)
        },
    };

    if matches.opt_present("help") {
        let usage = format!("{} [options] <output_file>", argv[0]);
        return usage_err!("{}", opts.usage(&usage));
    }

    if matches.free.len() == 1 {
        let write_mode =
            if matches.opt_present("append") { O_APPEND } else { O_TRUNC };
        Ok((matches.free.swap_remove(0), write_mode))
    } else {
        let usage = opts.usage("Exactly one file argument is required");
        return cmd_line_err!("{}", usage)
    }
}

fn build_options() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "display this usage message");
    opts.optflag("a", "append", "append output instead of truncating");
    opts
}
