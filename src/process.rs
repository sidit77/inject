use std::ffi::c_void;
use std::mem::size_of_val;

use anyhow::{ensure, Context, Error, Result};
use bytemuck::{cast_slice, Pod};
use windows::core::Error as WinError;
use windows::Win32::Foundation::{CloseHandle, FALSE, HANDLE, WAIT_FAILED, WAIT_OBJECT_0};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE};
use windows::Win32::System::Threading::*;

pub struct ProcessHandle(HANDLE);

impl ProcessHandle {
    pub fn open(pid: u32) -> Result<Self> {
        log::trace!("Trying to open process with pid {}", pid);
        let handle = unsafe {
            OpenProcess(
                PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ,
                FALSE,
                pid
            )
            .context("Failed to open process")?
        };
        log::debug!("Process handle is 0x{:x}", handle.0);
        Ok(Self(handle))
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        log::trace!("Closing process handle");
        unsafe {
            CloseHandle(self.0)
                .ok()
                .unwrap_or_else(|err| log::warn!("Failed to close process handle: {}", err))
        }
    }
}

pub struct ProcessMemory<'a> {
    ptr: *mut c_void,
    len: usize,
    process: &'a ProcessHandle
}

impl<'a> ProcessMemory<'a> {
    pub fn new<T: Pod>(process: &'a ProcessHandle, data: &[T]) -> Result<Self> {
        let memory = Self::alloc(process, size_of_val(data))?;
        memory.write(data)?;
        Ok(memory)
    }

    pub fn alloc(process: &'a ProcessHandle, size: usize) -> Result<Self> {
        log::trace!("Attempting to allocate {} bytes", size);

        let ptr = unsafe { VirtualAllocEx(process.0, None, size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };
        ensure!(
            !ptr.is_null(),
            Error::new(WinError::from_win32()).context("Failed to allocate process memory")
        );

        Ok(Self { ptr, len: size, process })
    }

    pub fn write<T: Pod>(&self, data: &[T]) -> Result<()> {
        let bytes: &[u8] = cast_slice(data);
        log::trace!("Attempting to write {} bytes to process memory", bytes.len());
        ensure!(bytes.len() <= self.len, "Too many elements");
        unsafe {
            WriteProcessMemory(self.process.0, self.ptr, bytes.as_ptr() as _, bytes.len(), None)
                .ok()
                .context("Failed to write process memory")?;
        }
        Ok(())
    }

    pub fn to_ptr(&self) -> *const c_void {
        self.ptr
    }
}

impl<'a> Drop for ProcessMemory<'a> {
    fn drop(&mut self) {
        log::trace!("Freeing process memory");
        unsafe {
            VirtualFreeEx(self.process.0, self.ptr, 0, MEM_RELEASE)
                .ok()
                .unwrap_or_else(|err| log::warn!("Failed to free process memory: {}", err))
        };
    }
}

pub struct ProcessThread(HANDLE);

impl ProcessThread {
    pub fn spawn(process: &ProcessHandle, func: LPTHREAD_START_ROUTINE, data: Option<*const c_void>) -> Result<Self> {
        log::trace!("Trying to spawn remote thread");
        let mut thread_id = 0;
        let handle = unsafe {
            CreateRemoteThread(process.0, None, 0, func, data, 0, Some(&mut thread_id))
                .ok()
                .context("Failed to spawn remote thread")?
        };
        log::debug!("Remote thread id is 0x{:x}", thread_id);
        Ok(Self(handle))
    }

    pub fn join(self) -> Result<u32> {
        unsafe {
            match WaitForSingleObject(self.0, INFINITE) {
                WAIT_OBJECT_0 => {
                    let mut exit_code = 0;
                    GetExitCodeThread(self.0, &mut exit_code)
                        .ok()
                        .context("Failed to get exit code for remote thread")?;
                    Ok(exit_code)
                }
                WAIT_FAILED => Err(WinError::from_win32().into()),
                _ => unreachable!()
            }
        }
    }
}

impl Drop for ProcessThread {
    fn drop(&mut self) {
        log::trace!("Closing remote thread handle");
        unsafe {
            CloseHandle(self.0)
                .ok()
                .unwrap_or_else(|err| log::warn!("Failed to close remote thread handle: {}", err))
        }
    }
}
