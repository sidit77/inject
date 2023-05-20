use std::mem::size_of;
use anyhow::{Context, Result};
use widestring::U16Str;
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::core::Result as WinResult;

struct ToolHelpSnapshot(HANDLE);

impl ToolHelpSnapshot {
    fn new(flags: CREATE_TOOLHELP_SNAPSHOT_FLAGS, pid: u32) -> WinResult<Self> {
        log::trace!("Creating a toolhelp snapshot");
        let snapshot = unsafe { CreateToolhelp32Snapshot(flags, pid)? };
        Ok(Self(snapshot))
    }
    fn next_process(&self) -> WinResult<PROCESSENTRY32W> {
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        unsafe { Process32NextW(self.0, &mut entry).ok()?; }
        Ok(entry)
    }
    fn next_module(&self) -> WinResult<MODULEENTRY32W> {
        let mut entry = MODULEENTRY32W {
            dwSize: size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };
        unsafe {
            Module32NextW(self.0, &mut entry).ok()?;
        }
        Ok(entry)
    }
}

impl Drop for ToolHelpSnapshot {
    fn drop(&mut self) {
        log::trace!("Closing toolhelp snapshot");
        if let Err(err) = unsafe { CloseHandle(self.0).ok() } {
            log::warn!("Failed to close toolhelp snapshot: {}", err);
        }
    }
}


pub struct ProcessIter(ToolHelpSnapshot);

impl ProcessIter {
    pub fn new() -> Result<Self> {
        let snapshot = ToolHelpSnapshot::new(TH32CS_SNAPPROCESS, 0)
            .context("Failed to take snapshot of current process list")?;
        Ok(Self(snapshot))
    }
}

impl Iterator for ProcessIter {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next_process() {
            Ok(entry) => Some(Process(entry)),
            Err(err) if err.code() == ERROR_NO_MORE_FILES.into() => None,
            Err(err) => panic!("{}", err)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Process(PROCESSENTRY32W);

impl Process {

    pub fn pid(&self) -> u32 {
        self.0.th32ProcessID
    }

    pub fn name(&self) -> &U16Str {
        make_str(&self.0.szExeFile)
    }

}

pub struct ModuleIter(ToolHelpSnapshot);

impl ModuleIter {
    pub fn new(pid: u32) -> Result<Self> {
        let snapshot = ToolHelpSnapshot::new(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)
            .context("Failed to take snapshot of current module list")?;
        Ok(Self(snapshot))
    }
}

impl Iterator for ModuleIter {
    type Item = Module;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next_module() {
            Ok(entry) => Some(Module(entry)),
            Err(err) if err.code() == ERROR_NO_MORE_FILES.into() => None,
            Err(err) => panic!("{}", err)
        }
    }
}

pub struct Module(MODULEENTRY32W);

impl Module {

    pub fn name(&self) -> &U16Str {
        make_str(&self.0.szModule)
    }

}


fn make_str(data: &[u16]) -> &U16Str {
    let len = data
        .iter()
        .take_while(|c| **c != 0)
        .count();
    U16Str::from_slice(&data[..len])
}