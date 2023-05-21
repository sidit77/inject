use std::ffi::c_void;
use std::mem::size_of_val;
use anyhow::{Context, ensure, Error, Result};
use bytemuck::{cast_slice, Pod};
use windows::core::Error as WinError;
use windows::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Memory::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx};
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
            ).context("Failed to open process")?
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
        let memory = Self::alloc(process,size_of_val(data))?;
        memory.write(data)?;
        Ok(memory)
    }

    pub fn alloc(process: &'a ProcessHandle, size: usize) -> Result<Self> {
        log::trace!("Attempting to allocate {} bytes", size);

        let ptr = unsafe {
            VirtualAllocEx(
                process.0,
                None,
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE
            )
        };
        ensure!(!ptr.is_null(), Error::new(WinError::from_win32()).context("Failed to allocate process memory"));

        Ok(Self {
            ptr,
            len: size,
            process
        })
    }

    pub fn write<T: Pod>(&self, data: &[T]) -> Result<()> {
        let bytes: &[u8] = cast_slice(data);
        log::trace!("Attempting to write {} bytes to process memory", bytes.len());
        ensure!(bytes.len() <= self.len, "Too many elements");
        unsafe {
            WriteProcessMemory(
                self.process.0,
                self.ptr,
                bytes.as_ptr() as _,
                bytes.len(),
                None
            ).ok().context("Failed to write process memory")?;
        }
        Ok(())
    }

}

impl<'a> Drop for ProcessMemory<'a> {
    fn drop(&mut self) {
        log::trace!("Freeing process memory");
        unsafe {
            VirtualFreeEx(
                self.process.0,
                self.ptr,
                0,
                MEM_RELEASE
            ).ok().unwrap_or_else(|err| log::warn!("Failed to free process memory: {}", err))
        };
    }
}