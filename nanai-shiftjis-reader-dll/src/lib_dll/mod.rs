#![cfg(windows)]

use std::sync::atomic::AtomicU32;

/// DLL内に現在存在するCOMオブジェクトの総数。
/// オブジェクト生成時にインクリメント、解放時にデクリメントする。
pub(super) static GLOBAL_OBJECT_COUNT: AtomicU32 = AtomicU32::new(0);

/// DLLのアンロードを防ぐためのサーバーロックカウント。
/// `LockServer(true)` でインクリメント、`LockServer(false)` でデクリメントする。
pub(super) static GLOBAL_LOCK_COUNT: AtomicU32 = AtomicU32::new(0);

mod class_factory;
mod class_ids;
mod dll;
mod explorer_command;
mod utils;

pub use dll::*;
