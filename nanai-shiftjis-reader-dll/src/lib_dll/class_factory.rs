#![cfg(windows)]

use super::GLOBAL_OBJECT_COUNT;
use super::explorer_command::create_explorer_command;
use std::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
use windows::{
    Win32::{
        Foundation::{CLASS_E_NOAGGREGATION, E_NOINTERFACE, E_POINTER, S_OK},
        System::Com::{IClassFactory, IClassFactory_Vtbl},
        UI::Shell::IExplorerCommand,
    },
    core::{IUnknown, IUnknown_Vtbl, Interface},
};

/// `IClassFactory` を実装するCOMクラスファクトリオブジェクト。
/// `ExplorerCommandObject` の生成を担当する。
#[repr(C)]
pub(super) struct ExplorerClassFactoryObject {
    lp_vtbl: *const IClassFactory_Vtbl,
    ref_count: AtomicU32,
}

/// `this` ポインタを `ExplorerClassFactoryObject` にキャストするヘルパー。
fn factory_from_this(this: *mut c_void) -> *mut ExplorerClassFactoryObject {
    this as *mut ExplorerClassFactoryObject
}

/// `IUnknown::QueryInterface` の実装。
/// `IClassFactory` または `IUnknown` の要求に対してインターフェースを提供する。
unsafe extern "system" fn class_factory_query_interface(
    this: *mut c_void,
    riid: *const windows::core::GUID,
    ppv: *mut *mut c_void,
) -> windows::core::HRESULT {
    if riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }

    let object = factory_from_this(this);
    unsafe {
        *ppv = ptr::null_mut();
    }

    let iid = unsafe { &*riid };
    if *iid == IClassFactory::IID || *iid == IUnknown::IID {
        unsafe {
            (*object).ref_count.fetch_add(1, Ordering::Relaxed);
            *ppv = this;
        }
        S_OK
    } else {
        E_NOINTERFACE
    }
}

/// `IUnknown::AddRef` の実装。
/// 参照カウントを増加させて、新しい参照数を返す。
unsafe extern "system" fn class_factory_add_ref(this: *mut c_void) -> u32 {
    let object = factory_from_this(this);
    unsafe { (*object).ref_count.fetch_add(1, Ordering::Relaxed) + 1 }
}

/// `IUnknown::Release` の実装。
/// 参照カウントを減少させ、ゼロになればオブジェクトを解放する。
unsafe extern "system" fn class_factory_release(this: *mut c_void) -> u32 {
    let object = factory_from_this(this);
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

/// `IClassFactory::CreateInstance` の実装。
/// 集約はサポートせず、`IExplorerCommand` または `IUnknown` を返す。
unsafe extern "system" fn class_factory_create_instance(
    _this: *mut c_void,
    punkouter: *mut c_void,
    riid: *const windows::core::GUID,
    ppvobject: *mut *mut c_void,
) -> windows::core::HRESULT {
    if !punkouter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    if riid.is_null() || ppvobject.is_null() {
        return E_POINTER;
    }

    unsafe {
        *ppvobject = ptr::null_mut();
    }
    let command = create_explorer_command();
    let iid = unsafe { &*riid };

    if *iid == IExplorerCommand::IID || *iid == IUnknown::IID {
        unsafe {
            *ppvobject = command;
        }
        S_OK
    } else {
        GLOBAL_OBJECT_COUNT.fetch_sub(1, Ordering::Relaxed);
        unsafe {
            drop(Box::from_raw(
                command as *mut super::explorer_command::ExplorerCommandObject,
            ));
        }
        E_NOINTERFACE
    }
}

/// `IClassFactory::LockServer` の実装。
/// DLL のアンロードを制御するためのサーバーロックを増減する。
unsafe extern "system" fn class_factory_lock_server(
    _this: *mut c_void,
    flock: windows::core::BOOL,
) -> windows::core::HRESULT {
    if flock.as_bool() {
        super::GLOBAL_LOCK_COUNT.fetch_add(1, Ordering::Relaxed);
    } else {
        super::GLOBAL_LOCK_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
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

/// COM クラスファクトリを生成し、参照カウントを初期化する。
/// `GLOBAL_OBJECT_COUNT` をインクリメントして、DLL のアンロードを制御する。
pub(super) fn create_class_factory() -> *mut std::ffi::c_void {
    GLOBAL_OBJECT_COUNT.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(Box::new(ExplorerClassFactoryObject {
        lp_vtbl: &CLASS_FACTORY_VTBL,
        ref_count: AtomicU32::new(1),
    })) as *mut std::ffi::c_void
}
