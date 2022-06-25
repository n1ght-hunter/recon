use serde::{Serialize, Deserialize};


use crate::{
    key_watcher::rdev::Key,
    media_controls::controller::{app_instance, application_task, Controls},
};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HotKey {
    pub keys: Vec<Key>,
    pub runner: (String, Controls),
}


pub async fn run_media_hotkey(name: Box<String>, control: Box<Controls>) {
    let instance = app_instance(&name).await;
    if instance.is_ok() {
        application_task(instance.unwrap(), &control).await;
    }
}
