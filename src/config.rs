use std::path::PathBuf;
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init() {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".matis-mem");
    DATA_DIR.set(dir).ok();
}

pub fn data_dir()     -> &'static PathBuf { DATA_DIR.get().expect("config::init not called") }
pub fn projects_dir() -> PathBuf { data_dir().join("01-projects") }
pub fn sessions_dir() -> PathBuf { data_dir().join("04-sessions") }
pub fn knowledge_dir()-> PathBuf { data_dir().join("02-knowledge") }
pub fn external_dir() -> PathBuf { data_dir().join("external") }
pub fn shims_dir()    -> PathBuf { data_dir().join("shims") }

pub fn ensure_dirs() -> anyhow::Result<()> {
    for d in &[
        projects_dir(), sessions_dir(), knowledge_dir(),
        external_dir(), shims_dir(),
        data_dir().join("03-prompts"),
        data_dir().join("05-scratch"),
        data_dir().join("06-agents/_shared"),
        data_dir().join("06-agents/.slots"),
    ] {
        std::fs::create_dir_all(d)?;
    }
    Ok(())
}
