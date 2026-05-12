#![cfg(windows)]

use super::GLOBAL_OBJECT_COUNT;
use super::guid::CLSID_EXPLORER_COMMAND;
use super::utils::{allocate_pwstr, get_selected_file_paths};
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

/// `this` ポインタを `ExplorerCommandObject` にキャストするヘルパー。
fn command_from_this(this: *mut c_void) -> *mut ExplorerCommandObject {
    this as *mut ExplorerCommandObject
}

/// `IUnknown::QueryInterface` の実装。
/// `IExplorerCommand` または `IUnknown` に対してポインタを返す。
unsafe extern "system" fn explorer_command_query_interface(
    this: *mut c_void,
    riid: *const windows::core::GUID,
    ppv: *mut *mut c_void,
) -> windows::core::HRESULT {
    if riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }

    let object = command_from_this(this);
    unsafe {
        *ppv = ptr::null_mut();
    }

    let iid = unsafe { &*riid };
    if *iid == IExplorerCommand::IID || *iid == IUnknown::IID {
        unsafe {
            (*object).ref_count.fetch_add(1, Ordering::Relaxed);
            *ppv = this;
        }
        S_OK
    } else {
        E_NOINTERFACE
    }
}

unsafe extern "system" fn explorer_command_add_ref(this: *mut c_void) -> u32 {
    let object = command_from_this(this);
    unsafe { (*object).ref_count.fetch_add(1, Ordering::Relaxed) + 1 }
}

unsafe extern "system" fn explorer_command_release(this: *mut c_void) -> u32 {
    let object = command_from_this(this);
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

/// `IExplorerCommand::GetTitle` の実装。
/// コンテキストメニューに表示するコマンド名を返す。
unsafe extern "system" fn explorer_command_get_title(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    ppszname: *mut PWSTR,
) -> windows::core::HRESULT {
    if ppszname.is_null() {
        return E_POINTER;
    }

    match allocate_pwstr("Open with nanai-shiftjis-reader") {
        Ok(ptr) => {
            unsafe {
                *ppszname = ptr;
            }
            S_OK
        }
        Err(err) => err.code(),
    }
}

/// `IExplorerCommand::GetIcon` の実装。
/// パッケージ内実行ファイルのアイコンを返す。
unsafe extern "system" fn explorer_command_get_icon(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    ppszicon: *mut PWSTR,
) -> windows::core::HRESULT {
    if ppszicon.is_null() {
        return E_POINTER;
    }

    if let Some(icon_spec) = get_viewer_icon_spec() {
        match allocate_pwstr(&icon_spec) {
            Ok(ptr) => {
                unsafe {
                    *ppszicon = ptr;
                }
                return S_OK;
            }
            Err(err) => return err.code(),
        }
    }

    unsafe {
        *ppszicon = PWSTR::default();
    }
    E_NOTIMPL
}

/// `IExplorerCommand::GetToolTip` の実装。
/// コンテキストメニューのツールチップ文字列を返す。
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

/// `IExplorerCommand::GetState` の実装。
/// コマンドを常に有効状態として返す。
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

/// パッケージインストール内の実行可能ファイルパスを取得する。
/// 見つからない場合は `None` を返す。
fn get_viewer_executable_path() -> Option<PathBuf> {
    let package = Package::Current().ok()?;
    let installed_location = package.InstalledLocation().ok()?;
    let path = installed_location.Path().ok()?;
    let exe_path =
        PathBuf::from(path.to_string_lossy().to_owned()).join("nanai-shiftjis-reader.exe");

    exe_path.exists().then_some(exe_path)
}

/// パッケージ内の実行可能ファイルのアイコン文字列を返す。
fn get_viewer_icon_spec() -> Option<String> {
    get_viewer_executable_path()
        .and_then(|exe_path| exe_path.to_str().map(|path| format!("{},0", path)))
}

/// 指定パスのファイルをビューアで開く。
/// 実行ファイルの起動に失敗した場合は `E_FAIL` を返す。
fn invoke_viewer_for_paths(paths: &[PathBuf]) -> windows::core::HRESULT {
    let exe_path =
        get_viewer_executable_path().unwrap_or_else(|| PathBuf::from("nanai-shiftjis-reader.exe"));
    let mut command = Command::new(exe_path);
    command.args(paths.iter().map(|path| path.as_os_str()));

    command.spawn().map(|_| S_OK).unwrap_or(E_FAIL)
}

/// `IExplorerCommand::Invoke` の実装。
/// 選択されたファイルを取得してビューアを起動する。
unsafe extern "system" fn explorer_command_invoke(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    _pbc: *mut c_void,
) -> windows::core::HRESULT {
    let paths = unsafe { get_selected_file_paths(_psiitemarray) };
    if paths.is_empty() {
        return E_FAIL;
    }

    invoke_viewer_for_paths(&paths)
}

/// `IExplorerCommand::GetFlags` の実装。
/// フラグを使用しないため、0 を返す。
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

/// `IExplorerCommand::EnumSubCommands` の実装。
/// サブコマンドを持たないため `E_NOTIMPL` を返す。
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

/// `ExplorerCommandObject` をヒープに確保して返す。
/// 参照カウントは1で初期化され、`GLOBAL_OBJECT_COUNT` を更新する。
pub(super) fn create_explorer_command() -> *mut std::ffi::c_void {
    GLOBAL_OBJECT_COUNT.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(ExplorerCommandObject {
        lp_vtbl: &EXPLORER_COMMAND_VTBL,
        ref_count: AtomicU32::new(1),
    })) as *mut std::ffi::c_void
}
