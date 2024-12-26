use std::path::PathBuf;

/// Returns the path to the user's video directory.
pub fn video_dir() -> PathBuf {
    let mut path = dirs::video_dir().unwrap();
    path.push("Recon");
    path
}