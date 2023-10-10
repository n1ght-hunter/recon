use std::ptr;

use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM},
    UI::WindowsAndMessaging::{
        GetMessageW, SetWindowsHookExA, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN,
        WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

use super::{
    keycodes::key_from_code,
    rdev::{EventType, Key},
};

pub static mut HOOK: HHOOK = HHOOK(0);

pub unsafe fn get_code(lpdata: LPARAM) -> u32 {
    let kb: KBDLLHOOKSTRUCT = *(lpdata.0 as *const KBDLLHOOKSTRUCT);
    kb.vkCode
}

// pub unsafe fn get_button_code(lpdata: LPARAM) {
//     let mouse = *(lpdata.0 as *const MSLLHOOKSTRUCT);
//     match MOUSEHOOKSTRUCTEX_MOUSE_DATA(HIWORD(mouse.mouseData.0)) {
//         XBUTTON1 => println!("XBUTTON1 {:?}", XBUTTON1),
//         XBUTTON2 => println!("XBUTTON2 {:?}", XBUTTON2),
//         _ => println!("random {:?}", mouse.mouseData),
//     }

// }

// pub unsafe fn get_scan_code(lpdata: LPARAM) -> u32 {
//     let kb = *(lpdata.0 as *const KBDLLHOOKSTRUCT);
//     kb.scanCode
// }

// #[inline]
// pub fn HIWORD(l: u32) -> u32 {
//     ((l >> 16) & 0xffff) as u32
// }

pub unsafe fn convert(param: WPARAM, lpdata: LPARAM) -> (Option<EventType>, Key) {
    let test = param.0 as u32;
    match test {
        WM_KEYDOWN => {
            let code = get_code(lpdata);
            let key = key_from_code(code);
            (Some(EventType::KeyPress(key)), key)
        }
        WM_KEYUP => {
            let code = get_code(lpdata);
            let key = key_from_code(code);
            (Some(EventType::KeyRelease(key)), key)
        }
        WM_SYSKEYDOWN => {
            let code = get_code(lpdata);
            let key = key_from_code(code);
            (Some(EventType::KeyPress(key)), key)
        }
        WM_SYSKEYUP => {
            let code = get_code(lpdata);
            let key = key_from_code(code);
            (Some(EventType::KeyPress(key)), key)
        }
        // WM_XBUTTONDOWN => {
        //     get_button_code(lpdata);
        //     // (
        //     //     Some(EventType::ButtonPress(Button::Unknown(code))),
        //     //     Test::Code(code),
        //     // )
        // }
        // WM_XBUTTONUP => {
        //     get_button_code(lpdata);
        //     // (
        //     //     Some(EventType::ButtonRelease(Button::Unknown(code))),
        //     //     Test::Code(code),
        //     // )
        // }
        _ => {
            println!("{:?}", test);
            (None, Key::Unknown(0))
        }
    }
}

type RawCallback = unsafe extern "system" fn(code: i32, param: WPARAM, lpdata: LPARAM) -> LRESULT;

pub unsafe fn set_key_hook(callback: RawCallback) -> Result<(), windows::core::Error> {
    let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(callback), HINSTANCE(0), 0);
    // let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(callback), HINSTANCE(0), 0);

    if let Err(error) = hook {
        return Err(error);
    }
    HOOK = hook.unwrap();
    GetMessageW(ptr::null_mut(), HWND(0), 0, 0);
    println!("test");
    Ok(())
}
