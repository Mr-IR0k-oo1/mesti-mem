use anyhow::{bail, Result};
use std::process::{Command, Stdio};

use super::Executor;

pub struct OllamaExecutor {
    model: String,
}

impl OllamaExecutor {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

impl Executor for OllamaExecutor {
    fn name(&self) -> &str {
        &self.model
    }

    fn run(&self, prompt: &str) -> Result<String> {
        // Check ollama is available
        if Command::new("which").arg("ollama")
            .stdout(Stdio::null()).stderr(Stdio::null())
            .status().map(|s| !s.success()).unwrap_or(true)
        {
            bail!("ollama not found in PATH — install from https://ollama.com");
        }

        let output = Command::new("ollama")
            .args(["run", &self.model, prompt])
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("ollama exited with error: {}", err.trim());
        }

        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if response.is_empty() {
            bail!("ollama returned empty response");
        }
        Ok(response)
    }
}
