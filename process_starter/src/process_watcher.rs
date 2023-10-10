use futures::{executor::block_on, SinkExt, Stream, StreamExt};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use wmi::{COMLibrary, FilterValue, WMIConnection, WMIError};

use crate::Message;

pub fn process_watcher() -> iced::Subscription<Message> {
    iced::subscription::channel("process water", 5, |mut sender| async move {
        tokio::task::spawn_blocking(move || {
            let com_con = COMLibrary::new().unwrap();
            let wmi_con = WMIConnection::new(com_con.into()).unwrap();

            let mut filters = HashMap::<String, FilterValue>::new();

            filters.insert(
                "TargetInstance".to_owned(),
                FilterValue::is_a::<Process>().unwrap(),
            );

            let mut stream = wmi_con
                .filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(1)))
                .unwrap();

            for result in stream {
                let process = result.unwrap().target_instance;
                println!("New process!");
                println!("PID:        {}", process.process_id);
                println!("Name:       {}", process.name);
                println!("Executable: {:?}", process.executable_path);
            }
        });

        futures::future::pending().await
    })
}

async fn exec_async_process_watcher(mut sender: futures::channel::mpsc::Sender<Message>) {
    let com_con = COMLibrary::new().unwrap();
    let wmi_con = WMIConnection::new(com_con.into()).unwrap();

    let mut filters = HashMap::<String, FilterValue>::new();

    filters.insert(
        "TargetInstance".to_owned(),
        FilterValue::is_a::<Process>().unwrap(),
    );

    let mut stream = wmi_con
        .filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(1)))
        .unwrap();

    for result in stream {
        let process = result.unwrap().target_instance;
        println!("New process!");
        println!("PID:        {}", process.process_id);
        println!("Name:       {}", process.name);
        println!("Executable: {:?}", process.executable_path);
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "__InstanceCreationEvent")]
#[serde(rename_all = "PascalCase")]
struct NewProcessEvent {
    target_instance: Process,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Process")]
#[serde(rename_all = "PascalCase")]
struct Process {
    process_id: u32,
    name: String,
    executable_path: Option<String>,
}
