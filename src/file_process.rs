use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// パス文字列の先頭と末尾にあるクォート文字（`"` または `'`）を取り除く。
/// Windowsのドラッグ&ドロップ等でパスがクォートで囲まれて渡される場合に対応する。
fn strip_quotes(path: &str) -> &str {
    let trimmed = path.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            return &trimmed[1..trimmed.len() - 1];
        }
    }
    trimmed
}

/// コマンドライン引数からファイルパスを取得し、Shift_JISとしてデコードしたUTF-8文字列を返す。
/// 引数が指定されていない場合やファイル内容が空の場合は `"None content"` を返す。
pub fn file_process() -> Result<String> {
    let maybe_path = env::args().nth(1);
    let decoded = if let Some(path_str) = maybe_path {
        let path = PathBuf::from(strip_quotes(&path_str));
        let input_file =
            fs::read(&path).with_context(|| format!("couldn't open {}", path.display()))?;
        let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&input_file);
        res.into_owned()
    } else {
        String::new()
    };

    if decoded.trim().is_empty() {
        Ok(String::from("None content"))
    } else {
        Ok(decoded)
    }
}
