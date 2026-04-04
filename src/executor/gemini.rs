use anyhow::{bail, Result};
use std::process::Command;
use super::Executor;

pub struct GeminiExecutor { cli_mode: bool }

impl GeminiExecutor {
    pub fn new(cli_mode: bool) -> Self { Self { cli_mode } }
}

impl Executor for GeminiExecutor {
    fn name(&self) -> &str { if self.cli_mode { "gemini-cli" } else { "gemini" } }
    fn run(&self, prompt: &str) -> Result<String> {
        if !crate::platform::bin_available("gemini") {
            bail!("gemini not found. Install: npm install -g @google/gemini-cli && gemini auth");
        }
        let out = Command::new("gemini").args(["-p", prompt]).output()?;
        if !out.status.success() {
            bail!("gemini failed: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if r.is_empty() { bail!("gemini returned empty response"); }
        Ok(r)
    }
}
