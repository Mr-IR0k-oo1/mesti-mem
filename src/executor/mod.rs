pub mod gemini;
pub mod ollama;

use anyhow::Result;

/// Every executor implements this. One interface, any backend.
pub trait Executor {
    fn name(&self) -> &str;
    fn run(&self, prompt: &str) -> Result<String>;
}

/// Registered models. Extend this when adding new backends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Model {
    Ollama { model: String },
    Gemini,
    GeminiCli,
}

impl Model {
    pub fn display_name(&self) -> String {
        match self {
            Model::Ollama { model } => format!("ollama/{}", model),
            Model::Gemini    => "gemini".into(),
            Model::GeminiCli => "gemini-cli".into(),
        }
    }

    pub fn all_presets() -> Vec<Model> {
        vec![
            Model::Ollama { model: "llama3".into() },
            Model::Ollama { model: "mistral".into() },
            Model::Ollama { model: "codellama".into() },
            Model::GeminiCli,
        ]
    }

    pub fn executor(&self) -> Box<dyn Executor> {
        match self {
            Model::Ollama { model } => Box::new(ollama::OllamaExecutor::new(model.clone())),
            Model::Gemini    => Box::new(gemini::GeminiExecutor::new(false)),
            Model::GeminiCli => Box::new(gemini::GeminiExecutor::new(true)),
        }
    }
}

/// Run a prompt through any model. This is the single entry point.
pub fn run(model: &Model, prompt: &str) -> Result<String> {
    model.executor().run(prompt)
}
