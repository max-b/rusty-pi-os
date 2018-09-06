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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[macro_use]
extern crate core;
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

#[macro_export]
pub mod console;
