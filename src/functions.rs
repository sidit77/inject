use once_cell::sync::Lazy;
use windows::{s, w};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Threading::LPTHREAD_START_ROUTINE;

static KERNEL32: Lazy<HMODULE> = Lazy::new(|| unsafe {
   GetModuleHandleW(w!("kernel32.dll"))
       .expect("Failed to load kernel32")
});

pub static LOAD_LIBRARY_W: Lazy<LPTHREAD_START_ROUTINE> = Lazy::new(|| unsafe {
   let func = GetProcAddress(*KERNEL32, s!("LoadLibraryW"))
       .expect("Failed to get the address of LoadLibraryW");
   std::mem::transmute(func)
});