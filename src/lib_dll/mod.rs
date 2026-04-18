#![cfg(windows)]

use std::sync::atomic::AtomicU32;

pub(super) static GLOBAL_OBJECT_COUNT: AtomicU32 = AtomicU32::new(0);
pub(super) static GLOBAL_LOCK_COUNT: AtomicU32 = AtomicU32::new(0);

mod class_factory;
mod dll;
mod explorer_command;
mod utils;

pub use dll::*;
