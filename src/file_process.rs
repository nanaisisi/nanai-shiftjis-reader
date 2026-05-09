use std::env;
use std::fs;
use std::path::PathBuf;

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
pub fn file_process() -> String {
    // 第1引数をファイルパスとして使用する
    let maybe_path = env::args().nth(1);
    let decoded = if let Some(path_str) = maybe_path {
        let path = PathBuf::from(strip_quotes(&path_str));
        match fs::read(&path) {
            Ok(input_file) => {
                // Shift_JISバイト列をUTF-8文字列にデコードする
                let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&input_file);
                res.into_owned()
            }
            Err(why) => format!("couldn't open {}: {}", path.display(), why),
        }
    } else {
        String::new()
    };

    // デコード結果が空白のみの場合はプレースホルダー文字列を返す
    if decoded.trim().is_empty() {
        String::from("None content")
    } else {
        decoded
    }
}
