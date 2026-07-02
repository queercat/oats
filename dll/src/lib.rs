use std::mem::transmute;
use std::os::raw::c_void;

use windows::Win32::System::Threading::CreateThread;
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
#[allow(non_snake_case, unused_variables)]
extern "system" fn CheatThread(data: *mut c_void) -> u32 {
    unsafe {
        MessageBoxA(
            None,
            PCSTR::from_raw(c"really it was that easy?".as_ptr().cast()),
            s!("hello.dll"),
            Default::default(),
        );
    }

    0
}

fn attach(dll_module: HINSTANCE) {
    unsafe {
        CreateThread(None, 0, Some(CheatThread), Some(transmute(dll_module)), windows::Win32::System::Threading::THREAD_CREATION_FLAGS(0), None).unwrap();
    }
}

fn detach() {}
