use std::path::{Path, PathBuf};

// ── TTY ───────────────────────────────────────────────────────────────────────
pub fn is_tty() -> bool {
    use crossterm::tty::IsTty;
    std::io::stdout().is_tty()
}

// ── OS ────────────────────────────────────────────────────────────────────────
pub fn os_name() -> &'static str {
    if cfg!(target_os = "linux")    { "Linux"   }
    else if cfg!(target_os = "macos")   { "macOS"   }
    else if cfg!(target_os = "windows") { "Windows" }
    else                                { "Unknown" }
}

pub fn is_windows() -> bool { cfg!(target_os = "windows") }

// ── Shell ─────────────────────────────────────────────────────────────────────
pub struct ShellInfo {
    pub name:       String,
    pub rc_file:    Option<PathBuf>,
    pub source_cmd: String,
}

pub fn detect_shell() -> ShellInfo {
    if is_windows() {
        return ShellInfo {
            name: "powershell".into(),
            rc_file: dirs::document_dir()
                .map(|d| d.join("PowerShell").join("Microsoft.PowerShell_profile.ps1")),
            source_cmd: ". $PROFILE".into(),
        };
    }

    let shell_env = std::env::var("SHELL").unwrap_or_default();
    let name = Path::new(&shell_env)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "bash".into());

    let home = dirs::home_dir().unwrap_or_default();
    let (rc, src) = match name.as_str() {
        "zsh"  => (Some(home.join(".zshrc")), "source ~/.zshrc".into()),
        "fish" => (
            dirs::config_dir().map(|c| c.join("fish").join("config.fish")),
            "source ~/.config/fish/config.fish".into(),
        ),
        "bash" => {
            let rc = if cfg!(target_os = "macos") { home.join(".bash_profile") } else { home.join(".bashrc") };
            (Some(rc), "source ~/.bashrc".into())
        }
        _ => (Some(home.join(".profile")), "source ~/.profile".into()),
    };

    ShellInfo { name, rc_file: rc, source_cmd: src }
}

pub fn path_export_line(dir: &Path) -> String {
    let s = dir.to_string_lossy();
    if is_windows() {
        format!("$env:PATH = \"{};$env:PATH\"", s)
    } else {
        format!("export PATH=\"{}:$PATH\"", s)
    }
}

pub fn path_apply_instruction() -> String {
    let shell = detect_shell();
    let rc = shell.rc_file.as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.profile".into());
    format!("Add to {} then run: {}", rc, shell.source_cmd)
}

pub fn reload_hint() -> String {
    let shell = detect_shell();
    if is_windows() {
        "Restart PowerShell to activate shims".into()
    } else {
        format!("Run: {}", shell.source_cmd)
    }
}

// ── Binary lookup ─────────────────────────────────────────────────────────────
pub fn bin_available(name: &str) -> bool {
    let cmd = if is_windows() { "where" } else { "which" };
    std::process::Command::new(cmd).arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn find_real_bin(names: &[&str], exclude_dir: &Path) -> Option<PathBuf> {
    let exclude = exclude_dir.to_string_lossy().to_string();
    let cmd = if is_windows() { "where" } else { "which" };
    for name in names {
        if let Ok(out) = std::process::Command::new(cmd).arg("-a").arg(name).output() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let p = PathBuf::from(line.trim());
                if p.exists() && !p.to_string_lossy().starts_with(&exclude) {
                    return Some(p);
                }
            }
        }
    }
    None
}

pub fn pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        let out = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output();
        out.map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
           .unwrap_or(false)
    }
    #[cfg(not(any(unix, windows)))]
    { false }
}

pub fn make_executable(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(path)?.permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(path, p)?;
    }
    let _ = path;
    Ok(())
}

pub fn data_dir_display() -> String {
    crate::config::data_dir().to_string_lossy().to_string()
}

pub fn install_instructions() -> Vec<String> {
    if cfg!(target_os = "linux") {
        vec!["curl -sSf https://matis-mem.sh | sh".into(), "  or: cargo install matis-mem".into()]
    } else if cfg!(target_os = "macos") {
        vec!["brew install matis-mem  (coming soon)".into(), "  or: cargo install matis-mem".into()]
    } else {
        vec!["cargo install matis-mem".into()]
    }
}
