use encoding_rs;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn file_process() -> String {
    let maybe_path = env::args().nth(1);
    let decoded = if let Some(path_str) = maybe_path {
        let path = PathBuf::from(path_str);
        let input_file = fs::read(&path)
            .unwrap_or_else(|why| panic!("couldn't open {}: {}", path.display(), why));
        let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&input_file);
        res.into_owned()
    } else {
        String::new()
    };

    if decoded.trim().is_empty() {
        String::from("None content")
    } else {
        decoded
    }
}
