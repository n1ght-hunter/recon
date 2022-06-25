use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Controls {
    Pause,
    Play,
    Stop,
    Next,
    Previous,
}
impl fmt::Display for Controls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Controls {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pause" => Ok(Controls::Pause),
            "Play" => Ok(Controls::Play),
            "Stop" => Ok(Controls::Stop),
            "Next" => Ok(Controls::Next),
            "Previous" => Ok(Controls::Previous),
            _ => Err(()),
        }
    }
}

pub enum Status {
    PlayState,
    // Stop,
    // Next,
    // Previous,
}

pub async fn app_instance(
    source: &String,
) -> Result<GlobalSystemMediaTransportControlsSession, String> {
    let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().unwrap();
    let try_get_sessions = session_manager.await;
    if try_get_sessions.is_ok() {
        let sessions = try_get_sessions.unwrap().GetSessions();
        if sessions.is_ok() {
            let tested = sessions.unwrap().First().unwrap();
            for item in tested {
                let name = item.SourceAppUserModelId().unwrap().to_string();
                if &name == source {
                    return Ok(item);
                }
            }
        }
    }
    Err("No session found".to_string())
}

pub enum TaskError<T, E> {
    Ok(T),
    Err(E),
}

pub async fn application_task(
    item: GlobalSystemMediaTransportControlsSession,
    todo: &Controls,
) -> bool {
    let async_task = match todo {
        Controls::Pause => item.TryPauseAsync(),
        Controls::Play => item.TryPlayAsync(),
        Controls::Stop => item.TryStopAsync(),
        Controls::Next => item.TrySkipNextAsync(),
        Controls::Previous => item.TrySkipPreviousAsync(),
    };
    match async_task {
        Ok(async_task) => {
            let task = async_task.await;
            match task {
                Ok(_) => true,
                Err(e) => {
                    println!("{:?}", e);
                    false
                }
            }
        }
        Err(er) => {
            println!("{:?}", er);
            false
        }
    }
}

pub fn status(
    application: GlobalSystemMediaTransportControlsSession,
) -> GlobalSystemMediaTransportControlsSessionPlaybackStatus {
    application
        .GetPlaybackInfo()
        .unwrap()
        .PlaybackStatus()
        .unwrap()
}

pub fn status_state_to_string(
    status: GlobalSystemMediaTransportControlsSessionPlaybackStatus,
) -> &'static str {
    match status {
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => "Playing",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => "Paused",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => "Stopped",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => "Closed",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Opened => "Opened",
        GlobalSystemMediaTransportControlsSessionPlaybackStatus::Changing => "Changing",
        _ => "Unknown",
    }
}
