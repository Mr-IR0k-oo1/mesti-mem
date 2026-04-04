use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use crate::config::external_dir;
use crate::data::AgentLog;

pub enum WatchEvent {
    NewLog(AgentLog),
    Error(String),
}

pub fn start() -> Result<Receiver<WatchEvent>> {
    let (tx, rx) = mpsc::channel::<WatchEvent>();
    let watch_dir = external_dir();
    std::fs::create_dir_all(&watch_dir)?;
    thread::spawn(move || {
        if let Err(e) = watch_loop(watch_dir, tx.clone()) {
            let _ = tx.send(WatchEvent::Error(e.to_string()));
        }
    });
    Ok(rx)
}

fn watch_loop(dir: PathBuf, tx: Sender<WatchEvent>) -> Result<()> {
    let (fs_tx, fs_rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = RecommendedWatcher::new(fs_tx, Config::default())?;
    watcher.watch(&dir, RecursiveMode::Recursive)?;
    for res in fs_rx {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Create(_)) {
                for path in event.paths {
                    if path.extension().and_then(|x| x.to_str()) == Some("json") {
                        thread::sleep(std::time::Duration::from_millis(60));
                        if let Ok(log) = AgentLog::load(&path) {
                            let _ = tx.send(WatchEvent::NewLog(log));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
