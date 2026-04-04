use anyhow::{bail, Result};
use std::process::Command;
use super::Executor;

pub struct VibeExecutor;
impl VibeExecutor { pub fn new() -> Self { Self } }

impl Executor for VibeExecutor {
    fn name(&self) -> &str { "vibe" }
    fn run(&self, prompt: &str) -> Result<String> {
        let bin = if crate::platform::bin_available("vibe") { "vibe" }
                  else if crate::platform::bin_available("cursor") { "cursor" }
                  else { bail!("vibe/cursor not found in PATH"); };
        let out = Command::new(bin).args(["--message", prompt]).output()?;
        if out.status.success() {
            let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !r.is_empty() { return Ok(r); }
        }
        // try positional
        let out2 = Command::new(bin).arg(prompt).output()?;
        let r = String::from_utf8_lossy(&out2.stdout).trim().to_string();
        if r.is_empty() { Ok("(vibe ran — check editor for output)".into()) } else { Ok(r) }
    }
}
