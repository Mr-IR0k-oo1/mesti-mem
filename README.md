# matis-mem

**v0.2.0** — Terminal AI workspace with persistent memory, context discipline, and automatic logging.

This is not a wrapper around LLMs. It is a **stateful execution system** that enforces context discipline and logs every interaction.

---

## 🧠 Why This Exists

Most AI tools are stateless.

That means:
- you repeat context constantly
- past work is lost
- outputs are inconsistent
- switching models breaks flow

**matis-mem** fixes that by making **memory and context first-class citizens**.

---

## ✨ Core Features

### 🧠 Persistent Projects

Define what you're working on once. Each project stores:
- **goal** — what you're trying to do
- **constraints** — what matters
- **decisions** — what you've chosen
- **notes** — any context

Every prompt automatically includes this context. No more repeating yourself.

---

### 🔁 Session Memory (Automatic)

Every run is saved and reused.
- last *N* sessions are injected into new prompts
- no copy-pasting
- real continuity across work

---

### ⚙️ Deterministic Context Builder

No hidden behavior. Each prompt is built from:
1. **project context** — always first
2. **recent sessions** — last N (default: 2)
3. **knowledge search** — optional keyword lookup

You decide what goes in via checkboxes before every run.

---

### 🔌 Multi-Model Execution

Run the same workflow across different models:

**Local:**
- Ollama (`llama3`, `mistral`, `codellama`, `deepseek-coder`)

**Cloud APIs:**
- Gemini (native or CLI)
- Claude (print or code mode)
- Mistral

**Agents:**
- Amp (Sourcegraph AI agent)
- Vibe (local custom model)

Switch models anytime without losing context.

---

### 🖥️ Multi-Tab Terminal UI

Keyboard-driven interface with 4 tabs:

1. **[1] RUN** — Write prompts with memory context
2. **[2] AGENTS** — Live feed of external agent sessions
3. **[3] SHIMS** — Install logging wrappers for agent CLIs
4. **[4] KNOWLEDGE** — Browse and add knowledge base

---

### 📚 Knowledge Base

Store reusable insights and inject them when needed:
- keyword-based search
- optional per-prompt inclusion
- plain JSON storage

---

### 🧾 Automatic Logging

Every interaction is saved:
- prompt
- context used
- response
- model used
- duration (ms)

All stored as plain JSON + JSONL for agent logs.

---

### 🔗 Agent Shims

Install logging wrappers for external agent CLIs:
- `claude`
- `amp`
- `gemini`
- `vibe`
- `aider`
- `copilot`
- `mistral`
- `ollama`

All calls from ANY terminal are auto-logged when shims are active.

---

## 📦 Installation

### Requirements

- **Rust 1.75+** (or: `cargo update unicode-segmentation --precise 1.12.0`)
- **At least one model installed** (see below)

### Install matis-mem

```bash
git clone <repo> matis-mem && cd matis-mem
bash install.sh
```

This compiles the binary and places it in `~/.local/bin/matis-mem`.

---

## 🚀 Getting Started

### 1. Start matis-mem

```bash
matis-mem
```

### 2. Install a Model

Press `[3]` for **SHIMS** tab, or install manually:

#### Option A — Ollama (recommended for local)

```bash
ollama pull llama3
ollama serve  # in another terminal
```

#### Option B — Gemini CLI

```bash
npm install -g @google/gemini-cli
gemini auth
```

#### Option C — Claude (API)

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

#### Option D — Mistral (API)

```bash
export MISTRAL_API_KEY=sk-...
```

---

## 🖥️ Interface Overview

```text
◆ matis-mem v0.2.0  [1]RUN [2]AGENTS [3]SHIMS [4]KNOWLEDGE

┌─────────────────┬──────────────────────────────────────────────────┐
│ PROJECTS        │ PROMPT                                            │
│ ▶ millcheck     │ fix null date panic in parser                     │
│   api-gateway   ├────────────────────────┬─────────────────────────│
│                 │ CONTEXT                │ MODEL                   │
│                 │ [x] project context    │ ▶ ollama/llama3         │
│                 │ [x] last 2 sessions    │   ollama/mistral        │
│                 │ [ ] knowledge search   │   gemini-cli            │
│                 │ [+ sessions] [- sessions] │   claude --print     │
│                 │ Ctrl+R = RUN           │   amp                   │
│                 ├────────────────────────┴─────────────────────────│
│                 │ RESPONSE (↓ scroll)                               │
│                 │ The panic occurs because `parse_date()` returns   │
│                 │ `Option<Date>` but the caller uses `.unwrap()`…   │
└─────────────────┴───────────────────────────────────────────────────┘

◀ ▶ nav | Tab cycle | Ctrl+R run | Ctrl+N new | Ctrl+K knowledge | q quit
```

---

## 🔁 Typical Workflow

### Step 1 — Create a Project

Press `Ctrl+N`:

```
Name: millcheck
Goal: validate mill test certificates for steel
Constraints: must handle null dates, ISO 8601 format
```

### Step 2 — Select Project

Use `j/k` to navigate, press `Enter` to select.

### Step 3 — Write Prompt

Navigate to prompt panel (`Tab`):

```
fix null date panic in parser
```

### Step 4 — Check Context

Verify checkboxes in the CONTEXT panel:
- `[x]` project context
- `[x]` last 2 sessions
- `[ ]` knowledge search (toggle with `k`)

### Step 5 — Run

Press `Ctrl+R` or `F5`.

Context is injected automatically:

```text
[PROJECT: millcheck]
Goal: validate mill test certificates for steel
Constraints: must handle null dates, ISO 8601 format

[RECENT SESSIONS]
Q: why does parse_date panic?
A: Because it unwraps Option<Date> without checking...

Q: how to handle Option<T>?
A: Use match, if let, or .unwrap_or()...

[YOUR QUESTION]
fix null date panic in parser
```

### Step 6 — Session is Logged

Automatically saved to:

```text
~/.matis-mem/sessions/millcheck/20260402_101500_001.json
```

Contains: prompt, response, model, duration.

---

## ⌨️ Keybindings

### Global

| Key | Action |
|-----|--------|
| `1`, `2`, `3`, `4` | Switch tabs |
| `q` / `Ctrl+C` | Quit |
| `Ctrl+L` | Clear terminal |

### RUN Tab

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle focus (Projects → Prompt → Context → Model → Response) |
| `j` / `k` / `↑` / `↓` | Navigate lists |
| `Enter` | Select project / toggle checkbox |
| `Ctrl+N` | New project |
| `Ctrl+R` / `F5` | Run prompt |
| `Shift+Enter` | Newline in prompt |
| `Ctrl+K` | Add knowledge entry |
| `Space` | Toggle project context checkbox |
| `+` / `-` | Increase/decrease session count |
| `k` | Toggle knowledge search checkbox |
| `g` / `G` | Jump to top/bottom of response |
| `c` | Clear response |
| `y` | Copy response to clipboard |

### AGENTS Tab

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll agent log |
| `Space` | Toggle agent on/off |

### SHIMS Tab

| Key | Action |
|-----|--------|
| `Space` | Install/uninstall shim |
| `j` / `k` | Navigate available shims |

### KNOWLEDGE Tab

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate knowledge |
| `Ctrl+K` | Add new entry |
| `Enter` | View/edit entry |

---

## 📁 Data Storage

All data is stored as plain text (JSON / JSONL) in `~/.matis-mem/`:

```text
~/.matis-mem/
├── projects/
│   ├── millcheck.json
│   └── api-gateway.json
│
├── sessions/
│   ├── millcheck/
│   │   ├── 20260402_101500_001.json
│   │   └── 20260402_102045_002.json
│   └── api-gateway/
│       └── 20260402_103200_001.json
│
├── knowledge/
│   ├── rust_patterns.json
│   └── project_context.json
│
├── agent_logs/
│   └── 2026-04-02.jsonl
│
└── shims/
    ├── claude
    ├── amp
    ├── gemini
    └── vibe
```

### Project Schema

```json
{
  "name": "millcheck",
  "goal": "validate mill test certificates",
  "constraints": "must handle null dates",
  "decisions": ["use rust for parsing", "error on invalid dates"],
  "notes": "critical path for Q1"
}
```

### Session Schema

```json
{
  "timestamp": "2026-04-02T10:15:00Z",
  "prompt": "fix null date panic",
  "context_summary": "project + 2 sessions",
  "model": "ollama/llama3",
  "response": "...",
  "duration_ms": 2450
}
```

### Agent Log Schema (JSONL)

```json
{
  "timestamp": "2026-04-02T10:15:00Z",
  "agent": "amp",
  "command": "amp ask fix parser bug",
  "response": "...",
  "thread_id": "T-...",
  "indexed": true
}
```

---

## 🏗️ Architecture

### Modules

```text
src/
├── executor/          # Model execution layer
│   ├── ollama.rs      # Ollama executor
│   ├── gemini.rs      # Gemini executors
│   ├── claude.rs      # Claude executors
│   ├── mistral.rs     # Mistral executor
│   ├── amp.rs         # Amp agent executor
│   ├── vibe.rs        # Vibe executor
│   ├── generic.rs     # Custom command executor
│   └── mod.rs         # Trait & Model enum
│
├── context/           # Context building
│   ├── builder.rs     # Assembles context from project + sessions + knowledge
│   └── mod.rs
│
├── data/              # Persistent storage
│   ├── project.rs     # Project CRUD
│   ├── session.rs     # Session CRUD
│   ├── knowledge.rs   # Knowledge CRUD
│   ├── agent_log.rs   # Agent log writing
│   └── mod.rs
│
├── ui/                # Terminal interface
│   ├── app.rs         # App state & event handling
│   ├── render.rs      # Rendering logic
│   ├── events.rs      # Keybindings
│   ├── theme.rs       # Colors
│   └── mod.rs
│
├── watcher/           # Log monitoring
│   ├── mod.rs
│   ├── log_watcher.rs # File watcher for agent logs
│   └── shim.rs        # Shim installation
│
├── config.rs          # Configuration
├── error.rs           # Error types
└── main.rs            # Entry point
```

### Data Flow

```
User Input (keybinding)
    ↓
Event Handler (ui/events.rs)
    ↓
App State (ui/app.rs)
    ↓
If "Run": Context Builder → Executor
    ↓
Response → Session Logger
    ↓
UI Render (ui/render.rs)
```

---

## ⚙️ Design Principles

1. **Deterministic context** — No magic injection. You see exactly what's being sent via checkboxes.
2. **Small > big** — Default is project + 2 sessions. Increase only when needed.
3. **Single execution path** — All models use unified `executor::run(model, prompt)` interface.
4. **Mandatory logging** — Every session is saved before UI confirmation.
5. **UI ≠ Logic** — Interface only reads/renders state. Logic lives in core modules.
6. **Plain text storage** — Easy to inspect, version control, and migrate.
7. **Extensible** — Adding models means implementing one `Executor` trait.

---

## 🔧 Extending

### Adding a New Model

1. **Create executor** (`src/executor/mymodel.rs`):

```rust
use crate::executor::Executor;
use anyhow::Result;

pub struct MyModelExecutor;

impl MyModelExecutor {
    pub fn new() -> Self { Self }
}

impl Executor for MyModelExecutor {
    fn name(&self) -> &str { "mymodel" }
    
    fn run(&self, prompt: &str) -> Result<String> {
        // spawn subprocess, call API, etc.
        // return stdout/response
        todo!()
    }
}
```

2. **Register in `src/executor/mod.rs`**:

```rust
mod mymodel;
pub use mymodel::MyModelExecutor;
```

3. **Add to `Model` enum**:

```rust
pub enum Model {
    // ... existing variants ...
    MyModel,
}
```

4. **Add to `Model::executor()`**:

```rust
Model::MyModel => Box::new(mymodel::MyModelExecutor::new()),
```

5. **Add to `Model::presets()`**:

```rust
Model::MyModel,
```

Done — it appears in the model selector automatically.

---

### Adding a New Shim

Shims are logging wrappers for external agent CLIs. See `src/watcher/shim.rs` for template.

---

## 📊 Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` 0.26 | Terminal UI |
| `crossterm` 0.27 | Terminal events & control |
| `serde_json` 1 | JSON storage |
| `chrono` 0.4 | Timestamps |
| `anyhow` 1 | Error handling |
| `dirs` 5 | XDG dirs |
| `notify` 6 | File watcher |

---

## 🚧 Known Limitations

- **No embeddings** — keyword search only
- **No cloud sync** — local files only
- **No multi-user** — single-user TUI
- **Single-threaded** — execution is blocking
- **TTY only** — requires interactive terminal

---

## 🧭 Roadmap

- [ ] Semantic search via embeddings
- [ ] Prompt templates and macros
- [ ] Multi-agent orchestration
- [ ] Response streaming
- [ ] Performance optimizations
- [ ] Web UI option
- [ ] Collaborative sessions

---

## 🐛 Troubleshooting

### "matis-mem: requires an interactive terminal"

Run from a real terminal, not a non-interactive shell.

### Models not showing in selector

Ensure executors are properly registered in `src/executor/mod.rs` and compiled.

### Sessions not saving

Check `~/.matis-mem/sessions/` directory exists and is writable.

### Keybindings not responding

Ensure terminal supports raw mode (most modern terminals do).

---

## 📜 License

MIT

---

## Final Note

**This tool only works if you actually use it consistently.**

If you bypass it and go back to raw AI commands, you're back to stateless chaos.

**The discipline is the feature.**
