
#![feature(libc, exit_status)]

#[macro_use]
extern crate tlpi_rust;

use std::env;
use tlpi_rust::fd::*;

const BUF_SIZE: usize = 1024;

fn main() {
    set_exit_status!(main_with_io());
}

fn main_with_io() -> bool {
    let argv: Vec<_> = env::args().collect();

    if argv.len() != 3 || argv[1] == "--help" {
        return usage_err!("{} old-file new-file", argv[0]);
    }

    // Open input and output files

    let src_path = argv[1].clone();
    let empty_perms = FilePerms::empty();
    let input_fd = match FileDescriptor::open(src_path, O_RDONLY, empty_perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "opening file {}", argv[1])
    };

    let open_flags = O_CREAT | O_WRONLY | O_TRUNC;

    // rw-rw-rw
    let file_perms = S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH;

    let dst_path = argv[2].clone();
    let output_fd = match FileDescriptor::open(dst_path, open_flags, file_perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "opening file {}", argv[2])
    };

    // Transfer data until we encounter end of input or an error

    let mut buf = [0u8; BUF_SIZE];
    loop {
        let bytes_read = match input_fd.read(buf.as_mut_slice()) {
            Ok(0) => break,
            Ok(bytes) => bytes,
            Err(errno) => return err_exit!(errno, "reading file {}", argv[1])
        };

        match output_fd.write(&buf[..bytes_read as usize]) {
            Ok(bytes_written) if bytes_read == bytes_written => {},
            Ok(_) => return fatal!("couldn't write whole buffer"),
            Err(errno) => return err_exit!(errno, "writing file {}", argv[2])
        };
    }

    // Clean up

    match input_fd.close() {
        Err(errno) => return err_exit!(errno, "close input"),
        _ => {}
    };

    match output_fd.close() {
        Err(errno) => return err_exit!(errno, "close output"),
        _ => {}
    };

    true
}
