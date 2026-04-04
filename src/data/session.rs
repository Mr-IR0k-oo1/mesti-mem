use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::config::sessions_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id:              String,
    pub project:         String,
    pub model:           String,
    pub prompt:          String,
    pub context_summary: String,
    pub response:        String,
    pub timestamp:       String,
    pub duration_ms:     u64,
}

impl Session {
    pub fn new(project: &str, model: &str, prompt: &str, ctx: &str, response: &str, ms: u64) -> Self {
        let ts = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        Self {
            id: ts, project: project.into(), model: model.into(),
            prompt: prompt.into(), context_summary: ctx.into(),
            response: response.into(), timestamp: Local::now().to_rfc3339(), duration_ms: ms,
        }
    }

    pub fn save(&self) -> Result<()> {
        let dir = sessions_dir().join(&self.project);
        std::fs::create_dir_all(&dir)?;
        let p = dir.join(format!("{}.json", self.id));
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(tmp, p)?;
        Ok(())
    }

    pub fn last_n(project: &str, n: usize) -> Result<Vec<Self>> {
        let dir = sessions_dir().join(project);
        if !dir.exists() { return Ok(vec![]); }
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .map(|e| e.path())
            .collect();
        files.sort_by(|a, b| b.cmp(a));
        let mut out = vec![];
        for p in files.into_iter().take(n) {
            if let Ok(s) = std::fs::read_to_string(&p)
                .and_then(|r| serde_json::from_str(&r).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
            {
                out.push(s);
            }
        }
        Ok(out)
    }

    pub fn as_context(&self) -> String {
        let preview = if self.response.len() > 200 {
            format!("{}…", &self.response[..200])
        } else { self.response.clone() };
        format!("[SESSION {}]\nPrompt: {}\nResponse: {}\n", &self.timestamp[..16], self.prompt, preview)
    }
}
