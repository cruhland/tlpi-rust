
#![feature(libc, exit_status, collections)]

#[macro_use]
extern crate tlpi_rust;

extern crate getopts;
use getopts::Options;

use tlpi_rust::err::*;
use std::env;

fn main() {
    set_exit_status!(main_with_result());
}

fn main_with_result() -> TlpiResult<()> {
    // TODO
    // Interpret command-line arguments
    let argv: Vec<_> = env::args().collect();
    let opts = build_options();
    let matches = match opts.parse(argv.tail()) {
        Ok(m) => m,
        Err(f) => {
            let usage = opts.usage(&f.to_err_msg()[..]);
            return cmd_line_err!("{}", usage)
        },
    };

    // Open destination file (truncate or append)
    // Until EOF on stdin:
    //   Read a chunk of bytes from stdin
    //   Write that chunk to stdout
    //   Write that chunk to destination file
    // Close destination file

    println!("Matches: {:?}", matches.free);
    Err(())
}

fn build_options() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "display this usage message");
    opts.optflag("a", "append", "append output instead of truncating");
    opts
}
