use anyhow::{bail, Result};
use std::process::{Command, Stdio};
use super::Executor;

pub struct ClaudeExecutor { print_mode: bool }

impl ClaudeExecutor {
    pub fn new(print_mode: bool) -> Self { Self { print_mode } }
}

impl Executor for ClaudeExecutor {
    fn name(&self) -> &str { if self.print_mode { "claude --print" } else { "claude code" } }
    fn run(&self, prompt: &str) -> Result<String> {
        if !crate::platform::bin_available("claude") {
            bail!("claude not found. Install: npm install -g @anthropic-ai/claude-code");
        }
        if self.print_mode {
            let out = Command::new("claude").args(["--print", prompt]).output()?;
            if !out.status.success() {
                bail!("claude failed: {}", String::from_utf8_lossy(&out.stderr).trim());
            }
            let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if r.is_empty() { bail!("claude returned empty response"); }
            Ok(r)
        } else {
            use std::io::Write;
            let mut child = Command::new("claude")
                .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
            if let Some(ref mut stdin) = child.stdin { stdin.write_all(prompt.as_bytes())?; }
            let out = child.wait_with_output()?;
            let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if r.is_empty() { Ok("(claude ran interactively — no stdout captured)".into()) } else { Ok(r) }
        }
    }
}
