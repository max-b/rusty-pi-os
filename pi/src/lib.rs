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
#![feature(pointer_methods)]
#![feature(allocator_api)]

extern crate volatile;
extern crate stack_vec;

pub mod timer;
pub mod uart;
pub mod gpio;
pub mod mutex;
pub mod mailbox;
pub mod propertytag;
pub mod framebuffer;
pub mod common;
pub mod allocator;

#[macro_export]
pub mod console;
pub mod atags;
