use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, ExportMode, Focus, Popup, RunState, Tab};
use super::theme::*;

// ── Responsive breakpoints ────────────────────────────────────────────────────

struct Layout_ {
    narrow:     bool,  // < 80 cols
    very_small: bool,  // < 60 cols
    short:      bool,  // < 20 rows
}

fn measure(area: Rect) -> Layout_ {
    Layout_ {
        narrow:     area.width < 80,
        very_small: area.width < 60,
        short:      area.height < 20,
    }
}

// ── Root render ───────────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();
    if area.width < 20 || area.height < 6 {
        f.render_widget(
            Paragraph::new("terminal too small")
                .style(Style::default().fg(RED))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let lm = measure(area);
    let header_h = if lm.short { 1u16 } else { 2 };

    let root = Layout::vertical([
        Constraint::Length(header_h),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).split(area);

    render_header(f, app, root[0], &lm);

    match app.tab {
        Tab::Run       => render_run(f, app, root[1], &lm),
        Tab::Agents    => render_agents(f, app, root[1], &lm),
        Tab::Shims     => render_shims(f, app, root[1]),
        Tab::Knowledge => render_knowledge(f, app, root[1], &lm),
    }

    render_footer(f, app, root[2]);

    match &app.popup {
        Popup::None => {}
        Popup::NewProject   { .. }       => render_new_project(f, app, area),
        Popup::AddKnowledge { .. }       => render_add_knowledge(f, app, area),
        Popup::EditKnowledge { .. }      => render_edit_knowledge(f, app, area),
        Popup::ImportKnowledge { .. }    => render_import_knowledge(f, app, area),
        Popup::ExportKnowledge { .. }    => render_export_knowledge(f, app, area),
        Popup::Confirm      { .. }       => render_confirm(f, app, area),
        Popup::Output       { .. }       => render_output(f, app, area),
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &App, area: Rect, lm: &Layout_) {
    if lm.short {
        // Compact single-line header
        let tab_char = match app.tab { Tab::Run=>"1", Tab::Agents=>"2", Tab::Shims=>"3", Tab::Knowledge=>"4" };
        let mdl = app.models.get(app.model_idx).map(|m| m.display_name()).unwrap_or_default();
        let proj = app.active_project.as_ref().map(|p| p.name.as_str()).unwrap_or("-");
        f.render_widget(
            Paragraph::new(format!(" ◆ [{tab_char}] {proj} · {mdl}"))
                .style(Style::default().fg(ACCENT).bg(BG)),
            area,
        );
        return;
    }

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);

    let proj = app.active_project.as_ref().map(|p| p.name.as_str()).unwrap_or("—");
    let mdl  = app.models.get(app.model_idx).map(|m| m.display_name()).unwrap_or_default();

    let run_span = match &app.run_state {
        RunState::Idle    => Span::styled("◎ idle",     Style::default().fg(DIM)),
        RunState::Running => Span::styled("◌ running…", Style::default().fg(YELLOW)),
        RunState::Done    => Span::styled("◉ done",     ok()),
        RunState::Error(_)=> Span::styled("✗ error",    err()),
    };

    // Truncate model name if narrow
    let mdl_display = if lm.narrow && mdl.len() > 14 {
        format!("{}…", &mdl[..14])
    } else { mdl };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ◆ matis-mem  ", accent()),
            Span::styled("proj:", dim()), Span::raw(format!(" {}  ", proj)),
            if !lm.narrow { Span::styled("model:", dim()) } else { Span::raw("") },
            if !lm.narrow { Span::raw(format!(" {}  ", mdl_display)) } else { Span::raw("") },
            run_span,
        ])).style(Style::default().bg(BG)),
        rows[0],
    );

    // Tab bar — collapse labels when narrow
    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    let tabs = [
        (Tab::Run,       "[1] RUN",        "1"),
        (Tab::Agents,    "[2] AGENTS",     "2"),
        (Tab::Shims,     "[3] SHIMS",      "3"),
        (Tab::Knowledge, "[4] KNOWLEDGE",  "4"),
    ];
    for (t, long, short) in &tabs {
        let label = if lm.narrow { short.to_string() } else {
            if *t == Tab::Agents && app.unread_count > 0 {
                format!("{} ({})", long, app.unread_count)
            } else { long.to_string() }
        };
        if *t == app.tab {
            spans.push(Span::styled(label, Style::default().fg(ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)));
        } else {
            spans.push(Span::styled(label, dim()));
        }
        spans.push(Span::raw("  "));
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)), rows[1]);
}

// ══════════════════════════════════════════════════════════════════════════════
// TAB 1 — RUN
// ══════════════════════════════════════════════════════════════════════════════

fn render_run(f: &mut Frame, app: &App, area: Rect, lm: &Layout_) {
    if lm.very_small {
        // Vertical stack only — no sidebar
        render_run_main(f, app, area, lm);
        return;
    }

    // Responsive sidebar width: clamp between 14 and 22 cols
    let sidebar_w = (area.width / 5).max(14).min(22);
    let cols = Layout::horizontal([
        Constraint::Length(sidebar_w),
        Constraint::Fill(1),
    ]).split(area);

    render_projects(f, app, cols[0]);
    render_run_main(f, app, cols[1], lm);
}

fn render_projects(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Projects;
    let items: Vec<ListItem> = app.projects.iter().enumerate().map(|(i, n)| {
        // Truncate to panel width - 3
        let max = (area.width as usize).saturating_sub(3);
        let display: String = n.chars().take(max).collect();
        ListItem::new(format!(" {}", display))
            .style(if i == app.project_idx { selected() } else { normal() })
    }).collect();

    let display = if items.is_empty() {
        vec![ListItem::new(" [n] new").style(dim())]
    } else { items };

    let mut st = ratatui::widgets::ListState::default();
    st.select(Some(app.project_idx));
    f.render_stateful_widget(
        List::new(display)
            .block(Block::bordered()
                .title(Span::styled(" PROJ ", dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .highlight_style(selected()),
        area, &mut st,
    );
}

fn render_run_main(f: &mut Frame, app: &App, area: Rect, lm: &Layout_) {
    if lm.short {
        // Very short: prompt only + response
        let rows = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
        ]).split(area);
        render_prompt(f, app, rows[0]);
        render_response(f, app, rows[1]);
        return;
    }

    // Context/model panel height — collapse when really narrow
    let ctrl_h = if lm.narrow { 5u16 } else { 7 };
    let prompt_h = if area.height > 25 { 5u16 } else { 4 };

    let rows = Layout::vertical([
        Constraint::Length(prompt_h),
        Constraint::Length(ctrl_h),
        Constraint::Fill(1),
    ]).split(area);

    render_prompt(f, app, rows[0]);
    render_controls(f, app, rows[1], lm);
    render_response(f, app, rows[2]);
}

fn render_prompt(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Prompt;
    let text = if app.prompt.is_empty() && !focused {
        "enter a prompt…".to_string()
    } else {
        // Safe cursor display — clamp to avoid panic on multi-byte boundaries
        let cursor = app.cursor.min(app.prompt.len());
        format!("{}▌{}", &app.prompt[..cursor], &app.prompt[cursor..])
    };
    let style = if app.prompt.is_empty() && !focused { dim() } else { normal() };
    f.render_widget(
        Paragraph::new(text.as_str()).style(style)
            .block(Block::bordered()
                .title(Span::styled(" PROMPT ", dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_controls(f: &mut Frame, app: &App, area: Rect, lm: &Layout_) {
    if lm.narrow {
        // Stacked vertically when narrow
        let rows = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]).split(area);
        render_context_panel(f, app, rows[0]);
        render_model_panel(f, app, rows[1]);
    } else {
        let cols = Layout::horizontal([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ]).split(area);
        render_context_panel(f, app, cols[0]);
        render_model_panel(f, app, cols[1]);
    }
}

fn cb(on: bool) -> Span<'static> {
    if on { Span::styled("[x] ", ok()) } else { Span::styled("[ ] ", dim()) }
}

fn render_context_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Context;
    let sess = match app.ctx_sessions {
        0 => "[ ] sessions".to_string(),
        n => format!("[x] last {}", n),
    };
    let lines = vec![
        Line::from(vec![cb(app.ctx_project),   Span::raw("project  "), Span::styled("[spc]", dim())]),
        Line::from(vec![Span::styled(&sess, if app.ctx_sessions>0{ok()}else{dim()}), Span::raw("  "), Span::styled("[-/+]", dim())]),
        Line::from(vec![cb(app.ctx_knowledge), Span::raw("knowledge"), Span::styled("  [k]", dim())]),
        Line::from(vec![Span::styled("  Ctrl+R / F5 = RUN", accent())]),
    ];
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered()
                .title(Span::styled(" CONTEXT ", dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE))),
        area,
    );
}

fn render_model_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Model;
    let max_display = area.width.saturating_sub(4) as usize;

    if app.model_scanning {
        f.render_widget(
            Paragraph::new("  scanning…").style(dim())
                .block(Block::bordered()
                    .title(Span::styled(" MODEL ", dim()))
                    .border_style(border(focused))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(SURFACE))),
            area,
        );
        return;
    }

    let mut last_cat = "";
    let mut items: Vec<ListItem> = Vec::new();
    let mut display_idx = 0usize;
    let mut selected_display = 0usize;

    for (i, m) in app.models.iter().enumerate() {
        let cat = m.category();
        if cat != last_cat {
            items.push(ListItem::new(format!(" {} ", cat)).style(dim()));
            display_idx += 1;
            last_cat = cat;
        }
        if i == app.model_idx { selected_display = display_idx; }
        let name: String = m.display_name().chars().take(max_display).collect();
        items.push(
            ListItem::new(format!("  {}", name))
                .style(if i == app.model_idx { selected() } else { normal() })
        );
        display_idx += 1;
    }

    let mut st = ratatui::widgets::ListState::default();
    st.select(Some(selected_display));

    let title = format!(" MODEL  [r]refresh ({} found) ", app.models.len());
    f.render_stateful_widget(
        List::new(items)
            .block(Block::bordered()
                .title(Span::styled(title, dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .highlight_style(selected()),
        area, &mut st,
    );
}

fn render_response(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Response;
    let (title, content, style) = match &app.run_state {
        RunState::Idle     => (" RESPONSE ", String::new(), dim()),
        RunState::Running  => (" RESPONSE  ◌ ", String::new(), dim()),
        RunState::Done     => (" RESPONSE ", app.response.clone(), normal()),
        RunState::Error(e) => (" RESPONSE  ✗ ", format!("Error: {}", e), err()),
    };
    f.render_widget(
        Paragraph::new(content.as_str()).style(style)
            .block(Block::bordered()
                .title(Span::styled(title, dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .wrap(Wrap { trim: false })
            .scroll((app.response_scroll as u16, 0)),
        area,
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// TAB 2 — AGENTS
// ══════════════════════════════════════════════════════════════════════════════

fn render_agents(f: &mut Frame, app: &App, area: Rect, lm: &Layout_) {
    if lm.very_small {
        render_agent_list(f, app, area);
        return;
    }
    let split = if lm.narrow { 50u16 } else { 38 };
    let cols = Layout::horizontal([
        Constraint::Percentage(split),
        Constraint::Percentage(100 - split),
    ]).split(area);
    render_agent_list(f, app, cols[0]);
    render_agent_detail(f, app, cols[1]);
}

fn agent_color(name: &str) -> Style {
    match name {
        "claude"  => Style::default().fg(ratatui::style::Color::Rgb(100,160,220)),
        "amp"     => Style::default().fg(ratatui::style::Color::Rgb(120,190,120)),
        "gemini"  => Style::default().fg(ratatui::style::Color::Rgb(220,190,70)),
        "vibe"    => Style::default().fg(ratatui::style::Color::Rgb(180,100,220)),
        "ollama"  => Style::default().fg(ratatui::style::Color::Rgb(200,140,80)),
        "mistral" => Style::default().fg(ratatui::style::Color::Rgb(220,80,80)),
        _         => Style::default().fg(ratatui::style::Color::Rgb(180,180,180)),
    }
}

fn render_agent_list(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::AgentList;
    let logs    = app.filtered_logs();
    let max_w   = (area.width as usize).saturating_sub(6);

    let items: Vec<ListItem> = logs.iter().map(|log| {
        let preview: String = if log.input.is_empty() { &log.args } else { &log.input }
            .chars().take(max_w.min(38)).collect();
        let proj_tag = if log.project.is_empty() { String::new() }
                       else { format!("[{}]", log.project) };
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!(" {:<10}", log.agent), agent_color(&log.agent).add_modifier(Modifier::BOLD)),
                Span::styled(proj_tag, dim()),
            ]),
            Line::from(Span::styled(format!("  {}", preview), dim())),
        ])
    }).collect();

    let display = if items.is_empty() {
        vec![ListItem::new("  no logs yet → [3] SHIMS").style(dim())]
    } else { items };

    let filter_str = app.agent_filter.as_deref().map(|f| format!(" :{} ", f)).unwrap_or_default();
    let max_sel = logs.len().saturating_sub(1);
    let mut st  = ratatui::widgets::ListState::default();
    st.select(Some(app.agent_idx.min(max_sel)));

    f.render_stateful_widget(
        List::new(display)
            .block(Block::bordered()
                .title(Span::styled(format!(" AGENTS{} ", filter_str), dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .highlight_style(selected()),
        area, &mut st,
    );
}

fn render_agent_detail(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::AgentDetail;
    let (title, content) = match app.selected_log() {
        None => (" detail ".to_string(), "select a log entry".to_string()),
        Some(log) => {
            let mut v = Vec::new();
            v.push(format!("Agent:   {}", log.agent));
            v.push(format!("Project: {}", if log.project.is_empty() { "—" } else { &log.project }));
            v.push(format!("CWD:     {}", log.cwd));
            v.push(format!("Time:    {}", &log.timestamp[..19].replace('T'," ")));
            v.push(format!("{}ms  exit:{}", log.duration_ms, log.exit_code));
            if !log.input.is_empty() {
                v.push(String::new());
                v.push("── input ──".into());
                for l in log.input.lines().take(30) { v.push(l.to_string()); }
            }
            if !log.output.is_empty() && log.output != "(interactive)" {
                v.push(String::new());
                v.push("── output ──".into());
                for l in log.output.lines().take(40) { v.push(l.to_string()); }
            }
            (format!(" {} ", log.agent), v.join("\n"))
        }
    };
    f.render_widget(
        Paragraph::new(content.as_str()).style(normal())
            .block(Block::bordered()
                .title(Span::styled(title, dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .wrap(Wrap { trim: false }),
        area,
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// TAB 3 — SHIMS
// ══════════════════════════════════════════════════════════════════════════════

fn render_shims(f: &mut Frame, app: &App, area: Rect) {
    let warn_h = if app.shims_need_path { 4u16 } else { 0 };
    let help_h = area.height.saturating_sub(warn_h + 2).min(6).max(3);
    let rows = Layout::vertical([
        Constraint::Length(warn_h),
        Constraint::Fill(1),
        Constraint::Length(help_h),
    ]).split(area);

    if app.shims_need_path {
        let line = crate::platform::path_export_line(&crate::config::shims_dir());
        let apply = crate::platform::path_apply_instruction();
        f.render_widget(
            Paragraph::new(format!("⚠  Shim dir not in PATH.\n{}\n  {}", apply, line))
                .style(Style::default().fg(YELLOW))
                .block(Block::bordered()
                    .border_style(Style::default().fg(YELLOW))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(SURFACE)))
                .wrap(Wrap { trim: false }),
            rows[0],
        );
    }

    let focused = app.focus == Focus::ShimList;
    let items: Vec<ListItem> = app.shim_statuses.iter().enumerate().map(|(i, s)| {
        let (icon, icon_style) = if !s.real_exists {
            ("○ not found   ", dim())
        } else if s.installed && s.active_in_path {
            ("● active       ", ok())
        } else if s.installed {
            ("◑ installed    ", Style::default().fg(YELLOW))
        } else {
            ("◌ not shimmed  ", dim())
        };
        let name_style = if i == app.shim_idx {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else { normal() };
        ListItem::new(Line::from(vec![
            Span::styled(format!("  {}", icon), icon_style),
            Span::styled(format!("{:<12}", s.name), name_style),
        ]))
    }).collect();

    let mut st = ratatui::widgets::ListState::default();
    st.select(Some(app.shim_idx));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::bordered()
                .title(Span::styled(" SHIMS  ● active  ◑ installed  ◌ not shimmed  ○ not found ", dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .highlight_style(selected()),
        rows[1], &mut st,
    );

    f.render_widget(
        Paragraph::new(" [i] install all  [u] uninstall  [r] refresh\n\
                        Shims intercept CLI calls from any terminal tab and log them live.")
            .style(dim())
            .block(Block::bordered()
                .border_style(Style::default().fg(BORDER))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .wrap(Wrap { trim: false }),
        rows[2],
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// TAB 4 — KNOWLEDGE
// ══════════════════════════════════════════════════════════════════════════════

fn render_knowledge(f: &mut Frame, app: &App, area: Rect, lm: &Layout_) {
    if lm.very_small {
        render_knowledge_list(f, app, area);
        return;
    }

    let split = if lm.narrow { 40u16 } else { 32 };
    let cols  = Layout::horizontal([
        Constraint::Percentage(split),
        Constraint::Percentage(100 - split),
    ]).split(area);

    render_knowledge_list(f, app, cols[0]);
    render_knowledge_detail(f, app, cols[1]);
}

fn render_knowledge_list(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::KnowledgeList;
    let max_w   = (area.width as usize).saturating_sub(4);

    let items: Vec<ListItem> = app.knowledge_topics.iter().enumerate().map(|(i, t)| {
        let display: String = t.chars().take(max_w).collect();
        ListItem::new(format!("  {}", display))
            .style(if i == app.knowledge_idx { selected() } else { normal() })
    }).collect();

    let display = if items.is_empty() {
        vec![
            ListItem::new("  (empty)").style(dim()),
            ListItem::new("  [n] new  Ctrl+K add").style(dim()),
        ]
    } else { items };

    let mut st = ratatui::widgets::ListState::default();
    st.select(Some(app.knowledge_idx));

    let count = app.knowledge_topics.len();
    let title = format!(" KNOWLEDGE ({}) ", count);

    f.render_stateful_widget(
        List::new(display)
            .block(Block::bordered()
                .title(Span::styled(title, dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .highlight_style(selected()),
        area, &mut st,
    );
}

fn render_knowledge_detail(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::KnowledgeDetail;

    // Render help hint row at bottom
    let help_h = 1u16;
    let rows = Layout::vertical([Constraint::Fill(1), Constraint::Length(help_h)]).split(area);

    let (title, content) = if app.knowledge_detail.is_empty() {
        (" detail ".to_string(), "Select a topic from the left.".to_string())
    } else {
        let topic = app.knowledge_topics.get(app.knowledge_idx)
            .map(|t| t.as_str()).unwrap_or("detail");
        (format!(" {} ", topic), app.knowledge_detail.clone())
    };

    f.render_widget(
        Paragraph::new(content.as_str()).style(normal())
            .block(Block::bordered()
                .title(Span::styled(title, dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)))
            .wrap(Wrap { trim: false })
            .scroll((app.knowledge_scroll as u16, 0)),
        rows[0],
    );

    f.render_widget(
        Paragraph::new(" [e] edit  [x] delete  [E] export one  Ctrl+I import  Ctrl+E export all")
            .style(dim()),
        rows[1],
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// FOOTER
// ══════════════════════════════════════════════════════════════════════════════

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some((ref msg, is_err, _)) = app.status {
        Paragraph::new(format!(" {}", msg))
            .style(if is_err { err() } else { ok() })
    } else {
        let hint = match (&app.tab, &app.focus) {
            (Tab::Run, Focus::Prompt)    => " [Enter/Ctrl+R] run  [Shift+Enter] newline  [Tab] next",
            (Tab::Run, Focus::Model)     => " [j/k] model  [r] refresh models  [Ctrl+R] run",
            (Tab::Run, Focus::Context)   => " [Spc] project  [-/+] sessions  [k] knowledge",
            (Tab::Run, Focus::Response)  => " [j/k] scroll  [g/G] top/bot  [c] clear",
            (Tab::Run, _)                => " [j/k] nav  [n] new project  [d] delete  [Enter] select",
            (Tab::Agents, _)             => " [j/k] nav  [f] filter  [a] all  [r] refresh",
            (Tab::Shims,  _)             => " [i] install  [u] uninstall  [r] refresh",
            (Tab::Knowledge, Focus::KnowledgeList)   => " [j/k] n  [n] new  [e] edit  [x] del  Ctrl+I import  Ctrl+E export",
            (Tab::Knowledge, Focus::KnowledgeDetail) => " [j/k] scroll  [g] top  Tab→list",
            _ => " [1-4] tabs  [Ctrl+N] project  [Ctrl+K] knowledge  [q] quit",
        };
        Paragraph::new(hint).style(dim())
    };
    f.render_widget(text.style(Style::default().bg(BG)), area);
}

// ══════════════════════════════════════════════════════════════════════════════
// POPUPS
// ══════════════════════════════════════════════════════════════════════════════

fn render_new_project(f: &mut Frame, app: &App, area: Rect) {
    let Popup::NewProject { name_buf, goal_buf, field } = &app.popup else { return };
    let p = centered_rect(58, 10, area);
    f.render_widget(Clear, p);
    let block = Block::bordered()
        .title(Span::styled(" New Project ", accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(p);
    f.render_widget(block, p);
    let r = Layout::vertical([
        Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
        Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
    ]).split(inner);
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled("Name: ", if *field==0{accent()}else{dim()}),
        Span::styled(format!("{}▌", name_buf), normal()),
    ])), r[1]);
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled("Goal: ", if *field==1{accent()}else{dim()}),
        Span::styled(format!("{}▌", goal_buf), normal()),
    ])), r[3]);
    f.render_widget(
        Paragraph::new("[Tab] next  [Enter] create  [Esc] cancel")
            .style(dim()).alignment(Alignment::Center), r[5],
    );
}

fn render_add_knowledge(f: &mut Frame, app: &App, area: Rect) {
    let Popup::AddKnowledge { topic_buf, note_buf, tag_buf, active_field, error } = &app.popup else { return };
    render_knowledge_form(f, "Add Knowledge", topic_buf, note_buf, tag_buf, *active_field, error.as_deref(), area);
}

fn render_edit_knowledge(f: &mut Frame, app: &App, area: Rect) {
    let Popup::EditKnowledge { topic_buf, note_buf, tag_buf, active_field, error, .. } = &app.popup else { return };
    render_knowledge_form(f, "Edit Knowledge", topic_buf, note_buf, tag_buf, *active_field, error.as_deref(), area);
}

fn render_knowledge_form(
    f: &mut Frame,
    title: &str,
    topic_buf: &str,
    note_buf:  &str,
    tag_buf:   &str,
    active:    usize,
    error:     Option<&str>,
    area:      Rect,
) {
    let height = if error.is_some() { 14u16 } else { 12 };
    let p = centered_rect(64, height, area);
    f.render_widget(Clear, p);
    let block = Block::bordered()
        .title(Span::styled(format!(" {} ", title), accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(p);
    f.render_widget(block, p);

    // Build rows dynamically
    let mut constraints = vec![
        Constraint::Length(1), // Topic label
        Constraint::Length(1), // Topic input
        Constraint::Length(1), // Note label
        Constraint::Length(1), // Note input
        Constraint::Length(1), // Tags label
        Constraint::Length(1), // Tags input
        Constraint::Length(1), // spacer
        Constraint::Length(1), // hint
    ];
    if error.is_some() {
        constraints.push(Constraint::Length(1)); // error row
    }
    let r = Layout::vertical(constraints).split(inner);

    let field_style = |i: usize| if i == active { accent() } else { dim() };

    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled("Topic  ", field_style(0)),
        Span::styled("(min 3 chars)", dim()),
    ])), r[0]);
    f.render_widget(
        Paragraph::new(format!("> {}▌", topic_buf)).style(if active==0{normal()}else{dim()}), r[1],
    );
    f.render_widget(Paragraph::new(Span::styled("Note   ", field_style(1))), r[2]);
    f.render_widget(
        Paragraph::new(format!("> {}▌", note_buf)).style(if active==1{normal()}else{dim()}), r[3],
    );
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled("Tags   ", field_style(2)),
        Span::styled("comma-separated", dim()),
    ])), r[4]);
    f.render_widget(
        Paragraph::new(format!("> {}▌", tag_buf)).style(if active==2{normal()}else{dim()}), r[5],
    );
    f.render_widget(
        Paragraph::new("[Tab] next field  [Enter] save  [Esc] cancel")
            .style(dim()).alignment(Alignment::Center), r[7],
    );
    if let Some(e) = error {
        if r.len() > 8 {
            f.render_widget(
                Paragraph::new(format!("✗ {}", e)).style(err()), r[8],
            );
        }
    }
}

fn render_import_knowledge(f: &mut Frame, app: &App, area: Rect) {
    let Popup::ImportKnowledge { path_buf, error } = &app.popup else { return };
    let p = centered_rect(62, 10, area);
    f.render_widget(Clear, p);
    let block = Block::bordered()
        .title(Span::styled(" Import Knowledge ", accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(p);
    f.render_widget(block, p);
    let r = Layout::vertical([
        Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
        Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
        Constraint::Length(1), Constraint::Length(1),
    ]).split(inner);
    f.render_widget(Paragraph::new("Path to .md file or directory of .md files:").style(dim()), r[1]);
    f.render_widget(Paragraph::new(format!("> {}▌", path_buf)).style(normal()), r[2]);
    f.render_widget(Paragraph::new("Files must have a # Title header line.").style(dim()), r[4]);
    if let Some(e) = error {
        f.render_widget(Paragraph::new(format!("✗ {}", e)).style(err()), r[5]);
    }
    f.render_widget(
        Paragraph::new("[Enter] import  [Esc] cancel")
            .style(dim()).alignment(Alignment::Center), r[7],
    );
}

fn render_export_knowledge(f: &mut Frame, app: &App, area: Rect) {
    let Popup::ExportKnowledge { path_buf, mode } = &app.popup else { return };
    let p = centered_rect(62, 10, area);
    f.render_widget(Clear, p);
    let mode_str = match mode {
        ExportMode::All          => "Export all entries (one .md per entry)",
        ExportMode::Bundle       => "Export all as single bundled .md file",
        ExportMode::Single(t)    => t.as_str(),
    };
    let block = Block::bordered()
        .title(Span::styled(" Export Knowledge ", accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(p);
    f.render_widget(block, p);
    let r = Layout::vertical([
        Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
        Constraint::Length(1), Constraint::Length(1), Constraint::Length(1),
        Constraint::Length(1), Constraint::Length(1),
    ]).split(inner);
    f.render_widget(Paragraph::new(format!("Mode: {}", mode_str)).style(normal()), r[1]);
    f.render_widget(Paragraph::new("Destination path:").style(dim()), r[3]);
    f.render_widget(Paragraph::new(format!("> {}▌", path_buf)).style(normal()), r[4]);
    f.render_widget(
        Paragraph::new("[Enter] export  [Esc] cancel")
            .style(dim()).alignment(Alignment::Center), r[7],
    );
}

fn render_confirm(f: &mut Frame, app: &App, area: Rect) {
    let Popup::Confirm { message, .. } = &app.popup else { return };
    let p = centered_rect(54, 7, area);
    f.render_widget(Clear, p);
    let block = Block::bordered()
        .title(Span::styled(" Confirm ", accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(p);
    f.render_widget(block, p);
    let r = Layout::vertical([
        Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1),
    ]).split(inner);
    f.render_widget(Paragraph::new(message.as_str()).style(normal()).alignment(Alignment::Center), r[1]);
    f.render_widget(
        Paragraph::new("[y / Enter] yes  [n / Esc] cancel")
            .style(dim()).alignment(Alignment::Center), r[3],
    );
}

fn render_output(f: &mut Frame, app: &App, area: Rect) {
    let Popup::Output { title, lines, scroll } = &app.popup else { return };
    // Responsive: use 90% width on narrow terminals
    let pct = if area.width < 80 { 90u16 } else { 70 };
    let p = centered_rect(pct, 60, area);
    f.render_widget(Clear, p);
    let block = Block::bordered()
        .title(Span::styled(format!(" {} ", title), accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));
    let inner = block.inner(p);
    f.render_widget(block, p);
    let h = inner.height.saturating_sub(1) as usize;
    let visible: Vec<ListItem> = lines.iter().skip(*scroll).take(h).map(|l| {
        let s = if l.contains('✓') || l.contains("Done")  { ok() }
                else if l.contains('✗') || l.contains("Error") { err() }
                else if l.contains('⚠') { Style::default().fg(YELLOW) }
                else { normal() };
        ListItem::new(l.as_str()).style(s)
    }).collect();
    f.render_widget(List::new(visible).style(Style::default().bg(SURFACE)), inner);
    let hint_y = inner.y + inner.height.saturating_sub(1);
    f.render_widget(
        Paragraph::new(" [j/k] scroll  any key dismiss").style(dim()),
        Rect { x: inner.x, y: hint_y, width: inner.width, height: 1 },
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn centered_rect(pct_x: u16, height: u16, r: Rect) -> Rect {
    // Clamp height to terminal height
    let h = height.min(r.height.saturating_sub(2));
    let v = Layout::vertical([
        Constraint::Fill(1), Constraint::Length(h), Constraint::Fill(1),
    ]).split(r);
    // Clamp width percentage to at least show something on small terminals
    let pct = pct_x.min(100);
    let margin = (100 - pct) / 2;
    Layout::horizontal([
        Constraint::Percentage(margin),
        Constraint::Percentage(pct),
        Constraint::Percentage(margin),
    ]).split(v[1])[1]
}
