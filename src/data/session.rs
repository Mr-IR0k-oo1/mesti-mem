use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::sessions_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,           // timestamp-based
    pub project: String,
    pub model: String,
    pub prompt: String,
    pub context_summary: String,
    pub response: String,
    pub timestamp: String,
    pub duration_ms: u64,
}

impl Session {
    pub fn new(
        project: &str,
        model: &str,
        prompt: &str,
        context_summary: &str,
        response: &str,
        duration_ms: u64,
    ) -> Self {
        let ts = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        Self {
            id: ts.clone(),
            project: project.to_string(),
            model: model.to_string(),
            prompt: prompt.to_string(),
            context_summary: context_summary.to_string(),
            response: response.to_string(),
            timestamp: Local::now().to_rfc3339(),
            duration_ms,
        }
    }

    fn dir_for(project: &str) -> PathBuf {
        sessions_dir().join(project)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::dir_for(&self.project);
        std::fs::create_dir_all(&dir)?;
        let p = dir.join(format!("{}.json", self.id));
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(tmp, p)?;
        Ok(())
    }

    /// Load last N sessions for a project (sorted newest-first)
    pub fn last_n(project: &str, n: usize) -> Result<Vec<Self>> {
        let dir = Self::dir_for(project);
        if !dir.exists() { return Ok(vec![]); }

        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .map(|e| e.path())
            .collect();

        // Sort by filename (timestamp-named) descending
        files.sort_by(|a, b| b.cmp(a));

        let mut sessions = Vec::new();
        for path in files.into_iter().take(n) {
            if let Ok(raw) = std::fs::read_to_string(&path) {
                if let Ok(s) = serde_json::from_str::<Session>(&raw) {
                    sessions.push(s);
                }
            }
        }
        Ok(sessions)
    }

    /// Format as context block for prompt injection
    pub fn as_context(&self) -> String {
        format!(
            "[SESSION {}]\nPrompt: {}\nResponse summary: {}\n",
            &self.timestamp[..16],
            self.prompt,
            if self.response.len() > 200 {
                format!("{}...", &self.response[..200])
            } else {
                self.response.clone()
            }
        )
    }

    /// All sessions for a project (newest first)
    pub fn list(project: &str) -> Result<Vec<Self>> {
        Self::last_n(project, 1000)
    }
}
