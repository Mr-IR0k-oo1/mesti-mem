#![allow(dead_code, unreachable_patterns)]
mod config;
mod context;
mod data;
mod error;
mod executor;
mod platform;
mod ui;
mod watcher;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use ui::app::App;

fn main() {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("matis-mem v{} ({})", env!("CARGO_PKG_VERSION"), platform::os_name());
                return;
            }
            "--help" | "-h" => {
                config::init();
                print_help();
                return;
            }
            _ => {}
        }
    }

    if !platform::is_tty() {
        eprintln!("matis-mem: requires an interactive terminal");
        std::process::exit(1);
    }

    if let Err(e) = run() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("matis-mem: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    config::init();
    config::ensure_dirs()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal.show_cursor();
    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new()?;
    let tick = Duration::from_millis(100);
    let mut last = Instant::now();

    loop {
        terminal.draw(|f| ui::render::render(f, &app))?;

        let timeout = tick.saturating_sub(last.elapsed());
        if crossterm::event::poll(timeout)? {
            let ev = crossterm::event::read()?;
            ui::events::handle(&ev, &mut app);
        }

        if last.elapsed() >= tick {
            app.tick();
            last = Instant::now();
        }

        if app.should_quit { break; }
    }
    Ok(())
}

fn print_help() {
    println!("matis-mem v{} — {}", env!("CARGO_PKG_VERSION"), platform::os_name());
    println!();
    println!("USAGE");
    println!("  matis-mem              Launch TUI");
    println!("  matis-mem --version    Version + OS");
    println!("  matis-mem --help       This help");
    println!();
    println!("DATA:   {}", platform::data_dir_display());
    println!("SHIMS:  {}", config::shims_dir().display());
    println!();
    println!("TABS");
    println!("  [1] RUN        Run prompts with automatic context injection");
    println!("  [2] AGENTS     Live feed of external agent sessions");
    println!("  [3] SHIMS      Install logging wrappers for agent CLIs");
    println!("  [4] KNOWLEDGE  Browse, add, import, export knowledge base");
    println!();
    println!("GLOBAL KEYS");
    println!("  1-4 / Tab     Switch tabs");
    println!("  Ctrl+R / F5   Run prompt");
    println!("  Ctrl+N        New project");
    println!("  Ctrl+K        Add knowledge");
    println!("  Ctrl+M        Refresh model list");
    println!("  Ctrl+I        Import knowledge file/dir");
    println!("  Ctrl+E        Export all knowledge as bundle");
    println!("  q / Ctrl+C    Quit");
    println!();
    println!("MODELS (auto-detected at startup)");
    println!("  ollama/*      requires: ollama + pulled models");
    println!("  gemini-cli    requires: npm i -g @google/gemini-cli && gemini auth");
    println!("  claude        requires: npm i -g @anthropic-ai/claude-code");
    println!("  amp           requires: ampcode.com");
    println!("  vibe          requires: vibe or cursor CLI");
    println!();
    for line in platform::install_instructions() {
        println!("  {}", line);
    }
}
