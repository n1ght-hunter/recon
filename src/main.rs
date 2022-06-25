use futures::executor::block_on;
use gui::MediaControl;
use iced::{pure::Application, Settings};
use key_watcher::listen;
use media_listener::storage_control::load_media;

pub mod gui;
pub mod key_watcher;
pub mod media_controls;
pub mod media_listener;
pub mod settings;

async fn async_main() {
    let result = listen();
    if result.is_err() {
        println!("error listening");
    }
    load_media();
}
pub fn main() {
    block_on(async_main());
    MediaControl::run(Settings::default()).expect("error running media control");
}
