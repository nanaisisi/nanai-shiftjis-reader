#![cfg(windows)]

use super::GLOBAL_OBJECT_COUNT;
use super::guid::CLSID_EXPLORER_COMMAND;
use super::utils::{allocate_pwstr, get_selected_file_path};
use std::{
    ffi::c_void,
    path::PathBuf,
    process::Command,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
use windows::{
    ApplicationModel::Package,
    Win32::{
        Foundation::{E_FAIL, E_NOINTERFACE, E_NOTIMPL, E_POINTER, S_OK},
        UI::Shell::{IExplorerCommand, IExplorerCommand_Vtbl},
    },
    core::{IUnknown, IUnknown_Vtbl, Interface, PWSTR},
};

/// Windows Explorer のコンテキストメニューコマンドを実装するCOMオブジェクト。
/// `IExplorerCommand` インターフェースを手動vtableで実装している。
#[repr(C)]
pub(super) struct ExplorerCommandObject {
    lp_vtbl: *const IExplorerCommand_Vtbl,
    ref_count: AtomicU32,
}

/// `IUnknown::QueryInterface` の実装。
/// `IExplorerCommand` または `IUnknown` のみをサポートし、それ以外は `E_NOINTERFACE` を返す。
unsafe extern "system" fn explorer_command_query_interface(
    this: *mut c_void,
    riid: *const windows::core::GUID,
    ppv: *mut *mut c_void,
) -> windows::core::HRESULT {
    if riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }

    let object = this as *mut ExplorerCommandObject;
    unsafe {
        *ppv = ptr::null_mut();
    }

    let iid = unsafe { &*riid };
    if *iid == IExplorerCommand::IID || *iid == IUnknown::IID {
        unsafe {
            (*object).ref_count.fetch_add(1, Ordering::Relaxed);
        }
        unsafe {
            *ppv = this;
        }
        S_OK
    } else {
        E_NOINTERFACE
    }
}

/// `IUnknown::AddRef` の実装。参照カウントをインクリメントして新しい値を返す。
unsafe extern "system" fn explorer_command_add_ref(this: *mut c_void) -> u32 {
    let object = this as *mut ExplorerCommandObject;
    unsafe { (*object).ref_count.fetch_add(1, Ordering::Relaxed) + 1 }
}

/// `IUnknown::Release` の実装。参照カウントをデクリメントし、ゼロになった場合にオブジェクトを解放する。
unsafe extern "system" fn explorer_command_release(this: *mut c_void) -> u32 {
    let object = this as *mut ExplorerCommandObject;
    let count = unsafe { (*object).ref_count.fetch_sub(1, Ordering::Release) - 1 };
    if count == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(object));
        }
        GLOBAL_OBJECT_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
    count
}

/// `IExplorerCommand::GetTitle` の実装。メニューに表示するコマンド名を返す。
unsafe extern "system" fn explorer_command_get_title(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    ppszname: *mut PWSTR,
) -> windows::core::HRESULT {
    if ppszname.is_null() {
        return E_POINTER;
    }
    match allocate_pwstr("nanai-txt-viewer") {
        Ok(ptr) => {
            unsafe {
                *ppszname = ptr;
            }
            S_OK
        }
        Err(err) => err.code(),
    }
}

/// `IExplorerCommand::GetIcon` の実装。アイコンは未実装のため `E_NOTIMPL` を返す。
unsafe extern "system" fn explorer_command_get_icon(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    ppszicon: *mut PWSTR,
) -> windows::core::HRESULT {
    if ppszicon.is_null() {
        return E_POINTER;
    }
    unsafe {
        *ppszicon = PWSTR::default();
    }
    E_NOTIMPL
}

/// `IExplorerCommand::GetToolTip` の実装。ツールチップ文字列を返す。
unsafe extern "system" fn explorer_command_get_tooltip(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    ppztip: *mut PWSTR,
) -> windows::core::HRESULT {
    if ppztip.is_null() {
        return E_POINTER;
    }
    match allocate_pwstr("Open with nanai-txt-viewer") {
        Ok(ptr) => {
            unsafe {
                *ppztip = ptr;
            }
            S_OK
        }
        Err(err) => err.code(),
    }
}

/// `IExplorerCommand::GetCanonicalName` の実装。このコマンドのCLSIDを返す。
unsafe extern "system" fn explorer_command_get_canonical_name(
    _this: *mut c_void,
    pguid: *mut windows::core::GUID,
) -> windows::core::HRESULT {
    if pguid.is_null() {
        return E_POINTER;
    }
    unsafe {
        *pguid = CLSID_EXPLORER_COMMAND;
    }
    S_OK
}

/// `IExplorerCommand::GetState` の実装。コマンドを常に有効（有効状態 = 0）として返す。
unsafe extern "system" fn explorer_command_get_state(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    _foktobeslow: windows::core::BOOL,
    pstate: *mut u32,
) -> windows::core::HRESULT {
    if pstate.is_null() {
        return E_POINTER;
    }
    unsafe {
        *pstate = 0;
    }
    S_OK
}

/// `IExplorerCommand::Invoke` の実装。
/// 選択されたファイルのパスを取得し、ビューアアプリを起動する。
fn get_viewer_executable_path() -> Option<PathBuf> {
    if let Ok(package) = Package::Current() {
        if let Ok(installed_location) = package.InstalledLocation() {
            if let Ok(path) = installed_location.Path() {
                let install_path = path.to_string_lossy();
                let exe_path =
                    PathBuf::from(install_path.to_owned()).join("nanai-shiftjis-reader.exe");
                if exe_path.exists() {
                    return Some(exe_path);
                }
            }
        }
    }
    None
}

fn invoke_viewer_for_path(path: PathBuf) -> windows::core::HRESULT {
    let exe_path =
        get_viewer_executable_path().unwrap_or_else(|| PathBuf::from("nanai-shiftjis-reader.exe"));
    let result = Command::new(exe_path).arg(path).spawn();
    if result.is_ok() { S_OK } else { E_FAIL }
}

unsafe extern "system" fn explorer_command_invoke(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    _pbc: *mut c_void,
) -> windows::core::HRESULT {
    let path = match unsafe { get_selected_file_path(_psiitemarray) } {
        Some(path) => path,
        None => return S_OK,
    };

    invoke_viewer_for_path(path)
}

/// `IExplorerCommand::GetFlags` の実装。フラグは設定しない（0を返す）。
unsafe extern "system" fn explorer_command_get_flags(
    _this: *mut c_void,
    pflags: *mut u32,
) -> windows::core::HRESULT {
    if pflags.is_null() {
        return E_POINTER;
    }
    unsafe {
        *pflags = 0;
    }
    S_OK
}

/// `IExplorerCommand::EnumSubCommands` の実装。サブコマンドは持たないため `E_NOTIMPL` を返す。
unsafe extern "system" fn explorer_command_enum_sub_commands(
    _this: *mut c_void,
    ppenum: *mut *mut c_void,
) -> windows::core::HRESULT {
    if ppenum.is_null() {
        return E_POINTER;
    }
    unsafe {
        *ppenum = ptr::null_mut();
    }
    E_NOTIMPL
}

/// `IExplorerCommand` の静的vtable。各関数ポインタを上記の実装関数に設定する。
static EXPLORER_COMMAND_VTBL: IExplorerCommand_Vtbl = IExplorerCommand_Vtbl {
    base__: IUnknown_Vtbl {
        QueryInterface: explorer_command_query_interface,
        AddRef: explorer_command_add_ref,
        Release: explorer_command_release,
    },
    GetTitle: explorer_command_get_title,
    GetIcon: explorer_command_get_icon,
    GetToolTip: explorer_command_get_tooltip,
    GetCanonicalName: explorer_command_get_canonical_name,
    GetState: explorer_command_get_state,
    Invoke: explorer_command_invoke,
    GetFlags: explorer_command_get_flags,
    EnumSubCommands: explorer_command_enum_sub_commands,
};

/// `ExplorerCommandObject` をヒープに確保してvoidポインタとして返す。
/// 参照カウントは1で初期化され、`GLOBAL_OBJECT_COUNT` をインクリメントする。
pub(super) unsafe fn create_explorer_command() -> *mut std::ffi::c_void {
    GLOBAL_OBJECT_COUNT.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(ExplorerCommandObject {
        lp_vtbl: &EXPLORER_COMMAND_VTBL,
        ref_count: AtomicU32::new(1),
    })) as *mut std::ffi::c_void
}
