use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::data::{Knowledge, Project};
use super::app::{App, ConfirmAction, Focus, Popup, RunState};

pub fn handle(event: &Event, app: &mut App) {
    let Event::Key(key) = event else { return };

    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // Handle popup first
    if !matches!(app.popup, Popup::None) {
        handle_popup(key.code, key.modifiers, app);
        return;
    }

    // Global regardless of focus
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('r'))
        | (KeyModifiers::NONE,  KeyCode::F(5)) => {
            app.run();
            return;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            // New project shortcut
            app.popup = Popup::NewProject {
                name_buf: String::new(),
                goal_buf: String::new(),
                field: 0,
            };
            return;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
            // Add knowledge shortcut
            app.popup = Popup::AddKnowledge {
                topic_buf: String::new(),
                note_buf: String::new(),
            };
            return;
        }
        _ => {}
    }

    // Tab cycles focus: Projects → Prompt → Context → Model → Response → Prompt
    if key.code == KeyCode::Tab && key.modifiers == KeyModifiers::NONE {
        app.focus = match app.focus {
            Focus::Projects  => Focus::Prompt,
            Focus::Prompt    => Focus::Context,
            Focus::Context   => Focus::Model,
            Focus::Model     => Focus::Response,
            Focus::Response  => Focus::Prompt,
        };
        return;
    }
    if key.code == KeyCode::BackTab {
        app.focus = match app.focus {
            Focus::Projects  => Focus::Response,
            Focus::Prompt    => Focus::Projects,
            Focus::Context   => Focus::Prompt,
            Focus::Model     => Focus::Context,
            Focus::Response  => Focus::Model,
        };
        return;
    }

    match app.focus {
        Focus::Projects => handle_projects(key.code, key.modifiers, app),
        Focus::Prompt   => handle_prompt(key.code, key.modifiers, app),
        Focus::Context  => handle_context(key.code, app),
        Focus::Model    => handle_model(key.code, app),
        Focus::Response => handle_response(key.code, app),
    }
}

fn handle_popup(code: KeyCode, modifiers: KeyModifiers, app: &mut App) {
    match &mut app.popup {
        Popup::NewProject { name_buf, goal_buf, field } => {
            let field_val = *field;
            match code {
                KeyCode::Esc => app.popup = Popup::None,
                KeyCode::Tab => *field = if field_val == 0 { 1 } else { 0 },
                KeyCode::Enter => {
                    let name = name_buf.trim().to_string();
                    let goal = goal_buf.trim().to_string();
                    if !name.is_empty() && !goal.is_empty() {
                        let proj = Project::new(&name, &goal);
                        if let Err(e) = proj.save() {
                            app.status = Some((format!("Save error: {}", e), true));
                        } else {
                            app.status = Some((format!("Project '{}' created", name), false));
                        }
                        app.popup = Popup::None;
                        app.reload_projects();
                        // Select newly created project
                        if let Some(idx) = app.projects.iter().position(|p| p == &name) {
                            app.project_idx = idx;
                            app.reload_active();
                        }
                    }
                }
                KeyCode::Backspace => {
                    if field_val == 0 { name_buf.pop(); }
                    else              { goal_buf.pop(); }
                }
                KeyCode::Char(c) => {
                    if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                        if field_val == 0 { name_buf.push(c); }
                        else              { goal_buf.push(c); }
                    }
                }
                _ => {}
            }
        }

        Popup::AddKnowledge { topic_buf, note_buf } => {
            match code {
                KeyCode::Esc => app.popup = Popup::None,
                KeyCode::Enter => {
                    let topic = topic_buf.trim().to_string();
                    let note  = note_buf.trim().to_string();
                    if !topic.is_empty() && !note.is_empty() {
                        let mut k = Knowledge::load(&topic)
                            .unwrap_or_else(|_| Knowledge::new(&topic));
                        k.notes.push(note);
                        if let Err(e) = k.save() {
                            app.status = Some((format!("Save error: {}", e), true));
                        } else {
                            app.status = Some((format!("Knowledge '{}' saved", topic), false));
                        }
                        app.popup = Popup::None;
                    }
                }
                KeyCode::Backspace => { note_buf.pop(); }
                KeyCode::Char(c) => {
                    if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                        if topic_buf.is_empty() || note_buf.is_empty() && !topic_buf.is_empty() {
                            note_buf.push(c);
                        } else {
                            note_buf.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Popup::Confirm { on_yes, .. } => {
            match code {
                KeyCode::Esc | KeyCode::Char('n') => app.popup = Popup::None,
                KeyCode::Enter | KeyCode::Char('y') => {
                    let action = on_yes.clone();
                    app.popup = Popup::None;
                    match action {
                        ConfirmAction::DeleteProject(name) => {
                            if let Err(e) = Project::delete(&name) {
                                app.status = Some((format!("Delete error: {}", e), true));
                            } else {
                                app.status = Some((format!("Deleted '{}'", name), false));
                                app.reload_projects();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Popup::None => {}
    }
}

fn handle_projects(code: KeyCode, _mods: KeyModifiers, app: &mut App) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.project_idx + 1 < app.projects.len() {
                app.project_idx += 1;
                app.reload_active();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.project_idx > 0 {
                app.project_idx -= 1;
                app.reload_active();
            }
        }
        KeyCode::Char('n') => {
            app.popup = Popup::NewProject {
                name_buf: String::new(),
                goal_buf: String::new(),
                field: 0,
            };
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if let Some(name) = app.projects.get(app.project_idx).cloned() {
                app.popup = Popup::Confirm {
                    message: format!("Delete project '{}'?", name),
                    on_yes: ConfirmAction::DeleteProject(name),
                };
            }
        }
        KeyCode::Enter => {
            app.reload_active();
            app.focus = Focus::Prompt;
        }
        _ => {}
    }
}

fn handle_prompt(code: KeyCode, mods: KeyModifiers, app: &mut App) {
    match code {
        KeyCode::Char(c) => {
            if mods == KeyModifiers::NONE || mods == KeyModifiers::SHIFT {
                app.prompt_push(c);
            }
        }
        KeyCode::Backspace => app.prompt_backspace(),
        KeyCode::Left      => app.prompt_cursor_left(),
        KeyCode::Right     => app.prompt_cursor_right(),
        KeyCode::Enter => {
            // Shift+Enter = newline in prompt, plain Enter = run
            if mods.contains(KeyModifiers::SHIFT) {
                app.prompt_push('\n');
            } else {
                app.run();
            }
        }
        _ => {}
    }
}

fn handle_context(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Char(' ') => app.ctx_include_project = !app.ctx_include_project,
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.ctx_recent_n < 10 { app.ctx_recent_n += 1; }
        }
        KeyCode::Char('-') => {
            if app.ctx_recent_n > 0 { app.ctx_recent_n -= 1; }
        }
        KeyCode::Char('k') => app.ctx_knowledge_query = !app.ctx_knowledge_query,
        _ => {}
    }
}

fn handle_model(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Char('j') | KeyCode::Down  => app.cycle_model_forward(),
        KeyCode::Char('k') | KeyCode::Up    => app.cycle_model_backward(),
        _ => {}
    }
}

fn handle_response(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Char('j') | KeyCode::Down  => app.response_scroll_down(),
        KeyCode::Char('k') | KeyCode::Up    => app.response_scroll_up(),
        KeyCode::Char('g') => app.response_scroll = 0,
        KeyCode::Char('G') => {
            let lines = app.response.lines().count();
            app.response_scroll = lines.saturating_sub(1);
        }
        // clear prompt and run again
        KeyCode::Char('c') => {
            if app.run_state == RunState::Done {
                app.prompt.clear();
                app.cursor = 0;
                app.run_state = RunState::Idle;
                app.response.clear();
                app.focus = Focus::Prompt;
            }
        }
        _ => {}
    }
}
