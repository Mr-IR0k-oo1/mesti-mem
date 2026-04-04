use anyhow::{Context, Result};
use std::path::{Path};
use crate::config::{external_dir, shims_dir};
use crate::platform::{find_real_bin, make_executable, is_windows};

pub struct AgentSpec {
    pub name: &'static str,
    pub bins: &'static [&'static str],
    pub mode: ShimMode,
}

#[derive(Clone, Copy)]
pub enum ShimMode { ArgLast, PrintFlag, PFlag, FirstArg, Interactive }

pub const AGENTS: &[AgentSpec] = &[
    AgentSpec { name: "claude",  bins: &["claude"],        mode: ShimMode::PrintFlag  },
    AgentSpec { name: "amp",     bins: &["amp"],            mode: ShimMode::FirstArg   },
    AgentSpec { name: "gemini",  bins: &["gemini"],         mode: ShimMode::PFlag      },
    AgentSpec { name: "vibe",    bins: &["vibe","cursor"],  mode: ShimMode::ArgLast    },
    AgentSpec { name: "aider",   bins: &["aider"],          mode: ShimMode::Interactive},
    AgentSpec { name: "copilot", bins: &["gh"],             mode: ShimMode::Interactive},
    AgentSpec { name: "mistral", bins: &["mistral"],        mode: ShimMode::ArgLast    },
    AgentSpec { name: "ollama",  bins: &["ollama"],         mode: ShimMode::ArgLast    },
];

pub fn install_all() -> Result<(usize, usize, usize)> {
    let dir = shims_dir();
    std::fs::create_dir_all(&dir)?;
    let (mut installed, mut already, mut not_found) = (0, 0, 0);
    for spec in AGENTS {
        match install_one(spec, &dir) {
            Ok(true)  => installed += 1,
            Ok(false) => already   += 1,
            Err(_)    => not_found += 1,
        }
    }
    Ok((installed, already, not_found))
}

pub fn install_one(spec: &AgentSpec, shim_dir: &Path) -> Result<bool> {
    let real = find_real_bin(spec.bins, shim_dir)
        .with_context(|| format!("binary not found for {}", spec.name))?;
    let log_dir = external_dir().join(spec.name);
    if is_windows() {
        install_windows(spec, &real, &log_dir, shim_dir)
    } else {
        install_unix(spec, &real, &log_dir, shim_dir)
    }
}

fn install_unix(spec: &AgentSpec, real: &Path, log_dir: &Path, shim_dir: &Path) -> Result<bool> {
    let shim = shim_dir.join(spec.name);
    if shim.exists() {
        if std::fs::read_to_string(&shim).unwrap_or_default().contains("matis-mem shim") {
            return Ok(false);
        }
    }
    let script = bash_shim(spec.name, real, log_dir, spec.mode);
    let tmp = shim.with_extension("tmp");
    std::fs::write(&tmp, &script)?;
    make_executable(&tmp)?;
    std::fs::rename(tmp, &shim)?;
    Ok(true)
}

fn install_windows(spec: &AgentSpec, real: &Path, log_dir: &Path, shim_dir: &Path) -> Result<bool> {
    let ps1 = shim_dir.join(format!("{}.ps1", spec.name));
    if ps1.exists() {
        if std::fs::read_to_string(&ps1).unwrap_or_default().contains("matis-mem") {
            return Ok(false);
        }
    }
    let script = powershell_shim(spec.name, real, log_dir);
    let tmp = ps1.with_extension("tmp");
    std::fs::write(&tmp, &script)?;
    std::fs::rename(tmp, &ps1)?;

    let cmd = shim_dir.join(format!("{}.cmd", spec.name));
    let launcher = format!(
        "@echo off\nREM matis-mem shim launcher for: {}\npowershell.exe -NoProfile -ExecutionPolicy Bypass -File \"%~dp0{}.ps1\" %*\n",
        spec.name, spec.name
    );
    std::fs::write(&cmd, launcher)?;
    Ok(true)
}

pub fn uninstall_all() -> Result<usize> {
    let dir = shims_dir();
    if !dir.exists() { return Ok(0); }
    let mut removed = 0usize;
    for e in std::fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
        let c = std::fs::read_to_string(e.path()).unwrap_or_default();
        if c.contains("matis-mem") {
            std::fs::remove_file(e.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub struct ShimStatus {
    pub name:           &'static str,
    pub installed:      bool,
    pub active_in_path: bool,
    pub real_exists:    bool,
}

pub fn status() -> Vec<ShimStatus> {
    let shim_dir = shims_dir();
    AGENTS.iter().map(|spec| {
        let shim_file = if is_windows() {
            shim_dir.join(format!("{}.cmd", spec.name))
        } else {
            shim_dir.join(spec.name)
        };
        let installed = shim_file.exists() && {
            std::fs::read_to_string(&shim_file)
                .map(|c| c.contains("matis-mem"))
                .unwrap_or(false)
        };
        let active_in_path = if installed {
            let path_env = std::env::var("PATH").or_else(|_| std::env::var("Path")).unwrap_or_default();
            let sep = if is_windows() { ';' } else { ':' };
            let shim_str = shim_dir.to_string_lossy().to_string();
            path_env.split(sep).any(|p| p == shim_str)
        } else { false };
        let real_exists = find_real_bin(spec.bins, &shim_dir).is_some();
        ShimStatus { name: spec.name, installed, active_in_path, real_exists }
    }).collect()
}

pub fn path_export_line() -> String {
    crate::platform::path_export_line(&shims_dir())
}

// ── Bash shim ─────────────────────────────────────────────────────────────────

fn bash_shim(name: &str, real: &Path, log_dir: &Path, mode: ShimMode) -> String {
    let real    = real.display();
    let log_dir = log_dir.display();

    let helpers = r#"
json_str() {
  local s="$1"
  s="${s//\\/\\\\}"; s="${s//\"/\\\"}"; s="${s//$'\n'/\\n}"; s="${s//$'\t'/\\t}"
  printf '%s' "$s"
}
CWD=$(pwd)
GIT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || true)
PROJECT=$(basename "${GIT_ROOT:-$CWD}")"#;

    let write_log = format!(
r#"mkdir -p "{log_dir}"
TS=$(date +%Y%m%d_%H%M%S_%3N 2>/dev/null || date +%Y%m%d_%H%M%S)
cat > "{log_dir}/$TS.json" <<JSON
{{
  "id":"$(json_str "$TS")","agent":"{name}","cwd":"$(json_str "$CWD")",
  "project":"$(json_str "$PROJECT")","args":"$(json_str "$ARGS_STR")",
  "input":"$(json_str "$INPUT")","output":"$(json_str "$OUTPUT")",
  "duration_ms":$DURATION,"exit_code":$EXIT_CODE,
  "timestamp":"$(date -Iseconds 2>/dev/null || date)","capture":"$CAPTURE_MODE"
}}
JSON"#, log_dir=log_dir, name=name);

    let header = format!("#!/usr/bin/env bash\n# matis-mem shim for: {name}\n{helpers}\nARGS_STR=\"$*\"\n");

    match mode {
        ShimMode::Interactive => format!(
r#"{header}INPUT=""; START=$(date +%s%3N 2>/dev/null || echo 0)
"{real}" "$@"; EXIT_CODE=$?
END=$(date +%s%3N 2>/dev/null || echo 0); DURATION=$((END-START))
OUTPUT="(interactive)"; CAPTURE_MODE="interactive"
{write_log}
exit $EXIT_CODE"#),

        ShimMode::PrintFlag => format!(
r#"{header}START=$(date +%s%3N 2>/dev/null || echo 0)
if [[ "$*" == *"--print"* ]] || [[ ! -t 0 ]]; then
  OUTPUT=$("{real}" "$@" 2>&1); EXIT_CODE=$?; CAPTURE_MODE="full"
  INPUT=$(echo "$*" | sed 's/.*--print[= ]//')
else
  "{real}" "$@"; EXIT_CODE=$?; OUTPUT="(interactive)"; CAPTURE_MODE="interactive"; INPUT=""
fi
END=$(date +%s%3N 2>/dev/null || echo 0); DURATION=$((END-START))
{write_log}
[[ "$CAPTURE_MODE" == "full" ]] && echo "$OUTPUT"
exit $EXIT_CODE"#),

        ShimMode::PFlag => format!(
r#"{header}START=$(date +%s%3N 2>/dev/null || echo 0)
if [[ "$*" == *"-p "* ]] || [[ "$*" == *"--prompt"* ]]; then
  OUTPUT=$("{real}" "$@" 2>&1); EXIT_CODE=$?; CAPTURE_MODE="full"
  INPUT=$(echo "$*" | sed 's/.*-p //')
else
  "{real}" "$@"; EXIT_CODE=$?; OUTPUT="(interactive)"; CAPTURE_MODE="interactive"; INPUT=""
fi
END=$(date +%s%3N 2>/dev/null || echo 0); DURATION=$((END-START))
{write_log}
[[ "$CAPTURE_MODE" == "full" ]] && echo "$OUTPUT"
exit $EXIT_CODE"#),

        ShimMode::ArgLast | ShimMode::FirstArg => format!(
r#"{header}INPUT="${{@: -1}}"; START=$(date +%s%3N 2>/dev/null || echo 0)
OUTPUT=$("{real}" "$@" 2>&1); EXIT_CODE=$?
END=$(date +%s%3N 2>/dev/null || echo 0); DURATION=$((END-START)); CAPTURE_MODE="full"
{write_log}
echo "$OUTPUT"; exit $EXIT_CODE"#),
    }
}

// ── PowerShell shim ───────────────────────────────────────────────────────────

fn powershell_shim(name: &str, real: &Path, log_dir: &Path) -> String {
    let real    = real.display().to_string().replace('\\', "\\\\");
    let log_dir = log_dir.display().to_string().replace('\\', "\\\\");
    format!(
r#"# matis-mem shim for: {name}
$RealBin = "{real}"; $LogDir = "{log_dir}"
$CWD = (Get-Location).Path
$GitRoot = (git rev-parse --show-toplevel 2>$null)
$Project = if ($GitRoot) {{ Split-Path $GitRoot -Leaf }} else {{ Split-Path $CWD -Leaf }}
$ArgsStr = $args -join ' '; $Input = if ($args.Count -gt 0) {{ $args[-1] }} else {{ "" }}
$Start = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$Output = & $RealBin @args 2>&1 | Out-String; $ExitCode = $LASTEXITCODE
$Duration = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() - $Start
$TS = Get-Date -Format "yyyyMMdd_HHmmss_fff"; $ISO = (Get-Date).ToString("o")
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
@{{ id=$TS; agent="{name}"; cwd=$CWD; project=$Project; args=$ArgsStr; input=$Input;
    output=$Output; duration_ms=$Duration; exit_code=$ExitCode; timestamp=$ISO; capture="full"
}} | ConvertTo-Json -Compress | Set-Content -Path "$LogDir\$TS.json" -Encoding UTF8
Write-Output $Output; exit $ExitCode"#, name=name, real=real, log_dir=log_dir)
}
