// Windows Explorer のコンテキストメニューコマンドを実装する COM DLL の公開エントリーポイント。
// Windows 専用であり、他のプラットフォームではコンパイルされない。
#![cfg(windows)]

mod lib_dll;

pub use lib_dll::*;
