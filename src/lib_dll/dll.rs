#![cfg(windows)]

use super::class_factory::create_class_factory;
use super::utils::CLSID_EXPLORER_COMMAND;
use super::{GLOBAL_LOCK_COUNT, GLOBAL_OBJECT_COUNT};
use std::ffi::c_void;
use std::sync::atomic::Ordering;
use windows::{
    Win32::{
        Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_NOINTERFACE, E_POINTER, S_FALSE, S_OK},
        System::Com::IClassFactory,
    },
    core::{GUID, HRESULT, Interface},
};

#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if GLOBAL_OBJECT_COUNT.load(Ordering::Relaxed) == 0
        && GLOBAL_LOCK_COUNT.load(Ordering::Relaxed) == 0
    {
        S_OK
    } else {
        S_FALSE
    }
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
        if *iid == IClassFactory::IID || *iid == windows::core::IUnknown::IID {
            *ppv = factory as *mut c_void;
            S_OK
        } else {
            std::sync::atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(
                factory as *mut super::class_factory::ExplorerClassFactoryObject,
            ));
            E_NOINTERFACE
        }
    }
}
