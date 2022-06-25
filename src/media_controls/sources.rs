use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

pub async fn media_sources() -> Vec<String> {
    let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().unwrap();
    let sessions = session_manager
        .await
        .unwrap()
        .GetSessions()
        .unwrap()
        .First()
        .unwrap();
    sessions
        .map(|current_item| current_item.SourceAppUserModelId().unwrap().to_string())
        .collect::<Vec<_>>()
}
