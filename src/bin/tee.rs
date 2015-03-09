
#![feature(libc, exit_status)]

#[macro_use]
extern crate tlpi_rust;

use tlpi_rust::err::*;

fn main() {
    set_exit_status!(main_with_result());
}

fn main_with_result() -> TlpiResult<()> {
    // TODO
    // Interpret command-line arguments
    // Open destination file (truncate or append)
    // Until EOF on stdin:
    //   Read a chunk of bytes from stdin
    //   Write that chunk to stdout
    //   Write that chunk to destination file
    // Close destination file

    println!("*** NOT IMPLEMENTED ***");
    Err(())
}
