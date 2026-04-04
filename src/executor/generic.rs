use anyhow::{bail, Result};
use std::process::{Command, Stdio};
use super::Executor;

pub struct GenericExecutor {
    pub label:      String,
    pub command:    String,
    pub args:       Vec<String>,
    pub stdin_mode: bool,
}

impl GenericExecutor {
    pub fn new(label: impl Into<String>, command: impl Into<String>, args: Vec<&str>) -> Self {
        Self {
            label: label.into(), command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(), stdin_mode: false,
        }
    }
}

impl Executor for GenericExecutor {
    fn name(&self) -> &str { &self.label }
    fn run(&self, prompt: &str) -> Result<String> {
        if !crate::platform::bin_available(&self.command) {
            bail!("'{}' not found in PATH", self.command);
        }
        let out = if self.stdin_mode {
            use std::io::Write;
            let mut child = Command::new(&self.command)
                .args(&self.args).stdin(Stdio::piped())
                .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
            if let Some(ref mut s) = child.stdin { s.write_all(prompt.as_bytes())?; }
            child.wait_with_output()?
        } else {
            let mut cmd = Command::new(&self.command);
            cmd.args(&self.args).arg(prompt);
            cmd.output()?
        };
        if !out.status.success() {
            bail!("'{}' failed: {}", self.command, String::from_utf8_lossy(&out.stderr).trim());
        }
        let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if r.is_empty() { Ok(format!("({} ran — no output)", self.label)) } else { Ok(r) }
    }
}
