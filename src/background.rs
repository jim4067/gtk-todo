use crate::Event;
use async_channel::{Receiver, Sender};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, io};

//events in the background that the thread will respond to
pub enum BgEvent {
    //save task to a file
    Save(PathBuf, String),

    //Exit from the program loop
    Quit,
}

pub async fn run(tx: Sender<Event>, rx: Receiver<BgEvent>) {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(crate::APP_ID).unwrap();
    let data_home = xdg_dirs.get_data_home();
    let _ = fs::create_dir_all(&data_home);

    if let Some(path) = most_recent_file(&data_home).unwrap() {
        if let Ok(data) = fs::read_to_string(&path) {
            let _ = tx.send(Event::Load(data)).await;
        }
    }

    while let Ok(event) = rx.recv().await {
        match event {
            BgEvent::Save(path, data) => {
                let path = xdg_dirs.place_data_file(path).unwrap();
                fs::write(&path, data.as_bytes()).unwrap();
            }
            BgEvent::Quit => break,
        }
    }
    let _ = tx.send(Event::Quit).await;
}

fn most_recent_file(path: &Path) -> io::Result<Option<PathBuf>> {
    let mut most_recent = SystemTime::UNIX_EPOCH;
    let mut target = None;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type().map_or(false, |kind| kind.is_file()) {
            if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
                if modified > most_recent {
                    target = Some(entry.path());
                    most_recent = modified;
                }
            }
        }
    }
    Ok(target)
}
