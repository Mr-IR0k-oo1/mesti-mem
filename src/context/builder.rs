use anyhow::Result;
use crate::data::{Knowledge, Project, Session};

#[derive(Debug, Clone)]
pub struct ContextOptions {
    pub include_project:  bool,
    pub recent_sessions:  usize,
    pub knowledge_query:  Option<String>,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self { include_project: true, recent_sessions: 2, knowledge_query: None }
    }
}

pub struct BuiltContext {
    pub text:    String,
    pub summary: String,
}

pub fn build(project: &Project, opts: &ContextOptions) -> Result<BuiltContext> {
    let mut parts   = vec![];
    let mut summary = vec![];

    if opts.include_project {
        parts.push(project.as_context());
        summary.push(format!("project:{}", project.name));
    }

    if opts.recent_sessions > 0 {
        let sessions = Session::last_n(&project.name, opts.recent_sessions)?;
        if !sessions.is_empty() {
            parts.push("[RECENT SESSIONS]".into());
            for s in &sessions { parts.push(s.as_context()); }
            summary.push(format!("sessions:{}", sessions.len()));
        }
    }

    if let Some(ref q) = opts.knowledge_query {
        let hits = Knowledge::search(q)?;
        if !hits.is_empty() {
            parts.push("[RELEVANT KNOWLEDGE]".into());
            for k in &hits { parts.push(k.as_context()); }
            summary.push(format!("knowledge:{}", hits.len()));
        }
    }

    Ok(BuiltContext { text: parts.join("\n"), summary: summary.join(", ") })
}

pub fn format_prompt(ctx: &BuiltContext, user_prompt: &str) -> String {
    if ctx.text.is_empty() {
        return user_prompt.to_string();
    }
    format!("{}\n\n---\n\n[QUESTION]\n{}", ctx.text.trim(), user_prompt.trim())
}
