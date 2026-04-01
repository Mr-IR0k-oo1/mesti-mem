# MESTI-MEM

A deterministic, terminal-native AI execution environment built in Rust.

This is not another wrapper around LLMs.
MESTI-MEM is a **stateful system** that enforces memory, context discipline, and reproducible execution across multiple AI backends.

---

## 🚧 Philosophy

AI tools are stateless by default. That leads to:

* repeated context injection
* lost reasoning
* inconsistent outputs
* zero continuity

MESTI-MEM fixes this by enforcing:

* **persistent memory**
* **deterministic context building**
* **mandatory session logging**
* **model-agnostic execution**

No magic. No hidden behavior. Everything is explicit and inspectable.

---

## 🧠 Core Concepts

### 1. Memory is First-Class

All state is stored locally:

```
~/.mesti-mem/
├── projects/
├── sessions/
├── knowledge/
└── prompts/
```

* **Projects** → long-lived context
* **Sessions** → interaction history
* **Knowledge** → reusable facts
* **Prompts** → reusable templates

---

### 2. Deterministic Context

Context is constructed explicitly:

```
[PROJECT]
+ project.json

[RECENT SESSIONS]
+ last N sessions

[RELEVANT KNOWLEDGE]
+ keyword search (optional)
```

No hidden retrieval. No automatic hallucinated summaries.

---

### 3. Model-Agnostic Execution

All models are accessed through a single interface:

```rust
run(model, prompt)
```

Supported:

* Ollama (local)
* Gemini CLI

Adding new models requires implementing the `Executor` trait.

---

### 4. Mandatory Logging

Every execution produces a session log:

```json
{
  "timestamp": "...",
  "prompt": "...",
  "context_used": "...",
  "response": "..."
}
```

No logs = no memory = no system.

---

### 5. TUI = Control Surface

The terminal UI is strictly for:

* selecting projects
* editing prompts
* toggling context
* choosing models

All logic lives outside the UI.

---

## 📁 Project Structure

```
mesti-mem/
├── src/
│   ├── main.rs              # terminal setup, main loop
│   ├── config.rs            # ~/.ai-os paths (OnceLock)
│   ├── error.rs             # typed errors
│
│   ├── data/
│   │   ├── project.rs       # Project CRUD + context export
│   │   ├── session.rs       # Session logging + retrieval
│   │   └── knowledge.rs     # Knowledge store + search
│
│   ├── context/
│   │   └── builder.rs       # context assembly logic
│
│   ├── executor/
│   │   ├── mod.rs           # Executor trait + Model enum
│   │   ├── ollama.rs        # ollama subprocess execution
│   │   └── gemini.rs        # gemini CLI execution
│
│   └── ui/
│       ├── app.rs           # app state + async execution
│       ├── events.rs        # keybindings
│       ├── render.rs        # layout rendering
│       └── theme.rs         # styling
```

---

## ⚙️ Installation

### Requirements

* Rust (>= 1.75)
* Ollama (optional but recommended)
* Gemini CLI (optional)

---

### Build

```bash
git clone https://github.com/Mr-IR0k-oo1/mesti-mem.git
cd mesti-mem
cargo build --release
```

---

### Fix for Unicode Dependency Issue

If build fails:

```bash
cargo update unicode-segmentation --precise 1.12.0
cargo build --release
```

This is required for Rust 1.75 compatibility.

---

## 🚀 Usage

Run:

```bash
./target/release/ai-os
```

---

## 🖥️ TUI Overview

```
[ Project: millcheck ]

Prompt:
> improve parsing logic

Context:
[x] project
[x] last 2 sessions
[ ] knowledge

Model:
> ollama

[ RUN ]
```

---

## 🔁 Execution Flow

1. Select project
2. Enter prompt
3. Build context
4. Execute model
5. Display response
6. Log session automatically

---

## 🔧 Configuration

Config is stored in:

```
~/.mesti-mem/
```

Context settings:

* `ctx_recent_n` → number of past sessions included
* knowledge inclusion → toggleable in UI

---

## 🧩 Extending the System

### Add a New Model

Implement the `Executor` trait:

```rust
trait Executor {
    fn run(prompt: &str) -> Result<String>;
}
```

Register it in:

```
executor/mod.rs
```

---

### Modify Context Logic

Edit:

```
context/builder.rs
```

This is the only place where context is assembled.

---

## ⚠️ Design Constraints

* No implicit behavior
* No automatic summarization
* No hidden memory injection
* No UI-driven logic

Everything must be:

* explicit
* deterministic
* reproducible

---

## 🧱 What This System Solves

* Eliminates repeated context setup
* Maintains continuity across sessions
* Enables cross-model workflows
* Provides inspectable AI interactions

---

## 💀 What It Does NOT Do

* It does not improve bad prompts
* It does not “think for you”
* It does not replace engineering discipline

---

## 📌 Status

Core system implemented:

* [x] Memory layer
* [x] Context builder
* [x] Executor abstraction
* [x] TUI control surface
* [x] Session logging

---

## 🧭 Next Steps

* smarter retrieval (only if needed)
* prompt templating
* multi-agent chaining
* benchmarking outputs

---

## 📜 License

MIT

---

## Final Note

This system is only useful if you actually use it consistently.

If you bypass it and go back to raw CLI usage, it becomes just another abandoned tool.


