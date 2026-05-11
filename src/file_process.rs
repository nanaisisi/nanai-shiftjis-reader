use std::env;
use std::path::PathBuf;
use std::thread;

use anyhow::{Context, Result, anyhow};
use compio::{fs::OpenOptions, io::AsyncReadAtExt, runtime::Runtime};
use encoding_rs::SHIFT_JIS;

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

/// compio を使って一つのファイルを読み込み、Shift_JIS を UTF-8 にデコードする。
fn read_file_with_compio(path: PathBuf) -> Result<String> {
    let display_path = path.display().to_string();
    Runtime::new()
        .with_context(|| format!("cannot create compio runtime for {}", display_path))?
        .block_on(async move {
            let file = OpenOptions::new()
                .read(true)
                .open(&path)
                .await
                .with_context(|| format!("couldn't open {}", path.display()))?;

            let buf_result = file.read_to_end_at(Vec::with_capacity(16 * 1024), 0).await;
            let (read_res, buffer) = buf_result.into_parts();
            read_res.with_context(|| format!("couldn't read {}", path.display()))?;

            file.close()
                .await
                .with_context(|| format!("couldn't close {}", path.display()))?;

            let (decoded, _, _) = SHIFT_JIS.decode(&buffer);
            Ok(decoded.into_owned())
        })
}

/// コマンドライン引数からファイルパスを取得し、Shift_JISとしてデコードしたUTF-8文字列を返す。
/// 引数が複数ある場合はすべてを並列に処理する。
/// 引数が指定されていない場合やファイル内容が空の場合は `"None content"` を返す。
pub fn file_process() -> Result<String> {
    let paths: Vec<PathBuf> = env::args()
        .skip(1)
        .map(|arg| PathBuf::from(strip_quotes(&arg)))
        .collect();

    if paths.is_empty() {
        return Ok(String::from("None content"));
    }

    let handles: Vec<_> = paths
        .into_iter()
        .map(|path| thread::spawn(move || read_file_with_compio(path)))
        .collect();

    let mut output = String::new();
    for (index, handle) in handles.into_iter().enumerate() {
        let decoded_file = handle
            .join()
            .map_err(|err| anyhow!("thread panicked: {err:?}"))??;

        if index > 0 {
            output.push_str("\n\n---\n\n");
        }
        output.push_str(&decoded_file);
    }

    if output.trim().is_empty() {
        Ok(String::from("None content"))
    } else {
        Ok(output)
    }
}
