use anyhow::{bail, Result};
use std::process::Command;
use super::Executor;

pub struct AmpExecutor;
impl AmpExecutor { pub fn new() -> Self { Self } }

impl Executor for AmpExecutor {
    fn name(&self) -> &str { "amp" }
    fn run(&self, prompt: &str) -> Result<String> {
        if !crate::platform::bin_available("amp") {
            bail!("amp not found. Install from: https://ampcode.com");
        }
        let out = Command::new("amp").args(["run", prompt]).output()?;
        if !out.status.success() {
            bail!("amp failed: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if r.is_empty() { Ok("(amp ran — check terminal for output)".into()) } else { Ok(r) }
    }
}
