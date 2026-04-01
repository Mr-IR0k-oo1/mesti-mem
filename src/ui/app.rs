use anyhow::Result;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::context::{build, format_prompt, ContextOptions};
use crate::data::{Project, Session};
use crate::executor::{self, Model};

// ── Focus ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Focus {
    Projects,
    Prompt,
    Context,
    Model,
    Response,
}

// ── Run state ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    Idle,
    Running,
    Done,
    Error(String),
}

// ── Popup ─────────────────────────────────────────────────────────────────────

pub enum Popup {
    None,
    NewProject { name_buf: String, goal_buf: String, field: usize },
    AddKnowledge { topic_buf: String, note_buf: String },
    Confirm { message: String, on_yes: ConfirmAction },
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteProject(String),
}

// ── Exec channel ─────────────────────────────────────────────────────────────

pub enum ExecMsg {
    Done { response: String, duration_ms: u64 },
    Error(String),
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    // Project list
    pub projects: Vec<String>,
    pub project_idx: usize,
    pub active_project: Option<Project>,

    // Prompt
    pub prompt: String,
    pub cursor: usize,    // byte position in prompt

    // Context options
    pub ctx_include_project: bool,
    pub ctx_recent_n: usize,
    pub ctx_knowledge_query: bool,  // if true, use prompt as query
    pub ctx_n_choices: [usize; 4],  // [0,1,2,3,4] for recent sessions selector

    // Model
    pub models: Vec<Model>,
    pub model_idx: usize,

    // Response
    pub response: String,
    pub response_scroll: usize,

    // Run state
    pub run_state: RunState,
    pub run_tx: Option<Sender<()>>,     // send () to cancel (future)
    pub exec_rx: Option<Receiver<ExecMsg>>,

    // Focus
    pub focus: Focus,

    // Popup
    pub popup: Popup,

    // Status bar
    pub status: Option<(String, bool)>,  // (message, is_error)

    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let projects = Project::list().unwrap_or_default();
        let active_project = if let Some(name) = projects.first() {
            Project::load(name).ok()
        } else {
            None
        };

        Ok(App {
            projects,
            project_idx: 0,
            active_project,

            prompt: String::new(),
            cursor: 0,

            ctx_include_project: true,
            ctx_recent_n: 2,
            ctx_knowledge_query: false,
            ctx_n_choices: [0, 1, 2, 3],  // 4 elements

            models: Model::all_presets(),
            model_idx: 0,

            response: String::new(),
            response_scroll: 0,

            run_state: RunState::Idle,
            run_tx: None,
            exec_rx: None,

            focus: Focus::Prompt,

            popup: Popup::None,
            status: None,
            should_quit: false,
        })
    }

    pub fn reload_projects(&mut self) {
        self.projects = Project::list().unwrap_or_default();
        self.project_idx = self.project_idx.min(self.projects.len().saturating_sub(1));
        self.reload_active();
    }

    pub fn reload_active(&mut self) {
        self.active_project = self.projects
            .get(self.project_idx)
            .and_then(|name| Project::load(name).ok());
    }

    pub fn selected_model(&self) -> &Model {
        &self.models[self.model_idx]
    }

    pub fn context_opts(&self) -> ContextOptions {
        ContextOptions {
            include_project: self.ctx_include_project,
            recent_sessions: self.ctx_recent_n,
            knowledge_query: if self.ctx_knowledge_query && !self.prompt.is_empty() {
                Some(self.prompt.clone())
            } else {
                None
            },
        }
    }

    /// Fire off the execution in a background thread.
    /// Results come back via exec_rx.
    pub fn run(&mut self) {
        if self.run_state == RunState::Running { return; }
        if self.prompt.trim().is_empty() {
            self.status = Some(("Enter a prompt first".into(), true));
            return;
        }
        let Some(ref project) = self.active_project.clone() else {
            self.status = Some(("Select or create a project first".into(), true));
            return;
        };

        let opts = self.context_opts();
        let project = project.clone();
        let prompt = self.prompt.clone();
        let model = self.selected_model().clone();

        // Build context on the main thread (fast, disk read only)
        let ctx = match build(&project, &opts) {
            Ok(c) => c,
            Err(e) => {
                self.run_state = RunState::Error(format!("Context error: {}", e));
                return;
            }
        };
        let full_prompt = format_prompt(&ctx, &prompt);
        let ctx_summary = ctx.summary.clone();
        let project_name = project.name.clone();
        let model_name = model.display_name();

        let (tx, rx) = mpsc::channel();
        self.exec_rx = Some(rx);
        self.run_state = RunState::Running;
        self.response.clear();
        self.response_scroll = 0;

        thread::spawn(move || {
            let start = Instant::now();
            match executor::run(&model, &full_prompt) {
                Ok(response) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    // Log the session
                    let session = Session::new(
                        &project_name,
                        &model_name,
                        &prompt,
                        &ctx_summary,
                        &response,
                        duration_ms,
                    );
                    let _ = session.save();
                    let _ = tx.send(ExecMsg::Done { response, duration_ms });
                }
                Err(e) => {
                    let _ = tx.send(ExecMsg::Error(e.to_string()));
                }
            }
        });
    }

    /// Poll the exec channel. Call this on every tick.
    pub fn poll_exec(&mut self) {
        if self.run_state != RunState::Running { return; }
        let Some(ref rx) = self.exec_rx else { return };

        match rx.try_recv() {
            Ok(ExecMsg::Done { response, duration_ms }) => {
                self.response = response;
                self.run_state = RunState::Done;
                self.exec_rx = None;
                self.status = Some((
                    format!("Done in {}ms — session saved", duration_ms),
                    false,
                ));
                self.focus = Focus::Response;
            }
            Ok(ExecMsg::Error(e)) => {
                self.run_state = RunState::Error(e.clone());
                self.exec_rx = None;
                self.status = Some((e, true));
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.run_state = RunState::Error("Executor thread died".into());
                self.exec_rx = None;
            }
        }
    }

    // ── Input handling ────────────────────────────────────────────────────────

    pub fn prompt_push(&mut self, c: char) {
        self.prompt.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn prompt_backspace(&mut self) {
        if self.cursor == 0 { return; }
        // Find previous char boundary
        let mut new_cursor = self.cursor - 1;
        while !self.prompt.is_char_boundary(new_cursor) {
            new_cursor -= 1;
        }
        self.prompt.remove(new_cursor);
        self.cursor = new_cursor;
    }

    pub fn prompt_cursor_left(&mut self) {
        if self.cursor == 0 { return; }
        let mut c = self.cursor - 1;
        while !self.prompt.is_char_boundary(c) { c -= 1; }
        self.cursor = c;
    }

    pub fn prompt_cursor_right(&mut self) {
        if self.cursor >= self.prompt.len() { return; }
        let mut c = self.cursor + 1;
        while !self.prompt.is_char_boundary(c) { c += 1; }
        self.cursor = c;
    }

    pub fn cycle_model_forward(&mut self) {
        self.model_idx = (self.model_idx + 1) % self.models.len();
    }

    pub fn cycle_model_backward(&mut self) {
        self.model_idx = if self.model_idx == 0 {
            self.models.len() - 1
        } else {
            self.model_idx - 1
        };
    }

    pub fn response_scroll_down(&mut self) {
        let lines = self.response.lines().count();
        if self.response_scroll + 1 < lines {
            self.response_scroll += 1;
        }
    }

    pub fn response_scroll_up(&mut self) {
        if self.response_scroll > 0 {
            self.response_scroll -= 1;
        }
    }
}
