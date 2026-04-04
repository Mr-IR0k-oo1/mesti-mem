use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use crate::config::external_dir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode { Full, Interactive, Task }

impl fmt::Display for CaptureMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureMode::Full        => write!(f, "full"),
            CaptureMode::Interactive => write!(f, "interactive"),
            CaptureMode::Task        => write!(f, "task"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLog {
    pub id:          String,
    pub agent:       String,
    pub cwd:         String,
    pub project:     String,
    pub args:        String,
    pub input:       String,
    pub output:      String,
    pub duration_ms: u64,
    pub exit_code:   i32,
    pub timestamp:   String,
    pub capture:     CaptureMode,
}

impl AgentLog {
    pub fn load(path: &Path) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn recent(n: usize) -> Result<Vec<Self>> {
        let root = external_dir();
        if !root.exists() { return Ok(vec![]); }
        let mut all: Vec<(std::time::SystemTime, PathBuf)> = vec![];
        for agent_dir in std::fs::read_dir(&root)?.filter_map(|e| e.ok()) {
            if !agent_dir.path().is_dir() { continue; }
            for log in std::fs::read_dir(agent_dir.path())?.filter_map(|e| e.ok()) {
                let p = log.path();
                if p.extension().and_then(|x| x.to_str()) == Some("json") {
                    let mtime = p.metadata().and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    all.push((mtime, p));
                }
            }
        }
        all.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(all.into_iter().take(n).filter_map(|(_, p)| Self::load(&p).ok()).collect())
    }

    pub fn known_agents() -> Vec<String> {
        let root = external_dir();
        if !root.exists() { return vec![]; }
        let mut agents: Vec<String> = std::fs::read_dir(&root).into_iter().flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        agents.sort();
        agents
    }
}
