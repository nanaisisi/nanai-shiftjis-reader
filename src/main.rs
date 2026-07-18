// Shift_JIS テキストビューアのエントリーポイント。
// MSIXパッケージ状態を確認した後、ファイルをデコードしてGUIを起動する。
mod text_io;
mod ui;
mod windows_msix;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MSIXパッケージとして動作しているかどうかを確認し、パッケージファミリー名を表示する
    #[cfg(target_os = "windows")]
    windows_msix::check_msix_package();

    // コマンドライン引数で指定されたファイルをShift_JISとして読み込み、UTF-8にデコードする
    let decoded_text = text_io::file_process().unwrap_or_else(|err| err.to_string());
    // デコードされたテキストをGUIウィンドウで表示する
    ui::ui(decoded_text);
    Ok(())
}
