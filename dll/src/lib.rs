use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};
use windows::{Win32::UI::WindowsAndMessaging::MessageBoxA, core::*};

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => detach(),
        _ => (),
    }

    true
}

fn attach() {
    unsafe {
        MessageBoxA(
            None,
            PCSTR::from_raw(c"hello world".as_ptr().cast()),
            s!("hello.dll"),
            Default::default(),
        );
    }
}

fn detach() {}
