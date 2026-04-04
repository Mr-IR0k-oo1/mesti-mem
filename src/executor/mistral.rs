use anyhow::{bail, Result};
use std::process::Command;
use super::Executor;

pub struct MistralExecutor { model: String }
impl MistralExecutor { pub fn new(m: impl Into<String>) -> Self { Self { model: m.into() } } }

impl Executor for MistralExecutor {
    fn name(&self) -> &str { "mistral" }
    fn run(&self, prompt: &str) -> Result<String> {
        // Try mistral CLI first, fall back to ollama
        if crate::platform::bin_available("mistral") {
            let out = Command::new("mistral").args(["chat", "--no-stream", "-m", &self.model, prompt]).output()?;
            if out.status.success() {
                let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !r.is_empty() { return Ok(r); }
            }
        }
        if crate::platform::bin_available("ollama") {
            let out = Command::new("ollama").args(["run", &self.model, prompt]).output()?;
            if out.status.success() {
                let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !r.is_empty() { return Ok(r); }
            }
        }
        bail!("mistral not available. Try: ollama pull {}", self.model)
    }
}
