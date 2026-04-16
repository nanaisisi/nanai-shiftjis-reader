use encoding_rs;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn file_process() -> String {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("shift_jis.txt"));
    let display = path.display();
    let input_file =
        fs::read(&path).unwrap_or_else(|why| panic!("couldn't open {}: {}", display, why));

    // Shift_JISのバイト列(Vec<u8>) を UTF-8の文字列(String) に変換
    let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&input_file);
    let text = res.into_owned();
    println!("text: {}", text);
    text
}
