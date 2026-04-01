use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, Focus, Popup, RunState};
use super::theme::*;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();

    // Dark background fill
    f.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(BG)),
        area,
    );

    // ── Main layout: header(1) | body(fill) | footer(1) ──────────────────────
    let root = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).split(area);

    render_header(f, app, root[0]);
    render_body(f, app, root[1]);
    render_footer(f, app, root[2]);

    // ── Popup on top ─────────────────────────────────────────────────────────
    match &app.popup {
        Popup::None => {}
        Popup::NewProject { .. } => render_new_project(f, app, area),
        Popup::AddKnowledge { .. } => render_add_knowledge(f, app, area),
        Popup::Confirm { .. } => render_confirm(f, app, area),
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let proj_name = app.active_project.as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("—");
    let model_name = app.selected_model().display_name();

    let run_icon = match app.run_state {
        RunState::Idle    => Span::styled("◎ ready", Style::default().fg(DIM)),
        RunState::Running => Span::styled("◌ running…", Style::default().fg(YELLOW)),
        RunState::Done    => Span::styled("◉ done", ok()),
        RunState::Error(_)=> Span::styled("✗ error", err()),
    };

    let spans = vec![
        Span::styled(" ◆ matis-mem  ", accent()),
        Span::styled("project:", dim()),
        Span::styled(format!(" {}  ", proj_name), normal()),
        Span::styled("model:", dim()),
        Span::styled(format!(" {}  ", model_name), normal()),
        run_icon,
    ];

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)),
        area,
    );
}

// ── Body: projects | main ─────────────────────────────────────────────────────

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Length(22),   // project sidebar
        Constraint::Fill(1),      // main area
    ]).split(area);

    render_projects(f, app, cols[0]);
    render_main(f, app, cols[1]);
}

fn render_projects(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Projects;
    let items: Vec<ListItem> = app.projects.iter().enumerate().map(|(i, name)| {
        let style = if i == app.project_idx {
            selected()
        } else {
            normal()
        };
        ListItem::new(format!(" {}", name)).style(style)
    }).collect();

    let display = if items.is_empty() {
        vec![ListItem::new(" (none)  [n] new").style(dim())]
    } else {
        items
    };

    let list = List::new(display)
        .block(
            Block::bordered()
                .title(Span::styled(" PROJECTS ", dim()))
                .border_style(border(focused))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(SURFACE)),
        )
        .highlight_style(selected());

    // We pass a throwaway ListState since we manage idx ourselves
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.project_idx));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    // Stack: prompt | context+model row | response
    let rows = Layout::vertical([
        Constraint::Length(5),     // prompt
        Constraint::Length(8),     // context + model side by side
        Constraint::Fill(1),       // response
    ]).split(area);

    render_prompt(f, app, rows[0]);
    render_controls(f, app, rows[1]);
    render_response(f, app, rows[2]);
}

// ── Prompt ────────────────────────────────────────────────────────────────────

fn render_prompt(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Prompt;

    // Show cursor by inserting block char at cursor position
    let before = &app.prompt[..app.cursor];
    let after  = &app.prompt[app.cursor..];
    let display = format!("{}▌{}", before, after);

    let placeholder = if app.prompt.is_empty() && !focused {
        "enter a prompt…"
    } else {
        ""
    };

    let text = if app.prompt.is_empty() && !focused {
        placeholder.to_string()
    } else {
        display
    };

    let style = if app.prompt.is_empty() && !focused {
        dim()
    } else {
        normal()
    };

    f.render_widget(
        Paragraph::new(text.as_str())
            .style(style)
            .block(
                Block::bordered()
                    .title(Span::styled(" PROMPT ", dim()))
                    .border_style(border(focused))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(SURFACE)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

// ── Context + Model controls ──────────────────────────────────────────────────

fn render_controls(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ]).split(area);

    render_context_panel(f, app, cols[0]);
    render_model_panel(f, app, cols[1]);
}

fn checkbox(on: bool) -> Span<'static> {
    if on {
        Span::styled("[x] ", ok())
    } else {
        Span::styled("[ ] ", dim())
    }
}

fn render_context_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Context;

    let session_label = match app.ctx_recent_n {
        0 => "[ ] recent sessions".to_string(),
        n => format!("[x] last {} session{}", n, if n == 1 { "" } else { "s" }),
    };
    let session_style = if app.ctx_recent_n > 0 { ok() } else { dim() };

    let lines = vec![
        Line::from(vec![checkbox(app.ctx_include_project),
            Span::styled("project context  ", normal()),
            Span::styled("[space]", dim())]),
        Line::from(vec![Span::styled(&session_label, session_style),
            Span::raw("  "),
            Span::styled("[-/+]", dim())]),
        Line::from(vec![checkbox(app.ctx_knowledge_query),
            Span::styled("knowledge search  ", normal()),
            Span::styled("[k]", dim())]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Ctrl+R / F5 = RUN", accent()),
        ]),
    ];

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title(Span::styled(" CONTEXT ", dim()))
                    .border_style(border(focused))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(SURFACE)),
            ),
        area,
    );
}

fn render_model_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Model;

    let items: Vec<ListItem> = app.models.iter().enumerate().map(|(i, m)| {
        let style = if i == app.model_idx { selected() } else { normal() };
        ListItem::new(format!(" {}", m.display_name())).style(style)
    }).collect();

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.model_idx));

    f.render_stateful_widget(
        List::new(items)
            .block(
                Block::bordered()
                    .title(Span::styled(" MODEL ", dim()))
                    .border_style(border(focused))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(SURFACE)),
            )
            .highlight_style(selected()),
        area,
        &mut state,
    );
}

// ── Response ──────────────────────────────────────────────────────────────────

fn render_response(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Response;

    let (title, content, style): (&str, String, Style) = match &app.run_state {
        RunState::Idle    => (" RESPONSE ", "".into(), dim()),
        RunState::Running => (" RESPONSE  running… ", "".into(), dim()),
        RunState::Done    => (" RESPONSE ", app.response.clone(), normal()),
        RunState::Error(e)=> (" RESPONSE  error ", format!("Error: {}", e), err()),
    };

    // Spinner animation for running state
    let title_str = if app.run_state == RunState::Running {
        let spinner = ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_millis() / 100) as usize % spinner.len();
        format!(" {} running ", spinner[idx])
    } else {
        title.to_string()
    };

    f.render_widget(
        Paragraph::new(content.as_str())
            .style(style)
            .block(
                Block::bordered()
                    .title(Span::styled(title_str, dim()))
                    .border_style(border(focused))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(SURFACE)),
            )
            .wrap(Wrap { trim: false })
            .scroll((app.response_scroll as u16, 0)),
        area,
    );
}

// ── Footer ────────────────────────────────────────────────────────────────────

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some((ref msg, is_err)) = app.status {
        let style = if is_err { err() } else { ok() };  // borrow fix
        Paragraph::new(format!(" {}", msg)).style(style)
    } else {
        let hints = match app.focus {
            Focus::Projects => " [j/k] nav  [n] new  [d] del  [Tab] focus prompt",
            Focus::Prompt   => " type prompt  [Ctrl+R / F5] run  [Tab] → context",
            Focus::Context  => " [Space] project  [-/+] sessions  [k] knowledge  [Tab] → model",
            Focus::Model    => " [j/k] select  [Tab] → response  [Ctrl+R] run",
            Focus::Response => " [j/k] scroll  [Tab] → prompt  [y] copy  [Ctrl+R] run again",
        };
        Paragraph::new(hints).style(dim())
    };

    f.render_widget(text.style(Style::default().bg(BG)), area);
}

// ── New Project Popup ─────────────────────────────────────────────────────────

fn render_new_project(f: &mut Frame, app: &App, area: Rect) {
    let Popup::NewProject { name_buf, goal_buf, field } = &app.popup else { return };

    let popup_area = centered(58, 10, area);
    f.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .title(Span::styled(" New Project ", accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Name: ", if *field == 0 { accent() } else { dim() }),
            Span::styled(format!("{}▌", name_buf), normal()),
        ])),
        rows[1],
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Goal: ", if *field == 1 { accent() } else { dim() }),
            Span::styled(format!("{}▌", goal_buf), normal()),
        ])),
        rows[3],
    );

    f.render_widget(
        Paragraph::new("[Tab] next field  [Enter] create  [Esc] cancel")
            .style(dim()).alignment(Alignment::Center),
        rows[5],
    );
}

// ── Add Knowledge Popup ───────────────────────────────────────────────────────

fn render_add_knowledge(f: &mut Frame, app: &App, area: Rect) {
    let Popup::AddKnowledge { topic_buf, note_buf } = &app.popup else { return };

    let popup_area = centered(60, 10, area);
    f.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .title(Span::styled(" Add Knowledge ", accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(inner);

    f.render_widget(
        Paragraph::new(format!("Topic:  {}▌", topic_buf)).style(normal()),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(format!("Note:   {}▌", note_buf)).style(normal()),
        rows[3],
    );
    f.render_widget(
        Paragraph::new("[Enter] save  [Esc] cancel").style(dim()).alignment(Alignment::Center),
        rows[5],
    );
}

// ── Confirm Popup ─────────────────────────────────────────────────────────────

fn render_confirm(f: &mut Frame, app: &App, area: Rect) {
    let Popup::Confirm { message, .. } = &app.popup else { return };

    let popup_area = centered(50, 7, area);
    f.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .title(Span::styled(" Confirm ", accent()))
        .border_style(Style::default().fg(FOCUS))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(SURFACE));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let rows = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).split(inner);

    f.render_widget(
        Paragraph::new(message.as_str()).style(normal()).alignment(Alignment::Center),
        rows[1],
    );
    f.render_widget(
        Paragraph::new("[y / Enter] yes  [n / Esc] cancel").style(dim()).alignment(Alignment::Center),
        rows[3],
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn centered(pct_x: u16, height: u16, r: Rect) -> Rect {
    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ]).split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ]).split(vert[1])[1]
}
