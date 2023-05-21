use std::ffi::c_void;
use windows_sys::Win32::Foundation::{BOOL, HMODULE, TRUE};
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

#[no_mangle]
pub unsafe extern "stdcall" fn DllMain(_hmodule: HMODULE, reason: u32, _: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => println!("Attached"),
        DLL_PROCESS_DETACH => println!("Detached"),
        _ => {}
    }
    TRUE
}
