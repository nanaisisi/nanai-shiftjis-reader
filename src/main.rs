// Shift_JIS テキストビューアのエントリーポイント。
// MSIXパッケージ状態を確認した後、ファイルをデコードしてGUIを起動する。
mod file_process;
mod ui;
use windows::ApplicationModel::Package;

fn main() {
    // MSIXパッケージとして動作しているかどうかを確認し、パッケージファミリー名を表示する
    match Package::Current() {
        Ok(package) => match package.Id() {
            Ok(id) => match id.FamilyName() {
                Ok(name) => println!("Package Family Name: {}", name),
                Err(e) => println!("Error getting family name: {}", e),
            },
            Err(e) => println!("Error getting package ID: {}", e),
        },
        Err(_) => println!("Not packaged"),
    }

    // コマンドライン引数で指定されたファイルをShift_JISとして読み込み、UTF-8にデコードする
    let decoded_text = file_process::file_process().unwrap_or_else(|err| err.to_string());
    // デコードされたテキストをGUIウィンドウで表示する
    ui::ui(decoded_text);
}
