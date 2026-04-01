use anyhow::{bail, Result};
use std::process::{Command, Stdio};

use super::Executor;

pub struct GeminiExecutor {
    use_cli: bool,
}

impl GeminiExecutor {
    pub fn new(use_cli: bool) -> Self {
        Self { use_cli }
    }
}

impl Executor for GeminiExecutor {
    fn name(&self) -> &str {
        if self.use_cli { "gemini-cli" } else { "gemini" }
    }

    fn run(&self, prompt: &str) -> Result<String> {
        // Support both `gemini` and `gemini-cli` depending on what's installed
        let cmd_name = if self.use_cli { "gemini" } else { "gemini" };

        if Command::new("which").arg(cmd_name)
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status().map(|s| !s.success()).unwrap_or(true)
        {
            bail!(
                "'{}' not found in PATH.\nInstall: npm install -g @google/gemini-cli",
                cmd_name
            );
        }

        let output = Command::new(cmd_name)
            .args(["-p", prompt])
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("gemini exited with error: {}", err.trim());
        }

        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if response.is_empty() {
            bail!("gemini returned empty response");
        }
        Ok(response)
    }
}
