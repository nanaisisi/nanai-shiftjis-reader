#![cfg(windows)]

use std::{
    ffi::{OsStr, c_void},
    os::windows::prelude::{OsStrExt, OsStringExt},
    path::PathBuf,
    ptr,
};
use windows::{
    Win32::{
        Foundation::E_OUTOFMEMORY,
        System::{
            Com::{CoTaskMemAlloc, CoTaskMemFree},
            LibraryLoader::{GetModuleFileNameW, GetModuleHandleW},
        },
        UI::Shell::{IShellItemArray, SIGDN_FILESYSPATH, ShellExecuteW},
        UI::WindowsAndMessaging::SW_SHOW,
    },
    core::{GUID, Interface, PCWSTR, PWSTR},
};

/// 文字列をnull終端のUTF-16ワイド文字列に変換する。
pub(super) fn to_wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

/// 現在のDLLモジュールのディレクトリパスを取得する。
/// 複数の候補ファイル名でモジュールハンドルを検索し、最初に見つかったものを返す。
pub unsafe fn get_dll_directory() -> Option<PathBuf> {
    let module_names = [
        "nanai_shiftjis_reader_dll.dll",
        "nanai_shiftjis_reader.dll",
        "nanai-shiftjis-reader.dll",
    ];
    for name in module_names {
        let module_name = to_wide_null(name);
        if let Ok(module) = unsafe { GetModuleHandleW(PCWSTR(module_name.as_ptr())) } {
            if !module.is_invalid() {
                let mut buffer = vec![0u16; 260];
                let len = unsafe { GetModuleFileNameW(Some(module), &mut buffer) };
                if len != 0 {
                    buffer.truncate(len as usize);
                    return Some(PathBuf::from(std::ffi::OsString::from_wide(&buffer)));
                }
            }
        }
    }
    None
}

/// DLLと同じディレクトリにあるビューア実行ファイルのパスをワイド文字列で返す。
/// 候補のファイル名を順に検索し、存在する最初のものを返す。
pub unsafe fn app_executable_path() -> Option<Vec<u16>> {
    let mut path = unsafe { get_dll_directory()? };
    let candidates = ["nanai-shiftjis-reader.exe", "nanai_shiftjis_reader.exe"];
    for exe_name in candidates {
        path.set_file_name(exe_name);
        if path.exists() {
            let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
            wide.push(0);
            return Some(wide);
        }
    }
    None
}

/// Explorer で選択されているファイルのパスをワイド文字列で取得する。
/// `psiitemarray` が `null` または空の場合は `None` を返す。
/// 先頭のアイテムのファイルシステムパスを返す。
pub unsafe fn get_selected_file_path(psiitemarray: *mut c_void) -> Option<Vec<u16>> {
    if psiitemarray.is_null() {
        return None;
    }

    let raw_unknown = unsafe { windows::core::IUnknown::from_raw(psiitemarray as *mut _) };
    let item_array: IShellItemArray = match raw_unknown.cast() {
        Ok(array) => array,
        Err(_) => {
            std::mem::forget(raw_unknown);
            return None;
        }
    };
    std::mem::forget(raw_unknown);

    let item_count = unsafe { item_array.GetCount().ok()? };
    if item_count == 0 {
        return None;
    }

    // 先頭のアイテムのファイルシステムパスを取得する
    let item = unsafe { item_array.GetItemAt(0).ok()? };
    let psz_path = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH).ok()? };
    if psz_path.is_null() {
        return None;
    }

    // null終端までの長さを計算する
    let mut len = 0usize;
    unsafe {
        while *psz_path.0.add(len) != 0 {
            len += 1;
        }
    }
    let path =
        unsafe { std::ffi::OsString::from_wide(std::slice::from_raw_parts(psz_path.0, len)) };
    // COMが確保したメモリを解放する
    unsafe {
        CoTaskMemFree(Some(psz_path.0 as *const _));
    }

    let mut wide_path: Vec<u16> = path.encode_wide().collect();
    wide_path.push(0);
    Some(wide_path)
}

/// 指定された実行ファイルでファイルを開く。
/// `ShellExecuteW` を使ってビューアアプリを起動し、成功した場合は `true` を返す。
pub unsafe fn launch_with_viewer(exe_path: &[u16], file_path: &[u16]) -> bool {
    let exe_pcw = PCWSTR(exe_path.as_ptr());

    let path_len = file_path
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(file_path.len());
    let file_path_str = String::from_utf16_lossy(&file_path[..path_len]);
    // ファイルパスをダブルクォートで囲んでコマンドライン引数として渡す
    let quoted = format!("\"{}\"", file_path_str);
    let params_wide: Vec<u16> = OsStr::new(&quoted).encode_wide().chain(Some(0)).collect();
    let params = PCWSTR(params_wide.as_ptr());

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(ptr::null()),
            exe_pcw,
            params,
            PCWSTR(ptr::null()),
            SW_SHOW,
        )
    };
    (result.0 as isize) > 32
}

/// UTF-8文字列をCOMタスクメモリにコピーしたワイド文字列（`PWSTR`）として返す。
/// 呼び出し元は使用後に `CoTaskMemFree` でメモリを解放する責任を持つ。
pub fn allocate_pwstr(value: &str) -> windows::core::Result<PWSTR> {
    let wide: Vec<u16> = OsStr::new(value).encode_wide().chain(Some(0)).collect();

    let size = wide.len() * std::mem::size_of::<u16>();
    let raw = unsafe { CoTaskMemAlloc(size) } as *mut u16;
    if raw.is_null() {
        return Err(E_OUTOFMEMORY.into());
    }

    unsafe {
        ptr::copy_nonoverlapping(wide.as_ptr(), raw, wide.len());
    }

    Ok(PWSTR(raw))
}
