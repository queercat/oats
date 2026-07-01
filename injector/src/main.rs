use std::{
    ffi::{CStr, CString},
    mem::transmute,
};

use anyhow::anyhow;
use widestring::u16str;
use windows::{
    Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE, MAX_PATH},
        System::{
            Diagnostics::{
                Debug::WriteProcessMemory,
                ToolHelp::{
                    CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next,
                    TH32CS_SNAPPROCESS,
                },
            },
            LibraryLoader::{GetModuleHandleA, GetModuleHandleW, GetProcAddress},
            Memory::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx},
            Threading::{CreateRemoteThread, OpenProcess, PROCESS_ALL_ACCESS, WaitForSingleObject, INFINITE},
        },
    },
    core::{PCSTR, PCWSTR},
};

fn get_pid(process_name: String) -> Option<u32> {
    let mut result: Option<u32> = None;

    unsafe {
        let process_handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let entry: *mut PROCESSENTRY32 = &mut PROCESSENTRY32::default();

        (*entry).dwSize = size_of::<PROCESSENTRY32>() as u32;

        if let Ok(_) = Process32First(process_handle, entry) {
            loop {
                let process_string = CStr::from_ptr((*entry).szExeFile.as_ptr()).to_string_lossy();
                if process_string.to_lowercase() == process_name.to_lowercase() {
                    result = Some((*entry).th32ProcessID);
                    break;
                }

                match Process32Next(process_handle, entry) {
                    Ok(_) => (),
                    _ => break,
                }
            }
        }

        let _ = CloseHandle(process_handle);
    };

    result
}

fn inject_dll(pid: u32, dll_path: String) -> anyhow::Result<()> {
    unsafe {
        let remote_process = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
        let path = VirtualAllocEx(
            remote_process,
            None,
            MAX_PATH as usize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if path.is_null() {
            Err(anyhow!("Unable to to reserve memory in process."))?;
        }

        let dll_path_null = format!("{}\0", dll_path);
        WriteProcessMemory(
            remote_process,
            path,
            dll_path_null.as_ptr().cast(),
            dll_path_null.len(),
            None,
        )?;

        let kernel32 = GetModuleHandleW(PCWSTR::from_raw(u16str!("kernel32.dll\0").as_ptr()))?;
        let load_library = GetProcAddress(kernel32, PCSTR::from_raw("LoadLibraryA\0".as_ptr()))
            .ok_or(anyhow!("Couldn't find LoadLibraryA"))?;

        let remote_thread = CreateRemoteThread(
            remote_process,
            None,
            0,
            transmute(load_library),
            Some(path),
            0,
            None,
        )?;

        if remote_thread == INVALID_HANDLE_VALUE {
            Err(anyhow!("Invalid handle value!"))?
        }

        WaitForSingleObject(remote_thread, INFINITE);
        VirtualFreeEx(remote_process, path, 0, MEM_RELEASE)?;
        CloseHandle(remote_thread)?;
        CloseHandle(remote_process)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let pid = get_pid("notepad.exe".to_string()).ok_or(anyhow!("Couldn't find PID."))?;
    inject_dll(pid, "C:\\Users\\Moon\\Documents\\GitHub\\oats\\target\\debug\\injected.dll".to_string())?;
    
    Ok(())
}
