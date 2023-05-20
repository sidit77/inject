use std::mem::size_of;
use anyhow::{Context, Result};
use widestring::U16Str;
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};

pub struct ProcessIter {
    snapshot: HANDLE
}

impl ProcessIter {
    pub fn new() -> Result<Self> {
        let snapshot = unsafe {
            log::trace!("Creating a toolhelp snapshot");
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .context("Failed to take snapshot of current process list")?
        };

        Ok(Self { snapshot })
    }
}

impl Drop for ProcessIter {
    fn drop(&mut self) {
        log::trace!("Closing toolhelp snapshot");
        if let Err(err) = unsafe { CloseHandle(self.snapshot).ok() } {
            log::warn!("Failed to close toolhelp snapshot: {}", err);
        }
    }
}

impl Iterator for ProcessIter {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        match unsafe { Process32NextW(self.snapshot, &mut entry).ok() } {
            Ok(()) => Some(Process(entry)),
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
        let len = self.0.szExeFile
            .iter()
            .take_while(|c| **c != 0)
            .count();
        U16Str::from_slice(&self.0.szExeFile[..len])
    }

}