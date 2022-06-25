use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str, to_string_pretty};
use std::fs::{read_to_string, write};

pub fn load_file<T: DeserializeOwned>(path: &str) -> Result<T, serde_json::Error> {
    from_str::<T>(&read_to_string(path).expect("error reading settings"))
}

pub fn save_file<T: Serialize>(path: &str, default: T) -> Result<(), std::io::Error> {
    let data = to_string_pretty(&default).expect("error serializing settings");
    write(path, data)
}
