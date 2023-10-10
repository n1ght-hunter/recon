mod common;
mod keycodes;
pub mod rdev;
use core::i32;
use std::ptr::null_mut;
use uuid::Uuid;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM},
    UI::WindowsAndMessaging::{CallNextHookEx, GetMessageA, GetMessageW, HC_ACTION},
};

use self::{
    common::{convert, set_key_hook, HOOK},
    rdev::{EventType, Key},
};

type CallBack = Box<dyn FnMut()>;

pub struct KeyWatcher {
    pub key: String,
    pub call_back: CallBack,
    pub keys: Vec<Key>,
}

impl KeyWatcher {
    pub fn new(call_back: CallBack, keys: Vec<Key>) -> Self {
        Self {
            key: Uuid::new_v4().to_string(),
            call_back,
            keys,
        }
    }
}

pub static mut GLOBAL_CALLBACK: Vec<KeyWatcher> = Vec::new();
pub static mut CURRENT_KEYS_PRESSED: Vec<Key> = Vec::new();

pub fn listen() -> Result<(), windows::core::Error> {
    unsafe {
        println!("listner");
        let value = set_key_hook(raw_callback);
        print!("listner2");
        value
    }
}


pub fn subscribe(watcher: KeyWatcher) -> Result<String, ()> {
    let key = watcher.key.clone();
    unsafe {
        GLOBAL_CALLBACK.push(watcher);
    }
    Ok(key)
}

fn unsub(key: String) -> Result<(), ()> {
    unsafe {
        GLOBAL_CALLBACK.remove(
            GLOBAL_CALLBACK
                .iter()
                .position(|x| &x.key == &key)
                .expect("error removing callback"),
        );
    }
    Ok(())
}

unsafe extern "system" fn raw_callback(
    code: core::primitive::i32,
    param: WPARAM,
    lpdata: LPARAM,
) -> LRESULT {
    println!("callback");
    // let action: i32 = HC_ACTION.try_into().unwrap();
    // if code == action {
    //     let (opt, _name) = convert(param, lpdata);
    //     if let Some(event_type) = opt {
    //         println!("{:?}", event_type);
    //         match event_type {
    //             EventType::KeyPress(key) => {
    //                 if !CURRENT_KEYS_PRESSED.contains(&key) {
    //                     CURRENT_KEYS_PRESSED.push(key);
    //                 }
    //             }
    //             EventType::KeyRelease(key) => {
    //                 let index = CURRENT_KEYS_PRESSED.iter().position(|x| *x == key);
    //                 if index.is_some() {
    //                     CURRENT_KEYS_PRESSED.remove(index.unwrap());
    //                 }
    //             }
    //         }
    //     }
    // }
    // run_callbacks();
    CallNextHookEx(HOOK, code, param, lpdata)
}

fn run_callbacks() {
    unsafe {
        for callback in &mut GLOBAL_CALLBACK {
            if callback
                .keys
                .iter()
                .all(|key| CURRENT_KEYS_PRESSED.contains(key))
            {
                (callback.call_back)();
            }
        }
    }
}
