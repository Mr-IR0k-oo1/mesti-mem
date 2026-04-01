#![allow(dead_code)]
mod config;
mod context;
mod data;
mod error;
mod executor;
mod ui;

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
    // Handle --help and --version before touching the terminal
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => { print_help(); return; }
            "--version" | "-V" => { println!("matis-mem v{}", env!("CARGO_PKG_VERSION")); return; }
            _ => {}
        }
    }

    if !is_tty() {
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
    let tick = Duration::from_millis(100); // faster tick for spinner
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render::render(f, &app))?;

        let timeout = tick.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            let event = crossterm::event::read()?;
            ui::events::handle(&event, &mut app);
        }

        if last_tick.elapsed() >= tick {
            app.poll_exec();
            last_tick = Instant::now();
        }

        if app.should_quit { break; }
    }

    Ok(())
}

fn is_tty() -> bool {
    use std::os::unix::io::AsRawFd;
    unsafe { libc::isatty(io::stdout().as_raw_fd()) != 0 }
}

fn print_help() {
    println!("matis-mem v{} — Terminal AI operating layer", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE");
    println!("  matis-mem              Launch TUI");
    println!("  matis-mem --version    Version");
    println!("  matis-mem --help       This help");
    println!();
    println!("DATA");
    println!("  ~/.matis-mem/projects/    project JSON files");
    println!("  ~/.matis-mem/sessions/    session logs (auto-saved)");
    println!("  ~/.matis-mem/knowledge/   knowledge base");
    println!();
    println!("KEYBINDINGS");
    println!("  Tab / Shift+Tab     Cycle focus panels");
    println!("  Ctrl+R / F5         Run prompt");
    println!("  Ctrl+N              New project");
    println!("  Ctrl+K              Add knowledge");
    println!("  Enter               Run (in prompt panel)");
    println!("  Shift+Enter         Newline in prompt");
    println!("  q / Ctrl+C          Quit");
    println!();
    println!("MODELS");
    println!("  ollama/llama3       requires: ollama pull llama3");
    println!("  gemini-cli          requires: npm install -g @google/gemini-cli");
}
