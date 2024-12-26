pub mod audio_router;
pub mod gui;
pub mod key_watcher;
pub mod media_controls;
pub mod media_listener;
pub mod settings;
use std::{sync::mpsc, time::Duration};

use audio_router::audio_router;
use futures::executor::block_on;
// use gui::MediaControl;
use iced::{Application, Settings};
use key_watcher::listen;
use media_listener::storage_control::load_media;



#[tokio::main]
async fn main() {
    audio_router()
    // let (tx, rx) = mpsc::channel();
    // let result = listen();
    // if result.is_err() {
    //     println!("error listening");
    // }
    // load_media();
    // test();
    // (|| {
    //     let result = listen();
    //     if result.is_err() 
    //         println!("error listening");
    //     }
    // });
    // for _received in rx {
    //     print!("test");
    // }
    // println!("");
    // MediaControl::run(Settings::default()).expect("error running media control");
}
