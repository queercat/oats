use std::mem::transmute;
use std::os::raw::c_void;

use windows::Win32::System::Diagnostics::Debug::{
    FlushInstructionCache, ReadProcessMemory, WriteProcessMemory,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtectEx,
};
use windows::Win32::System::Threading::{CreateThread, GetCurrentProcess, THREAD_CREATION_FLAGS};
use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};
use windows::{Win32::UI::WindowsAndMessaging::MessageBoxA, core::*};

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(dll_module),
        DLL_PROCESS_DETACH => detach(),
        _ => (),
    }

    true
}

#[unsafe(no_mangle)]
fn message_box(text: String, title: Option<String>) {
    unsafe {
        MessageBoxA(
            None,
            PCSTR::from_raw(format!("{}\0", text).as_ptr().cast()),
            PCSTR::from_raw(
                format!("{}\0", title.unwrap_or("Info".to_owned()))
                    .as_ptr()
                    .cast(),
            ),
            Default::default(),
        )
    };
}

fn read_memory(handle: HANDLE, address: usize, result: *mut c_void, data_size: usize) {
    unsafe {
        ReadProcessMemory(
            handle,
            std::ptr::with_exposed_provenance::<c_void>(address),
            result,
            data_size,
            None,
        )
        .unwrap();
    }
}

fn write_memory(handle: HANDLE, address: usize, data: *mut c_void, data_size: usize) {
    unsafe {
        let target = std::ptr::with_exposed_provenance::<c_void>(address);

        let mut old_protect = PAGE_PROTECTION_FLAGS(0);
        VirtualProtectEx(
            handle,
            target,
            data_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
        .unwrap();

        WriteProcessMemory(handle, target, data, data_size, None).unwrap();
        FlushInstructionCache(handle, Some(target), data_size).unwrap();

        let mut _restored_protect = PAGE_PROTECTION_FLAGS(0);
        VirtualProtectEx(
            handle,
            target,
            data_size,
            old_protect,
            &mut _restored_protect,
        )
        .unwrap();
    }
}

extern "system" fn cheat_thread(_data: *mut c_void) -> u32 {
    // message_box("fuck".to_string(), Some("me".to_string()));
    let base = unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as usize;
    message_box(format!("{:x}", base + 0xC73EF), None);
    write_memory(
        unsafe { GetCurrentProcess() },
        base + 0xC73EF,
        [0x90u8, 0x90u8].as_mut_ptr().cast(),
        2,
    );
    0
}

fn attach(dll_module: HINSTANCE) {
    unsafe {
        CreateThread(
            None,
            0,
            Some(cheat_thread),
            Some(transmute(dll_module)),
            THREAD_CREATION_FLAGS(0),
            None,
        )
        .unwrap();
    }
}

fn detach() {}
