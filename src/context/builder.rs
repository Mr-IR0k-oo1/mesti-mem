use anyhow::Result;

use crate::data::{Knowledge, Project, Session};

/// Which sources to include when building context
#[derive(Debug, Clone)]
pub struct ContextOptions {
    pub include_project: bool,
    pub recent_sessions: usize,   // 0 = none
    pub knowledge_query: Option<String>, // None = skip
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            include_project: true,
            recent_sessions: 2,
            knowledge_query: None,
        }
    }
}

/// The assembled context string + a human-readable summary
pub struct BuiltContext {
    pub text: String,
    pub summary: String,
}

/// Build context for a prompt.
///
/// Rule: minimal and relevant. Never load everything.
///
///   CONTEXT =
///     project core (if include_project)
///   + recent N sessions (if recent_sessions > 0)
///   + knowledge matches (if knowledge_query set)
pub fn build(
    project: &Project,
    opts: &ContextOptions,
) -> Result<BuiltContext> {
    let mut parts: Vec<String> = Vec::new();
    let mut summary_parts: Vec<String> = Vec::new();

    // 1. Project core — always first if enabled
    if opts.include_project {
        parts.push(project.as_context());
        summary_parts.push(format!("project:{}", project.name));
    }

    // 2. Recent sessions
    if opts.recent_sessions > 0 {
        let sessions = Session::last_n(&project.name, opts.recent_sessions)?;
        if !sessions.is_empty() {
            parts.push("[RECENT SESSIONS]".to_string());
            for s in &sessions {
                parts.push(s.as_context());
            }
            summary_parts.push(format!("sessions:{}", sessions.len()));
        }
    }

    // 3. Knowledge search
    if let Some(ref q) = opts.knowledge_query {
        let hits = Knowledge::search(q)?;
        if !hits.is_empty() {
            parts.push("[RELEVANT KNOWLEDGE]".to_string());
            for k in &hits {
                parts.push(k.as_context());
            }
            summary_parts.push(format!("knowledge:{}", hits.len()));
        }
    }

    Ok(BuiltContext {
        text: parts.join("\n"),
        summary: summary_parts.join(", "),
    })
}

/// Format the final prompt that gets sent to the model.
/// Structure: context block + separator + user question.
pub fn format_prompt(context: &BuiltContext, user_prompt: &str) -> String {
    if context.text.is_empty() {
        return user_prompt.to_string();
    }
    format!(
        "{}\n\n---\n\n[QUESTION]\n{}",
        context.text.trim(),
        user_prompt.trim()
    )
}
