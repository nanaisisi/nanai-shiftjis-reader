use encoding_rs;
use std::env;
use std::fs;
use std::path::PathBuf;

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

pub fn file_process() -> String {
    let maybe_path = env::args().nth(1);
    let decoded = if let Some(path_str) = maybe_path {
        let path = PathBuf::from(strip_quotes(&path_str));
        match fs::read(&path) {
            Ok(input_file) => {
                let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&input_file);
                res.into_owned()
            }
            Err(why) => String::from(format!("couldn't open {}: {}", path.display(), why)),
        }
    } else {
        String::new()
    };

    if decoded.trim().is_empty() {
        String::from("None content")
    } else {
        decoded
    }
}
