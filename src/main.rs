// Shift_JIS テキストビューアのエントリーポイント。
// MSIXパッケージ状態を確認した後、ファイルをデコnードしてGUIを起動する。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod text_io;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // コマンドライン引数で指定されたファイルをShift_JISとして読み込み、UTF-8にデコードする
    let loaded_file = text_io::file_process().unwrap_or_else(|err| text_io::LoadedFile {
        path: None,
        content: err.to_string(),
    });
    // デコードされたテキストをGUIウィンドウで表示・編集する
    ui::ui(loaded_file);
    Ok(())
}
