
#![feature(collections, core, env, libc)]

#[macro_use]
extern crate tlpi_rust;

extern crate libc;

use std::env;
use tlpi_rust::fd::*;
use libc::{EXIT_SUCCESS, EXIT_FAILURE};
use std::num;

fn main() {
    let status = if main_with_result() { EXIT_SUCCESS } else { EXIT_FAILURE };
    env::set_exit_status(status);
}

fn main_with_result() -> bool {
    let argv: Vec<_> = env::args().collect();

    if argv.len() < 3 || argv[1] == "--help" {
        return usage_err!(
            "{} file {{r<length>|R<length>|w<string>|s<offset>}}...", argv[0]
        );
    }

    let flags = O_RDWR | O_CREAT;
    // rw-rw-rw
    let perms = S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH;

    let fd = match FileDescriptor::open(argv[1].clone(), flags, perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "open")
    };

    for arg in argv.iter().skip(2) {
        let result = match arg.char_at(0) {
            'r' | 'R' => read_file(&fd, arg),
            'w' => write_file(&fd, arg),
            's' => seek_file(&fd, arg),
            _ => cmd_line_err!("Argument must start with [rRws]: {}", arg)
        };
        if !result { return false };
    }

    true
}

fn read_file(fd: &FileDescriptor, arg: &str) -> bool {
    let byte_count = match num::FromStrRadix::from_str_radix(&arg[1..], 10) {
        Ok(count) => count,
        _ => return cmd_line_err!("Invalid length: {}", arg)
    };

    let mut buf = vec![0u8; byte_count];
    let num_read = match fd.read(buf.as_mut_slice()) {
        Ok(count) => count,
        Err(errno) => return err_exit!(errno, "read"),
    };

    print!("{}: ", arg);
    if num_read == 0 {
        println!("end-of-file");
    } else {
        display_bytes(&buf[..num_read], arg.char_at(0));
    }

    true
}

fn display_bytes(bytes: &[u8], format: char) {
    match format {
        'r' => {
            // TODO Handle UTF-8 error gracefully
            let buf_str = std::str::from_utf8(bytes).unwrap();
            for c in buf_str.chars() {
                let out_char = if c.is_control() { '?' } else { c };
                print!("{}", out_char);
            }
        },
        _ => {
            for byte in bytes {
                print!("{:0>2x} ", byte);
            }
        }
    };
    println!("");
}

fn write_file(fd: &FileDescriptor, arg: &str) -> bool {
    let num_written = match fd.write(arg[1..].as_bytes()) {
        Ok(bytes) => bytes,
        Err(errno) => return err_exit!(errno, "write")
    };
    println!("{}: wrote {} bytes", arg, num_written);
    true
}

fn seek_file(fd: &FileDescriptor, arg: &str) -> bool {
    let offset = match num::FromStrRadix::from_str_radix(&arg[1..], 10) {
        Ok(count) => count,
        _ => return cmd_line_err!("Invalid offset: {}", arg)
    };

    match fd.lseek(offset, OffsetBase::SeekSet) {
        Err(errno) => return err_exit!(errno, "lseek"),
        _ => {}
    };

    println!("{}: seek succeeded", arg);
    true
}
