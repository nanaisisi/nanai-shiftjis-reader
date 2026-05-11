#![cfg(windows)]

use super::class_factory::create_class_factory;
use super::guid::CLSID_EXPLORER_COMMAND;
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

/// DLLがアンロード可能かどうかをCOMランタイムに通知する。
/// オブジェクト数とロック数が両方ゼロのときのみ `S_OK` を返してアンロードを許可する。
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

/// 指定されたCLSIDに対応するクラスファクトリオブジェクトを取得するCOMエクスポート関数。
/// `CLSID_EXPLORER_COMMAND` 以外のCLSIDには `CLASS_E_CLASSNOTAVAILABLE` を返す。
/// 要求されたインターフェース（`IClassFactory` または `IUnknown`）が一致しない場合は
/// `E_NOINTERFACE` を返し、作成したファクトリオブジェクトを破棄する。
///
/// # Safety
///
/// The caller must ensure that `rclsid`, `riid`, and `ppv` are valid, non-null pointers.
/// Dereferencing and writing through these raw pointers is only safe when the provided
/// pointer values are valid COM arguments from the caller.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
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
            *ppv = factory;
            S_OK
        } else {
            GLOBAL_OBJECT_COUNT.fetch_sub(1, Ordering::Relaxed);
            std::sync::atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(
                factory as *mut super::class_factory::ExplorerClassFactoryObject,
            ));
            E_NOINTERFACE
        }
    }
}
