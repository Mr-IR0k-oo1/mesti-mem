use anyhow::{bail, Result};
use std::process::Command;
use super::Executor;

pub struct OllamaExecutor { model: String }

impl OllamaExecutor {
    pub fn new(model: String) -> Self { Self { model } }
}

impl Executor for OllamaExecutor {
    fn name(&self) -> &str { &self.model }
    fn run(&self, prompt: &str) -> Result<String> {
        let out = Command::new("ollama").args(["run", &self.model, prompt]).output()?;
        if !out.status.success() {
            bail!("ollama failed: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if r.is_empty() { bail!("ollama returned empty response"); }
        Ok(r)
    }
}
