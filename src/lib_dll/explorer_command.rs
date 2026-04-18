#![cfg(windows)]

use super::GLOBAL_OBJECT_COUNT;
use super::utils::{
    CLSID_EXPLORER_COMMAND, allocate_pwstr, app_executable_path, get_selected_file_path,
    launch_with_viewer,
};
use std::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
use windows::{
    Win32::{
        Foundation::{E_NOINTERFACE, E_NOTIMPL, E_POINTER, S_OK},
        UI::Shell::{IExplorerCommand, IExplorerCommand_Vtbl},
    },
    core::{IUnknown, IUnknown_Vtbl, Interface, PWSTR},
};

#[repr(C)]
pub(super) struct ExplorerCommandObject {
    lp_vtbl: *const IExplorerCommand_Vtbl,
    ref_count: AtomicU32,
}

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

unsafe extern "system" fn explorer_command_add_ref(this: *mut c_void) -> u32 {
    let object = this as *mut ExplorerCommandObject;
    unsafe { (*object).ref_count.fetch_add(1, Ordering::Relaxed) + 1 }
}

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

unsafe extern "system" fn explorer_command_invoke(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    _pbc: *mut c_void,
) -> windows::core::HRESULT {
    let file_path = match unsafe { get_selected_file_path(_psiitemarray) } {
        Some(path) => path,
        None => return S_OK,
    };

    if let Some(exe_path) = unsafe { app_executable_path() } {
        if unsafe { launch_with_viewer(&exe_path, &file_path) } {
            return S_OK;
        }
    }

    S_OK
}

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

pub(super) unsafe fn create_explorer_command() -> *mut std::ffi::c_void {
    GLOBAL_OBJECT_COUNT.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(ExplorerCommandObject {
        lp_vtbl: &EXPLORER_COMMAND_VTBL,
        ref_count: AtomicU32::new(1),
    })) as *mut std::ffi::c_void
}
