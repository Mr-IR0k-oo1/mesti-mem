use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::knowledge_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Knowledge {
    pub topic: String,
    pub notes: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Knowledge {
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            notes: Vec::new(),
            tags: Vec::new(),
        }
    }

    fn path(topic: &str) -> PathBuf {
        let safe = topic.replace(' ', "_").to_lowercase();
        knowledge_dir().join(format!("{}.json", safe))
    }

    pub fn save(&self) -> Result<()> {
        let p = Self::path(&self.topic);
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(tmp, p)?;
        Ok(())
    }

    pub fn load(topic: &str) -> Result<Self> {
        let raw = std::fs::read_to_string(Self::path(topic))?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn list() -> Result<Vec<String>> {
        let dir = knowledge_dir();
        if !dir.exists() { return Ok(vec![]); }
        let mut topics: Vec<String> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let p = e.path();
                if p.extension()?.to_str()? == "json" {
                    Some(p.file_stem()?.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();
        topics.sort();
        Ok(topics)
    }

    /// Simple keyword search across all knowledge files
    pub fn search(query: &str) -> Result<Vec<Self>> {
        let q = query.to_lowercase();
        let dir = knowledge_dir();
        if !dir.exists() { return Ok(vec![]); }

        let mut results = Vec::new();
        for entry in std::fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
            if let Ok(raw) = std::fs::read_to_string(entry.path()) {
                if raw.to_lowercase().contains(&q) {
                    if let Ok(k) = serde_json::from_str::<Knowledge>(&raw) {
                        results.push(k);
                    }
                }
            }
        }
        Ok(results)
    }

    /// Format as context block
    pub fn as_context(&self) -> String {
        let notes = self.notes.join("; ");
        format!("[KNOWLEDGE: {}]\n{}\n", self.topic, notes)
    }
}
