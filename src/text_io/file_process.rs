use std::env;
use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{Context, Result, anyhow};
use compio::{fs::OpenOptions, io::AsyncReadAtExt, runtime::Runtime};
use encoding_rs::SHIFT_JIS;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LoadedFile {
    #[allow(dead_code)]
    pub path: Option<PathBuf>,
    #[allow(dead_code)]
    pub content: String,
}

/// 指定されたパスへ Shift_JIS エンコーディングでテキストを保存する。
#[allow(dead_code)]
pub fn save_file_shiftjis(path: &Path, content: &str) -> Result<()> {
    let (encoded_bytes, _, _) = SHIFT_JIS.encode(content);
    std::fs::write(path, &encoded_bytes)
        .with_context(|| format!("couldn't save to {}", path.display()))?;
    Ok(())
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
pub fn file_process() -> Result<LoadedFile> {
    let paths: Vec<PathBuf> = env::args_os().skip(1).map(PathBuf::from).collect();

    if paths.is_empty() {
        return Ok(LoadedFile {
            path: None,
            content: String::from("None content"),
        });
    }

    let primary_path = paths.first().cloned();
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

    let final_content = if output.trim().is_empty() {
        String::from("None content")
    } else {
        output
    };

    Ok(LoadedFile {
        path: primary_path,
        content: final_content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_read_shiftjis() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("nanai_notepad_test.txt");
        let sample_text = "こんにちは、Shift_JIS メモ帳テスト";

        save_file_shiftjis(&test_file, sample_text).expect("save failed");
        let read_result = read_file_with_compio(test_file.clone()).expect("read failed");

        assert_eq!(read_result, sample_text);
        let _ = std::fs::remove_file(test_file);
    }
}
