
// Using unstable features
#![feature(libc, io, os, core)]

extern crate libc;

#[macro_use]
extern crate bitflags;

pub mod err;
pub mod fd;
