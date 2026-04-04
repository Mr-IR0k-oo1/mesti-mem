use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::config::projects_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name:        String,
    pub goal:        String,
    pub constraints: Vec<String>,
    pub decisions:   Vec<String>,
    #[serde(default)]
    pub notes:       String,
}

impl Project {
    pub fn new(name: impl Into<String>, goal: impl Into<String>) -> Self {
        Self { name: name.into(), goal: goal.into(), constraints: vec![], decisions: vec![], notes: String::new() }
    }

    fn path(name: &str) -> PathBuf {
        projects_dir().join(format!("{}.json", name))
    }

    pub fn save(&self) -> Result<()> {
        let p = Self::path(&self.name);
        if let Some(d) = p.parent() { std::fs::create_dir_all(d)?; }
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(tmp, p)?;
        Ok(())
    }

    pub fn load(name: &str) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(Self::path(name))?)?)
    }

    pub fn list() -> Result<Vec<String>> {
        let dir = projects_dir();
        if !dir.exists() { return Ok(vec![]); }
        let mut names: Vec<String> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let p = e.path();
                if p.extension()?.to_str()? == "json" {
                    Some(p.file_stem()?.to_string_lossy().to_string())
                } else { None }
            })
            .collect();
        names.sort();
        Ok(names)
    }

    pub fn delete(name: &str) -> Result<()> {
        let p = Self::path(name);
        if p.exists() { std::fs::remove_file(p)?; }
        Ok(())
    }

    pub fn as_context(&self) -> String {
        let mut s = format!("[PROJECT: {}]\nGoal: {}\n", self.name, self.goal);
        if !self.constraints.is_empty() {
            s += &format!("Constraints: {}\n", self.constraints.join(", "));
        }
        if !self.decisions.is_empty() {
            s += &format!("Decisions: {}\n", self.decisions.join(", "));
        }
        if !self.notes.is_empty() {
            s += &format!("Notes: {}\n", self.notes);
        }
        s
    }
}
