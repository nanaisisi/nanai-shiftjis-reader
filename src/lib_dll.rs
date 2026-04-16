#![cfg(windows)]

use std::{
    ffi::{OsStr, c_void},
    mem,
    os::windows::prelude::OsStrExt,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
use windows::{
    Win32::{
        Foundation::{
            CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_NOINTERFACE, E_NOTIMPL,
            E_OUTOFMEMORY, E_POINTER, S_OK,
        },
        System::Com::{CoTaskMemAlloc, IClassFactory, IClassFactory_Vtbl},
        UI::Shell::{IExplorerCommand, IExplorerCommand_Vtbl},
    },
    core::{self, BOOL, GUID, HRESULT, IUnknown, IUnknown_Vtbl, Interface, PWSTR},
};

const CLSID_EXPLORER_COMMAND: GUID = GUID::from_values(
    0xcc19e147,
    0x7757,
    0x483c,
    [0xb2, 0x7f, 0x3d, 0x81, 0xbc, 0xeb, 0x38, 0xfe],
);

#[repr(C)]
struct ExplorerCommandObject {
    lp_vtbl: *const IExplorerCommand_Vtbl,
    ref_count: AtomicU32,
}

#[repr(C)]
struct ExplorerClassFactoryObject {
    lp_vtbl: *const IClassFactory_Vtbl,
    ref_count: AtomicU32,
}

fn allocate_pwstr(value: &str) -> core::Result<PWSTR> {
    let wide: Vec<u16> = OsStr::new(value).encode_wide().chain(Some(0)).collect();

    let size = wide.len() * mem::size_of::<u16>();
    let raw = unsafe { CoTaskMemAlloc(size) } as *mut u16;
    if raw.is_null() {
        return Err(E_OUTOFMEMORY.into());
    }

    unsafe {
        ptr::copy_nonoverlapping(wide.as_ptr(), raw, wide.len());
    }

    Ok(PWSTR(raw))
}

unsafe extern "system" fn explorer_command_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
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
    }
    count
}

unsafe extern "system" fn explorer_command_get_title(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    ppszname: *mut PWSTR,
) -> HRESULT {
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
) -> HRESULT {
    unsafe {
        *ppszicon = PWSTR::default();
    }
    S_OK
}

unsafe extern "system" fn explorer_command_get_tooltip(
    _this: *mut c_void,
    _psiitemarray: *mut c_void,
    ppztip: *mut PWSTR,
) -> HRESULT {
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
    pguid: *mut GUID,
) -> HRESULT {
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
    _foktobeslow: core::BOOL,
    pstate: *mut u32,
) -> HRESULT {
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
) -> HRESULT {
    S_OK
}

unsafe extern "system" fn explorer_command_get_flags(
    _this: *mut c_void,
    pflags: *mut u32,
) -> HRESULT {
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
) -> HRESULT {
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

unsafe fn create_explorer_command() -> *mut ExplorerCommandObject {
    Box::into_raw(Box::new(ExplorerCommandObject {
        lp_vtbl: &EXPLORER_COMMAND_VTBL,
        ref_count: AtomicU32::new(1),
    }))
}

unsafe extern "system" fn class_factory_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }

    let object = this as *mut ExplorerClassFactoryObject;
    unsafe {
        *ppv = ptr::null_mut();
    }

    let iid = unsafe { &*riid };
    if *iid == IClassFactory::IID || *iid == IUnknown::IID {
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

unsafe extern "system" fn class_factory_add_ref(this: *mut c_void) -> u32 {
    let object = this as *mut ExplorerClassFactoryObject;
    unsafe { (*object).ref_count.fetch_add(1, Ordering::Relaxed) + 1 }
}

unsafe extern "system" fn class_factory_release(this: *mut c_void) -> u32 {
    let object = this as *mut ExplorerClassFactoryObject;
    let count = unsafe { (*object).ref_count.fetch_sub(1, Ordering::Release) - 1 };
    if count == 0 {
        std::sync::atomic::fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(object));
        }
    }
    count
}

unsafe extern "system" fn class_factory_create_instance(
    _this: *mut c_void,
    punkouter: *mut c_void,
    riid: *const GUID,
    ppvobject: *mut *mut c_void,
) -> HRESULT {
    if !punkouter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    if riid.is_null() || ppvobject.is_null() {
        return E_POINTER;
    }

    unsafe {
        *ppvobject = ptr::null_mut();
    }
    let command = unsafe { create_explorer_command() };
    let iid = unsafe { &*riid };

    if *iid == IExplorerCommand::IID || *iid == IUnknown::IID {
        unsafe {
            *ppvobject = command as *mut c_void;
        }
        S_OK
    } else {
        unsafe {
            (*command).ref_count.fetch_sub(1, Ordering::Relaxed);
        }
        E_NOINTERFACE
    }
}

unsafe extern "system" fn class_factory_lock_server(_this: *mut c_void, _flock: BOOL) -> HRESULT {
    S_OK
}

static CLASS_FACTORY_VTBL: IClassFactory_Vtbl = IClassFactory_Vtbl {
    base__: IUnknown_Vtbl {
        QueryInterface: class_factory_query_interface,
        AddRef: class_factory_add_ref,
        Release: class_factory_release,
    },
    CreateInstance: class_factory_create_instance,
    LockServer: class_factory_lock_server,
};

unsafe fn create_class_factory() -> *mut ExplorerClassFactoryObject {
    Box::into_raw(Box::new(ExplorerClassFactoryObject {
        lp_vtbl: &CLASS_FACTORY_VTBL,
        ref_count: AtomicU32::new(1),
    }))
}

#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_OK
}

#[unsafe(no_mangle)]
pub extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        if rclsid.is_null() || riid.is_null() || ppv.is_null() {
            return E_POINTER;
        }

        if *rclsid != CLSID_EXPLORER_COMMAND {
            return CLASS_E_CLASSNOTAVAILABLE;
        }

        let factory = create_class_factory();
        let iid = &*riid;
        if *iid == IClassFactory::IID || *iid == IUnknown::IID {
            *ppv = factory as *mut c_void;
            S_OK
        } else {
            std::sync::atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(factory));
            E_NOINTERFACE
        }
    }
}
