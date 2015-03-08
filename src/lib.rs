
// Using unstable features
#![feature(libc, old_io, os)]

extern crate libc;

#[macro_use]
extern crate bitflags;

pub mod err;
pub mod fd;
