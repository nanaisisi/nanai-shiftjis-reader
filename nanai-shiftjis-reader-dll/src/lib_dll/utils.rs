#![cfg(windows)]

use std::{
    ffi::{OsStr, c_void},
    os::windows::prelude::{OsStrExt, OsStringExt},
    ptr,
};
use windows::{
    Win32::{
        Foundation::E_OUTOFMEMORY,
        System::Com::{CoTaskMemAlloc, CoTaskMemFree},
        UI::Shell::{IShellItemArray, SIGDN_FILESYSPATH},
    },
    core::{Interface, PWSTR},
};

/// Explorer で選択されているファイルのパスをすべて取得する。
/// `psiitemarray` が `null` または空の場合は空の `Vec` を返す。
pub unsafe fn get_selected_file_paths(psiitemarray: *mut c_void) -> Vec<std::path::PathBuf> {
    if psiitemarray.is_null() {
        return Vec::new();
    }

    let raw_unknown = unsafe { windows::core::IUnknown::from_raw(psiitemarray as *mut _) };
    let item_array: IShellItemArray = match raw_unknown.cast() {
        Ok(array) => array,
        Err(_) => {
            std::mem::forget(raw_unknown);
            return Vec::new();
        }
    };
    std::mem::forget(raw_unknown);

    let item_count = match unsafe { item_array.GetCount() } {
        Ok(count) => count,
        Err(_) => return Vec::new(),
    };

    let mut paths = Vec::new();
    for index in 0..item_count {
        let item = match unsafe { item_array.GetItemAt(index) } {
            Ok(item) => item,
            Err(_) => continue,
        };

        let psz_path = match unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) } {
            Ok(path) => path,
            Err(_) => continue,
        };
        if psz_path.is_null() {
            continue;
        }

        let mut len = 0usize;
        unsafe {
            while *psz_path.0.add(len) != 0 {
                len += 1;
            }
        }
        let path =
            unsafe { std::ffi::OsString::from_wide(std::slice::from_raw_parts(psz_path.0, len)) };
        unsafe {
            CoTaskMemFree(Some(psz_path.0 as *const _));
        }
        paths.push(std::path::PathBuf::from(path));
    }

    paths
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
