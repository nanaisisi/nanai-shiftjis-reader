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

pub(super) fn to_wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

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

    let item = unsafe { item_array.GetItemAt(0).ok()? };
    let psz_path = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH).ok()? };
    if psz_path.is_null() {
        return None;
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

    let mut wide_path: Vec<u16> = path.encode_wide().collect();
    wide_path.push(0);
    Some(wide_path)
}

pub unsafe fn launch_with_viewer(exe_path: &[u16], file_path: &[u16]) -> bool {
    let exe_pcw = PCWSTR(exe_path.as_ptr());
    let params = PCWSTR(file_path.as_ptr());
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

pub const CLSID_EXPLORER_COMMAND: GUID = GUID::from_values(
    0x13c09516,
    0x2403,
    0x4688,
    [0x9c, 0xf2, 0x2c, 0x9c, 0x41, 0xf3, 0x38, 0xbf],
);
