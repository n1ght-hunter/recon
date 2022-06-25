use std::{collections::HashMap, str::FromStr};

use futures::executor::block_on;
use serde::{Deserialize, Serialize};

use crate::{
    key_watcher::{rdev::Key, subscribe, KeyWatcher},
    media_controls::controller::Controls,
    media_listener::lib::run_media_hotkey,
    settings::{load_file, save_file},
};

pub type MediaAction = String;

pub type MediaSource = String;

pub type MediaHotkey = HashMap<MediaSource, HashMap<MediaAction, Vec<Key>>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaHotkeys {
    pub media_hot_keys: MediaHotkey,
}

pub static mut CURRENT_MEDIA: Option<MediaHotkey> = None;

pub fn load_media() {
    let file = load_file::<MediaHotkeys>("./src/persist/media_listener.json");
    if file.is_ok() {
        let map = file.unwrap();
        for (source, actions) in map.media_hot_keys {
            for (action, keys) in actions {
                let moved_source = source.clone();
                let control_action = Controls::from_str(&action);
                if control_action.is_ok() {
                    let call_back = Box::new(move || {
                        block_on(run_media_hotkey(
                            Box::new(moved_source.clone()),
                            Box::new(control_action.clone().unwrap()),
                        ));
                    });
                    let _test = subscribe(KeyWatcher {
                        call_back,
                        keys,
                        key: format!("{}-{}", source.clone(), action),
                    });
                }
            }
        }
    } else {
        println!("error loading media {:?}", file);
    }
}

pub fn subscribe_media(source: String, keys: Vec<Key>, action: Controls) {
    let (move_source, action_move) = (source.clone(), action.clone());
    let callback = Box::new(move || {
        block_on(run_media_hotkey(
            Box::new(move_source.clone()),
            Box::new(action_move.clone()),
        ));
    });
    // subscribe to the key watcher
    let _test = subscribe(KeyWatcher::new(callback, keys.clone()));
    unsafe {
        // if the current media does not excist create it
        if CURRENT_MEDIA.is_none() {
            CURRENT_MEDIA = Some(HashMap::new());
        }
        //  clone of the current media
        let mut map = CURRENT_MEDIA.clone().unwrap();
        // if the source does not excist create it
        if !map.contains_key(&source) {
            map.insert(source.clone(), HashMap::new());
        }
        // clone of the source
        let actions = map.get_mut(&source).unwrap();
        // if the action does not excist create it
        if !actions.contains_key(&action.to_string()) {
            actions.insert(action.clone().to_string(), keys.clone());
        }
        println!("{:?}", map);
        CURRENT_MEDIA = Some(map);
        save_file("./src/persist/media_listener.json", CURRENT_MEDIA.clone())
            .expect("error saving settings");
    }
}
