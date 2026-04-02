# matis-mem

A terminal-native AI workspace that gives your prompts memory, structure, and continuity.

This is not a wrapper around LLMs.
It is a **stateful execution system** that enforces context discipline and logs every interaction.

---

## 🧠 Why This Exists

Most AI tools are stateless.

That means:

* you repeat context constantly
* past work is lost
* outputs are inconsistent
* switching models breaks flow

matis-mem fixes that by making **memory and context first-class**.

---

## ✨ Features

### 🧠 Persistent Projects

Define what you're working on once.

Each project stores:

* goal
* constraints
* decisions
* notes

Every prompt automatically includes this context.

---

### 🔁 Session Memory (Automatic)

Every run is saved and reused.

* last *N* sessions are injected into new prompts
* no copy-pasting
* real continuity across work

---

### ⚙️ Deterministic Context Builder

No hidden behavior.

Each prompt is built from:

* project context
* recent sessions
* optional knowledge search

You decide what goes in.

---

### 🔌 Multi-Model Support

Run the same workflow across:

* Ollama (local models like `llama3`, `mistral`)
* Gemini CLI
* Claude (via API)
* Mistral (via API)
* Amp Agent
* Custom executors

Switch models anytime without losing context.

---

### 🖥️ Terminal UI (TUI)

Keyboard-driven interface with:

* project selector
* prompt editor
* context controls
* model switcher
* response viewer

---

### 📚 Knowledge Base

Store reusable insights and inject them when needed.

* keyword-based search
* optional per prompt

---

### 🧾 Automatic Logging

Every interaction is saved:

* prompt
* context used
* response
* duration

All stored as plain JSON.

---

## 📦 Installation

### 1. Clone

```bash
git clone <repo> && cd matis-mem
```

---

### 2. Install

```bash
bash install.sh
```

Requires: **Rust 1.75+**

---

### 3. Install a Model

#### Option A — Local (recommended)

```bash
ollama pull llama3
```

---

#### Option B — Gemini CLI

```bash
npm install -g @google/gemini-cli
gemini auth
```

---

#### Option C — Claude (API)

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

---

## 🚀 Usage

Launch:

```bash
matis-mem
```

---

## 🖥️ Interface Overview

```text
◆ matis-mem  project: millcheck  model: ollama/llama3  ◉ done

┌─────────────────┬──────────────────────────────────────────────────┐
│ PROJECTS        │ PROMPT                                            │
│ ▶ millcheck     │ fix null date panic in parser                     │
│   api-gateway   ├────────────────────────┬─────────────────────────│
│                 │ CONTEXT                │ MODEL                   │
│                 │ [x] project context    │ ▶ ollama/llama3         │
│                 │ [x] last 2 sessions    │   ollama/mistral        │
│                 │ [ ] knowledge search   │   gemini-cli            │
│                 │ Ctrl+R / F5 = RUN      │   claude                │
│                 ├────────────────────────┴─────────────────────────│
│                 │ RESPONSE                                          │
│                 │ The panic occurs because `parse_date()` returns   │
│                 │ `Option<Date>` but the caller uses `.unwrap()`…   │
└─────────────────┴───────────────────────────────────────────────────┘
```

---

## 🔁 Workflow

### 1. Create a Project

`Ctrl + N`

Example:

```
Name: millcheck  
Goal: validate mill test certificates for steel
Constraints: must parse dates correctly, handle nulls
```

---

### 2. Select Project

Use `j/k` and press `Enter`

---

### 3. Write Prompt

Navigate to prompt panel (`Tab`):

```
fix null date panic in parser
```

---

### 4. Run

```
Ctrl + R
```

---

### 5. Context is Built Automatically

Example:

```text
[PROJECT: millcheck]
Goal: validate mill test certificates for steel
Constraints: must parse dates correctly, handle nulls

[RECENT SESSIONS]
Session 1: "why does parse_date panic?"
Response: Because it unwraps Option<Date> without checking...

[QUESTION]
fix null date panic in parser
```

---

### 6. Session is Logged

Saved automatically to:

```text
~/.matis-mem/sessions/<project>/
```

---

## ⌨️ Keybindings

### Core

| Key         | Action        |
| ----------- | ------------- |
| Tab / Shift+Tab | Switch panel  |
| Ctrl+R / F5 | Run prompt    |
| Ctrl+N      | New project   |
| Ctrl+K      | Add knowledge |
| q / Ctrl+C  | Quit          |

### Navigation

| Area     | Keys         |
| -------- | ------------ |
| Projects | j / k        |
| Models   | j / k        |
| Response | j / k, g / G |

### Context Controls

| Key   | Action                  |
| ----- | ----------------------- |
| Space | Toggle project context  |
| +/-   | Adjust session count    |
| k     | Toggle knowledge search |

### Editing

| Key         | Action         |
| ----------- | -------------- |
| Shift+Enter | Newline in prompt |
| Ctrl+C      | Quit from prompt |

---

## 📁 Data Storage

```text
~/.matis-mem/
├── projects/
│   ├── millcheck.json
│   └── api-gateway.json
├── sessions/
│   ├── millcheck/
│   │   ├── 20260401_143022_001.json
│   │   └── 20260401_143045_002.json
│   └── api-gateway/
│       └── 20260401_144200_001.json
├── knowledge/
│   ├── rust_patterns.json
│   └── project_context.json
└── agent_logs/
    └── 2026-04-01.jsonl
```

All files are human-readable JSON.

**Session format:**

```json
{
  "timestamp": "2026-04-01T14:30:22Z",
  "prompt": "fix null date panic in parser",
  "context_summary": "project + 2 sessions",
  "model": "ollama/llama3",
  "response": "...",
  "duration_ms": 2450
}
```

---

## 🏗️ Architecture

### Core Modules

```text
src/
├── executor/    # model execution layer
├── context/     # context building
├── data/        # storage (projects, sessions, knowledge)
├── ui/          # terminal interface (ratatui)
├── watcher/     # log monitoring & indexing
├── config.rs    # configuration
├── error.rs     # error types
└── main.rs      # app entry point
```

### Execution Flow

```
User Input
    ↓
Context Builder (project + sessions + knowledge)
    ↓
Executor (routes to model)
    ↓
Response
    ↓
Session Logger (saves automatically)
```

---

## ⚙️ Design Principles

* **Deterministic context** — no magic. You see exactly what's injected.
* **Small > big** — default is project + 2 sessions. Increase only when needed.
* **Single execution path** — all models use unified interface.
* **Mandatory logging** — every run is saved before confirmation.
* **UI ≠ logic** — interface only reads state. Logic is in core modules.
* **Extensible** — adding models means implementing one trait.

---

## 🔧 Adding a New Model

### 1. Create executor

```rust
// src/executor/mymodel.rs
use crate::executor::Executor;

pub struct MyModelExecutor;

impl Executor for MyModelExecutor {
    fn name(&self) -> &str { "mymodel" }
    fn run(&self, prompt: &str) -> Result<String> {
        // spawn subprocess, return stdout
        unimplemented!()
    }
}
```

### 2. Register in mod.rs

```rust
// src/executor/mod.rs
mod mymodel;
pub use mymodel::MyModelExecutor;
```

### 3. Add to Model enum

Update `Model` enum and `impl Model::all_presets()`.

Done — it appears in the model selector automatically.

---

## ⚠️ Compatibility Note

Rust 1.75 compatibility:

```bash
cargo update unicode-segmentation --precise 1.12.0
```

---

## 🚧 Limitations

* no embeddings (yet)
* keyword search only
* no cloud sync
* no multi-user support
* single-threaded execution

---

## 🧭 Roadmap

* semantic search via embeddings
* prompt templates and macros
* multi-agent workflows
* response streaming
* performance optimizations
* web UI option

---

## 📜 License

MIT

---

## Final Note

This tool only works if you actually use it consistently.

If you bypass it and go back to raw AI commands, you're back to stateless chaos.

The discipline is the feature.
