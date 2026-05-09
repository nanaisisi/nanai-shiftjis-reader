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
