use crossterm::event::{Event, KeyCode, KeyModifiers};
use crate::data::{Knowledge, Project};
use crate::watcher::shim;
use super::app::{App, ConfirmAction, ExportMode, Focus, Popup, RunState, Tab};

pub fn handle(event: &Event, app: &mut App) {
    let Event::Key(key) = event else { return };

    // Always: Ctrl+C
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // Popup routing first
    if !matches!(app.popup, Popup::None) {
        handle_popup(key.code, key.modifiers, app);
        return;
    }

    // Global Ctrl shortcuts
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('r') => { app.run(); return; }
            KeyCode::Char('n') => {
                app.popup = Popup::NewProject { name_buf: String::new(), goal_buf: String::new(), field: 0 };
                return;
            }
            KeyCode::Char('k') => {
                app.popup = Popup::AddKnowledge {
                    topic_buf: String::new(), note_buf: String::new(), tag_buf: String::new(),
                    active_field: 0, error: None,
                };
                return;
            }
            KeyCode::Char('m') => { app.refresh_models(); return; }
            KeyCode::Char('i') => {
                app.popup = Popup::ImportKnowledge { path_buf: String::new(), error: None };
                return;
            }
            KeyCode::Char('e') if app.tab == Tab::Knowledge => {
                let default = crate::config::data_dir()
                    .join("knowledge-export.md").to_string_lossy().to_string();
                app.popup = Popup::ExportKnowledge { path_buf: default, mode: ExportMode::Bundle };
                return;
            }
            _ => {}
        }
    }

    if key.code == KeyCode::F(5) { app.run(); return; }

    // Number keys switch tabs
    match key.code {
        KeyCode::Char('1') => { app.switch_tab(Tab::Run);       return; }
        KeyCode::Char('2') => { app.switch_tab(Tab::Agents);    return; }
        KeyCode::Char('3') => { app.switch_tab(Tab::Shims);     return; }
        KeyCode::Char('4') => { app.switch_tab(Tab::Knowledge); return; }
        KeyCode::Tab => {
            app.focus = cycle_focus_fwd(&app.focus, &app.tab);
            return;
        }
        KeyCode::BackTab => {
            app.focus = cycle_focus_bwd(&app.focus, &app.tab);
            return;
        }
        // 'q' quits everywhere except the prompt field
        KeyCode::Char('q') if !matches!(app.focus, Focus::Prompt) => {
            app.should_quit = true;
            return;
        }
        _ => {}
    }

    match app.tab {
        Tab::Run       => handle_run(key.code, key.modifiers, app),
        Tab::Agents    => handle_agents(key.code, app),
        Tab::Shims     => handle_shims(key.code, app),
        Tab::Knowledge => handle_knowledge(key.code, key.modifiers, app),
    }
}

fn cycle_focus_fwd(f: &Focus, tab: &Tab) -> Focus {
    match tab {
        Tab::Run => match f {
            Focus::Projects => Focus::Prompt,
            Focus::Prompt   => Focus::Context,
            Focus::Context  => Focus::Model,
            Focus::Model    => Focus::Response,
            _               => Focus::Projects,
        },
        Tab::Agents    => if *f == Focus::AgentList { Focus::AgentDetail } else { Focus::AgentList },
        Tab::Shims     => Focus::ShimList,
        Tab::Knowledge => if *f == Focus::KnowledgeList { Focus::KnowledgeDetail } else { Focus::KnowledgeList },
    }
}

fn cycle_focus_bwd(f: &Focus, tab: &Tab) -> Focus {
    match tab {
        Tab::Run => match f {
            Focus::Projects => Focus::Response,
            Focus::Prompt   => Focus::Projects,
            Focus::Context  => Focus::Prompt,
            Focus::Model    => Focus::Context,
            Focus::Response => Focus::Model,
            _               => Focus::Prompt,
        },
        _ => cycle_focus_fwd(f, tab),
    }
}

// ── Popup ─────────────────────────────────────────────────────────────────────

fn handle_popup(code: KeyCode, mods: KeyModifiers, app: &mut App) {
    match &mut app.popup {
        Popup::NewProject { name_buf, goal_buf, field } => {
            let f = *field;
            match code {
                KeyCode::Esc   => app.popup = Popup::None,
                KeyCode::Tab   => *field = if f == 0 { 1 } else { 0 },
                KeyCode::Enter => {
                    let name = name_buf.trim().to_string();
                    let goal = goal_buf.trim().to_string();
                    if name.len() >= 2 && !goal.is_empty() {
                        match Project::new(&name, &goal).save() {
                            Ok(_)  => app.set_status(format!("Project '{}' created", name), false),
                            Err(e) => app.set_status(format!("Error: {}", e), true),
                        }
                        app.popup = Popup::None;
                        app.reload_projects();
                        if let Some(i) = app.projects.iter().position(|p| p == &name) {
                            app.project_idx = i;
                            app.project_list_state.select(Some(i));
                            app.reload_active();
                        }
                    }
                }
                KeyCode::Backspace => { if f == 0 { name_buf.pop(); } else { goal_buf.pop(); } }
                KeyCode::Char(c) if is_text(mods) => {
                    if f == 0 { name_buf.push(c); } else { goal_buf.push(c); }
                }
                _ => {}
            }
        }

        Popup::AddKnowledge { topic_buf, note_buf, tag_buf, active_field, error } => {
            let f = *active_field;
            match code {
                KeyCode::Esc => app.popup = Popup::None,
                KeyCode::Tab    => *active_field = (f + 1) % 3,
                KeyCode::BackTab => *active_field = if f == 0 { 2 } else { f - 1 },
                KeyCode::Enter => {
                    match Knowledge::validate_topic(topic_buf) {
                        Err(e) => { *error = Some(e.to_string()); }
                        Ok(_) => {
                            let topic = topic_buf.trim().to_string();
                            let note  = note_buf.trim().to_string();
                            let tags: Vec<String> = tag_buf.split(',')
                                .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                            let mut k = Knowledge::load(&topic)
                                .unwrap_or_else(|_| Knowledge::new(&topic));
                            if !note.is_empty() { k.notes.push(note); }
                            k.tags.extend(tags);
                            match k.save() {
                                Ok(_) => {
                                    app.set_status(format!("Saved: {}", topic), false);
                                    app.popup = Popup::None;
                                    app.reload_knowledge();
                                    if let Some(i) = app.knowledge_topics.iter().position(|t| t == &topic) {
                                        app.knowledge_idx = i;
                                        app.knowledge_list_state.select(Some(i));
                                        app.refresh_knowledge_detail();
                                    }
                                }
                                Err(e) => { *error = Some(format!("Save failed: {}", e)); }
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    match f { 0=>{topic_buf.pop();*error=None;} 1=>{note_buf.pop();} _=>{tag_buf.pop();} }
                }
                KeyCode::Char(c) if is_text(mods) => {
                    match f { 0=>{topic_buf.push(c);*error=None;} 1=>{note_buf.push(c);} _=>{tag_buf.push(c);} }
                }
                _ => {}
            }
        }

        Popup::EditKnowledge { original_topic, topic_buf, note_buf, tag_buf, active_field, error } => {
            let f = *active_field;
            match code {
                KeyCode::Esc => app.popup = Popup::None,
                KeyCode::Tab    => *active_field = (f + 1) % 3,
                KeyCode::BackTab => *active_field = if f == 0 { 2 } else { f - 1 },
                KeyCode::Enter => {
                    match Knowledge::validate_topic(topic_buf) {
                        Err(e) => { *error = Some(e.to_string()); }
                        Ok(_) => {
                            let new_topic = topic_buf.trim().to_string();
                            let note      = note_buf.trim().to_string();
                            let new_tags: Vec<String> = tag_buf.split(',')
                                .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                            let mut k = Knowledge::load(original_topic)
                                .unwrap_or_else(|_| Knowledge::new(&new_topic));
                            k.topic = new_topic.clone();
                            if !note.is_empty() { k.notes.push(note); }
                            k.tags = new_tags;
                            if original_topic != &new_topic {
                                let _ = Knowledge::delete(original_topic);
                            }
                            match k.save() {
                                Ok(_) => {
                                    app.set_status(format!("Updated: {}", new_topic), false);
                                    app.popup = Popup::None;
                                    app.reload_knowledge();
                                }
                                Err(e) => { *error = Some(format!("Save failed: {}", e)); }
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    match f { 0=>{topic_buf.pop();*error=None;} 1=>{note_buf.pop();} _=>{tag_buf.pop();} }
                }
                KeyCode::Char(c) if is_text(mods) => {
                    match f { 0=>{topic_buf.push(c);*error=None;} 1=>{note_buf.push(c);} _=>{tag_buf.push(c);} }
                }
                _ => {}
            }
        }

        Popup::ImportKnowledge { path_buf, error } => {
            match code {
                KeyCode::Esc => app.popup = Popup::None,
                KeyCode::Enter => {
                    let path = std::path::PathBuf::from(expand_tilde(path_buf.trim()));
                    if path.is_dir() {
                        match Knowledge::import_from_dir(&path) {
                            Ok(topics) => {
                                app.reload_knowledge();
                                let mut lines = vec![format!("Imported {} entries:", topics.len())];
                                for t in &topics { lines.push(format!("  ✓ {}", t)); }
                                app.popup = Popup::Output { title: "Import Complete".into(), lines, scroll: 0 };
                            }
                            Err(e) => { *error = Some(e.to_string()); }
                        }
                    } else if path.is_file() {
                        match Knowledge::import_from_file(&path) {
                            Ok(topic) => {
                                app.reload_knowledge();
                                app.set_status(format!("Imported: {}", topic), false);
                                app.popup = Popup::None;
                            }
                            Err(e) => { *error = Some(e.to_string()); }
                        }
                    } else {
                        *error = Some(format!("Not found: {}", path.display()));
                    }
                }
                KeyCode::Backspace => { path_buf.pop(); *error = None; }
                KeyCode::Char(c) if is_text(mods) || matches!(c, '/'|'~'|'.'|'-'|'_') => {
                    path_buf.push(c); *error = None;
                }
                _ => {}
            }
        }

        Popup::ExportKnowledge { path_buf, mode } => {
            let mode = mode.clone();
            match code {
                KeyCode::Esc => app.popup = Popup::None,
                KeyCode::Enter => {
                    let path = std::path::PathBuf::from(expand_tilde(path_buf.trim()));
                    let result = match &mode {
                        ExportMode::All    => Knowledge::export_all(&path).map(|fs| format!("Exported {} files", fs.len())),
                        ExportMode::Bundle => Knowledge::export_bundle(&path).map(|n| format!("Bundled {} entries → {}", n, path.display())),
                        ExportMode::Single(t) => {
                            let t = t.clone();
                            Knowledge::export_to_file(&t, &path).map(|_| format!("Exported '{}' → {}", t, path.display()))
                        }
                    };
                    match result {
                        Ok(msg) => { app.set_status(msg, false); app.popup = Popup::None; }
                        Err(e)  => { app.set_status(format!("Export failed: {}", e), true); app.popup = Popup::None; }
                    }
                }
                KeyCode::Backspace => { path_buf.pop(); }
                KeyCode::Char(c) if is_text(mods) || matches!(c, '/'|'~'|'.'|'-'|'_') => { path_buf.push(c); }
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
                            let _ = Project::delete(&name);
                            app.set_status(format!("Deleted '{}'", name), false);
                            app.reload_projects();
                        }
                        ConfirmAction::DeleteKnowledge(topic) => {
                            let _ = Knowledge::delete(&topic);
                            app.set_status(format!("Deleted '{}'", topic), false);
                            app.reload_knowledge();
                        }
                        ConfirmAction::InstallShims => {
                            match shim::install_all() {
                                Ok((i, a, n)) => {
                                    let export = shim::path_export_line();
                                    let hint   = crate::platform::reload_hint();
                                    app.popup = Popup::Output {
                                        title: "Shim Install".into(),
                                        lines: vec![
                                            format!("  ✓ Installed:  {}", i),
                                            format!("  ─ Already:    {}", a),
                                            format!("  ○ Not found:  {}", n),
                                            String::new(),
                                            format!("  Add to shell rc:"),
                                            format!("    {}", export),
                                            String::new(),
                                            format!("  Then: {}", hint),
                                        ],
                                        scroll: 0,
                                    };
                                    app.reload_shim_status();
                                }
                                Err(e) => app.set_status(format!("Install failed: {}", e), true),
                            }
                        }
                        ConfirmAction::UninstallShims => {
                            match shim::uninstall_all() {
                                Ok(n) => { app.set_status(format!("Removed {} shim(s)", n), false); app.reload_shim_status(); }
                                Err(e) => app.set_status(format!("Error: {}", e), true),
                            }
                        }
                        ConfirmAction::RefreshModels => { app.refresh_models(); }
                    }
                }
                _ => {}
            }
        }

        Popup::Output { scroll, lines, .. } => {
            match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if *scroll + 1 < lines.len() { *scroll += 1; }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if *scroll > 0 { *scroll -= 1; }
                }
                _ => app.popup = Popup::None,
            }
        }

        Popup::None => {}
    }
}

// ── Run tab ───────────────────────────────────────────────────────────────────

fn handle_run(code: KeyCode, mods: KeyModifiers, app: &mut App) {
    match app.focus {
        Focus::Projects => match code {
            KeyCode::Char('k') | KeyCode::Up   => app.project_up(),
            KeyCode::Char('j') | KeyCode::Down => app.project_down(),
            KeyCode::Enter => { app.reload_active(); app.focus = Focus::Prompt; }
            KeyCode::Char('n') => {
                app.popup = Popup::NewProject { name_buf: String::new(), goal_buf: String::new(), field: 0 };
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(name) = app.projects.get(app.project_idx).cloned() {
                    app.popup = Popup::Confirm {
                        message: format!("Delete project '{}'?", name),
                        on_yes: ConfirmAction::DeleteProject(name),
                    };
                }
            }
            _ => {}
        }

        Focus::Prompt => match code {
            KeyCode::Enter => {
                if mods.contains(KeyModifiers::SHIFT) { app.prompt_push('\n'); } else { app.run(); }
            }
            KeyCode::Char(c) if is_text(mods) => app.prompt_push(c),
            KeyCode::Backspace => app.prompt_backspace(),
            KeyCode::Left      => app.prompt_left(),
            KeyCode::Right     => app.prompt_right(),
            _ => {}
        }

        Focus::Context => match code {
            KeyCode::Char(' ')                       => app.ctx_project   = !app.ctx_project,
            KeyCode::Char('+') | KeyCode::Char('=') => { if app.ctx_sessions < 10 { app.ctx_sessions += 1; } }
            KeyCode::Char('-')                       => { if app.ctx_sessions > 0  { app.ctx_sessions -= 1; } }
            KeyCode::Char('k')                       => app.ctx_knowledge = !app.ctx_knowledge,
            _ => {}
        }

        Focus::Model => match code {
            KeyCode::Char('j') | KeyCode::Down => app.model_next(),
            KeyCode::Char('k') | KeyCode::Up   => app.model_prev(),
            KeyCode::Char('r') => {
                app.popup = Popup::Confirm {
                    message: "Re-scan for installed models?".into(),
                    on_yes: ConfirmAction::RefreshModels,
                };
            }
            _ => {}
        }

        Focus::Response => match code {
            KeyCode::Char('j') | KeyCode::Down => app.response_down(),
            KeyCode::Char('k') | KeyCode::Up   => app.response_up(),
            KeyCode::Char('g')                  => app.response_scroll = 0,
            KeyCode::Char('G')                  => {
                app.response_scroll = app.response.lines().count().saturating_sub(1);
            }
            KeyCode::Char('c') if app.run_state == RunState::Done => {
                app.prompt.clear(); app.cursor = 0;
                app.run_state = RunState::Idle;
                app.response.clear(); app.focus = Focus::Prompt;
            }
            _ => {}
        }

        _ => {}
    }
}

// ── Agents ────────────────────────────────────────────────────────────────────

fn handle_agents(code: KeyCode, app: &mut App) {
    match app.focus {
        Focus::AgentList | Focus::AgentDetail => match code {
            KeyCode::Char('j') | KeyCode::Down => app.agent_down(),
            KeyCode::Char('k') | KeyCode::Up   => app.agent_up(),
            KeyCode::Char('r') => {
                app.agent_logs = crate::data::AgentLog::recent(200).unwrap_or_default();
                app.agent_idx  = 0;
                app.agent_list_state.select(Some(0));
                app.set_status("Refreshed agent logs", false);
            }
            KeyCode::Char('a') => { app.agent_filter = None; app.agent_idx = 0; }
            KeyCode::Char('f') => {
                let agents = crate::data::AgentLog::known_agents();
                if !agents.is_empty() {
                    app.agent_filter = match &app.agent_filter {
                        None => agents.first().cloned(),
                        Some(cur) => {
                            let pos = agents.iter().position(|a| a == cur).unwrap_or(0);
                            if pos + 1 < agents.len() { Some(agents[pos+1].clone()) } else { None }
                        }
                    };
                    app.agent_idx = 0;
                    app.agent_list_state.select(Some(0));
                }
            }
            _ => {}
        }
        _ => {}
    }
}

// ── Shims ─────────────────────────────────────────────────────────────────────

fn handle_shims(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.shim_down(),
        KeyCode::Char('k') | KeyCode::Up   => app.shim_up(),
        KeyCode::Char('i') => {
            app.popup = Popup::Confirm { message: "Install shims for all agents?".into(), on_yes: ConfirmAction::InstallShims };
        }
        KeyCode::Char('u') => {
            app.popup = Popup::Confirm { message: "Uninstall all matis-mem shims?".into(), on_yes: ConfirmAction::UninstallShims };
        }
        KeyCode::Char('r') => { app.reload_shim_status(); app.set_status("Refreshed", false); }
        _ => {}
    }
}

// ── Knowledge ─────────────────────────────────────────────────────────────────

fn handle_knowledge(code: KeyCode, mods: KeyModifiers, app: &mut App) {
    // Ctrl+K from knowledge tab opens add popup
    if mods.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('k') {
        app.popup = Popup::AddKnowledge {
            topic_buf: String::new(), note_buf: String::new(), tag_buf: String::new(),
            active_field: 0, error: None,
        };
        return;
    }

    match app.focus {
        Focus::KnowledgeList => match code {
            KeyCode::Char('j') | KeyCode::Down => app.knowledge_down(),
            KeyCode::Char('k') | KeyCode::Up   => app.knowledge_up(),
            KeyCode::Char('n') => {
                app.popup = Popup::AddKnowledge {
                    topic_buf: String::new(), note_buf: String::new(), tag_buf: String::new(),
                    active_field: 0, error: None,
                };
            }
            KeyCode::Char('e') => {
                if let Some(topic) = app.knowledge_topics.get(app.knowledge_idx).cloned() {
                    if let Ok(k) = Knowledge::load(&topic) {
                        app.popup = Popup::EditKnowledge {
                            original_topic: topic,
                            topic_buf: k.topic, note_buf: String::new(),
                            tag_buf: k.tags.join(", "), active_field: 1, error: None,
                        };
                    }
                }
            }
            KeyCode::Char('x') | KeyCode::Delete => {
                if let Some(topic) = app.knowledge_topics.get(app.knowledge_idx).cloned() {
                    app.popup = Popup::Confirm {
                        message: format!("Delete knowledge '{}'?", topic),
                        on_yes: ConfirmAction::DeleteKnowledge(topic),
                    };
                }
            }
            KeyCode::Char('E') => {
                if let Some(topic) = app.knowledge_topics.get(app.knowledge_idx).cloned() {
                    let default = crate::config::data_dir()
                        .join(format!("{}.md", Knowledge::filename(&topic)))
                        .to_string_lossy().to_string();
                    app.popup = Popup::ExportKnowledge {
                        path_buf: default, mode: ExportMode::Single(topic),
                    };
                }
            }
            KeyCode::Char('r') => { app.reload_knowledge(); app.set_status("Knowledge refreshed", false); }
            _ => {}
        }
        Focus::KnowledgeDetail => match code {
            KeyCode::Char('j') | KeyCode::Down => app.knowledge_detail_down(),
            KeyCode::Char('k') | KeyCode::Up   => app.knowledge_detail_up(),
            KeyCode::Char('g')                  => app.knowledge_scroll = 0,
            _ => {}
        }
        _ => {}
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_text(mods: KeyModifiers) -> bool {
    mods == KeyModifiers::NONE || mods == KeyModifiers::SHIFT
}

fn expand_tilde(s: &str) -> String {
    if s.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &s[1..]);
        }
    }
    s.to_string()
}
