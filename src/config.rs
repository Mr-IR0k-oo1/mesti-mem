use std::path::PathBuf;
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init() {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".matis-mem");
    DATA_DIR.set(dir).ok();
}

pub fn data_dir() -> &'static PathBuf {
    DATA_DIR.get().expect("config::init() not called")
}

pub fn projects_dir() -> PathBuf { data_dir().join("projects") }
pub fn sessions_dir() -> PathBuf { data_dir().join("sessions") }
pub fn knowledge_dir()-> PathBuf { data_dir().join("knowledge") }
pub fn prompts_dir()  -> PathBuf { data_dir().join("prompts") }
pub fn state_file()   -> PathBuf { data_dir().join("state.json") }

/// Ensure all base directories exist
pub fn ensure_dirs() -> anyhow::Result<()> {
    for dir in &[projects_dir(), sessions_dir(), knowledge_dir(), prompts_dir()] {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}
