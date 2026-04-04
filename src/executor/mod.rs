pub mod amp;
pub mod claude;
pub mod gemini;
pub mod generic;
pub mod mistral;
pub mod ollama;
pub mod vibe;

use anyhow::Result;

pub trait Executor {
    fn name(&self) -> &str;
    fn run(&self, prompt: &str) -> Result<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Model {
    Ollama      { model: String },
    Mistral     { model: String },
    Gemini,
    GeminiCli,
    ClaudePrint,
    ClaudeCode,
    Amp,
    Vibe,
    Custom { label: String, command: String, args: Vec<String> },
}

impl Model {
    pub fn display_name(&self) -> String {
        match self {
            Model::Ollama   { model } => format!("ollama/{}", model),
            Model::Mistral  { model } => format!("mistral/{}", model),
            Model::Gemini             => "gemini".into(),
            Model::GeminiCli          => "gemini-cli".into(),
            Model::ClaudePrint        => "claude --print".into(),
            Model::ClaudeCode         => "claude code".into(),
            Model::Amp                => "amp".into(),
            Model::Vibe               => "vibe".into(),
            Model::Custom { label, .. } => label.clone(),
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Model::Ollama  { .. } | Model::Mistral { .. } => "Local",
            Model::Gemini  | Model::GeminiCli             => "Cloud",
            Model::ClaudePrint | Model::ClaudeCode         => "Claude",
            Model::Amp     | Model::Vibe                   => "Agents",
            Model::Custom  { .. }                          => "Custom",
        }
    }

    pub fn executor(&self) -> Box<dyn Executor> {
        match self {
            Model::Ollama  { model } => Box::new(ollama::OllamaExecutor::new(model.clone())),
            Model::Mistral { model } => Box::new(mistral::MistralExecutor::new(model.clone())),
            Model::Gemini            => Box::new(gemini::GeminiExecutor::new(false)),
            Model::GeminiCli         => Box::new(gemini::GeminiExecutor::new(true)),
            Model::ClaudePrint       => Box::new(claude::ClaudeExecutor::new(true)),
            Model::ClaudeCode        => Box::new(claude::ClaudeExecutor::new(false)),
            Model::Amp               => Box::new(amp::AmpExecutor::new()),
            Model::Vibe              => Box::new(vibe::VibeExecutor::new()),
            Model::Custom { label, command, args } => Box::new(
                generic::GenericExecutor::new(label.clone(), command.clone(),
                    args.iter().map(|s| s.as_str()).collect())
            ),
        }
    }

    /// Detect only installed models — runs `ollama list` and checks PATH for each binary.
    pub fn detect_available() -> Vec<Model> {
        let mut models = vec![];
        let av = crate::platform::bin_available;

        // ── Local ollama ──────────────────────────────────────────────────────
        if av("ollama") {
            let pulled = query_ollama_models();
            if pulled.is_empty() {
                // Installed but nothing pulled yet — show placeholder
                models.push(Model::Custom {
                    label:   "ollama (no models pulled)".into(),
                    command: "echo".into(),
                    args:    vec!["run: ollama pull llama3".into()],
                });
            } else {
                for m in pulled { models.push(Model::Ollama { model: m }); }
            }
        }

        // ── Local mistral CLI ─────────────────────────────────────────────────
        if av("mistral") {
            models.push(Model::Mistral { model: "mistral-small".into() });
        }

        // ── Cloud: gemini ─────────────────────────────────────────────────────
        if av("gemini") {
            models.push(Model::GeminiCli);
        }

        // ── Claude ────────────────────────────────────────────────────────────
        if av("claude") {
            models.push(Model::ClaudePrint);
            models.push(Model::ClaudeCode);
        }

        // ── Agents ────────────────────────────────────────────────────────────
        if av("amp") { models.push(Model::Amp); }
        if av("vibe") || av("cursor") { models.push(Model::Vibe); }

        // Nothing found — show a helpful placeholder
        if models.is_empty() {
            models.push(Model::Custom {
                label:   "no models found".into(),
                command: "echo".into(),
                args:    vec!["install ollama, gemini-cli, or claude".into()],
            });
        }

        models
    }
}

fn query_ollama_models() -> Vec<String> {
    let Ok(out) = std::process::Command::new("ollama").arg("list").output() else { return vec![]; };
    if !out.status.success() { return vec![]; }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .skip(1) // skip header
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            if name.is_empty() { return None; }
            Some(name.strip_suffix(":latest").unwrap_or(name).to_string())
        })
        .collect()
}

pub fn run(model: &Model, prompt: &str) -> Result<String> {
    model.executor().run(prompt)
}
