#![feature(allow_internal_unstable)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(decl_macro)]
#![feature(never_type)]
#![feature(extern_prelude)]
#![feature(optin_builtin_traits)]
#![feature(nll)]
#![feature(allocator_api)]
#![feature(alloc)]
#![feature(raw_vec_internals)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;

extern crate volatile;
extern crate stack_vec;
extern crate byteorder;

pub mod timer;
pub mod uart;
pub mod gpio;
pub mod mutex;
pub mod mailbox;
pub mod propertytag;
pub mod framebuffer;
pub mod common;
pub mod allocator;
mod character_set;
pub mod console;
pub mod screen;
pub mod atags;
