use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use crate::config::knowledge_dir;

/// Stored as markdown:
/// ```
/// # Topic Name
/// tags: rust, async
///
/// - Note one
/// - Note two
/// ```
#[derive(Debug, Clone)]
pub struct Knowledge {
    pub topic: String,
    pub notes: Vec<String>,
    pub tags:  Vec<String>,
}

impl Knowledge {
    pub fn new(topic: impl Into<String>) -> Self {
        Self { topic: topic.into(), notes: vec![], tags: vec![] }
    }

    // ── Validation ────────────────────────────────────────────────────────────
    pub fn validate_topic(topic: &str) -> Result<()> {
        let t = topic.trim();
        if t.len() < 3  { bail!("topic must be at least 3 characters"); }
        if t.len() > 80 { bail!("topic too long (max 80 characters)"); }
        Ok(())
    }

    // ── Filename ──────────────────────────────────────────────────────────────
    pub fn filename(topic: &str) -> String {
        topic.trim().to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
            .collect::<String>()
            .split('_').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("_")
    }

    pub fn md_path(topic: &str) -> PathBuf {
        knowledge_dir().join(format!("{}.md", Self::filename(topic)))
    }

    // ── Markdown serialisation ────────────────────────────────────────────────
    pub fn to_markdown(&self) -> String {
        let mut out = format!("# {}\n", self.topic);
        if !self.tags.is_empty() {
            out += &format!("tags: {}\n", self.tags.join(", "));
        }
        out += "\n";
        for n in &self.notes { out += &format!("- {}\n", n); }
        out
    }

    pub fn from_markdown(raw: &str) -> Result<Self> {
        let mut topic = String::new();
        let mut tags  = vec![];
        let mut notes = vec![];
        for line in raw.lines() {
            let l = line.trim();
            if l.starts_with("# ") && topic.is_empty() {
                topic = l[2..].trim().to_string();
            } else if l.starts_with("tags:") {
                tags = l["tags:".len()..].split(',')
                    .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            } else if l.starts_with("- ") || l.starts_with("* ") {
                notes.push(l[2..].trim().to_string());
            }
        }
        if topic.is_empty() { bail!("missing # Title header"); }
        Ok(Self { topic, notes, tags })
    }

    // ── Persistence ───────────────────────────────────────────────────────────
    pub fn save(&self) -> Result<()> {
        let p = Self::md_path(&self.topic);
        if let Some(d) = p.parent() { std::fs::create_dir_all(d)?; }
        let tmp = p.with_extension("tmp");
        std::fs::write(&tmp, self.to_markdown())?;
        std::fs::rename(tmp, p)?;
        Ok(())
    }

    pub fn load(topic: &str) -> Result<Self> {
        let p = Self::md_path(topic);
        let raw = std::fs::read_to_string(&p)
            .with_context(|| format!("reading {}", p.display()))?;
        Self::from_markdown(&raw)
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        Self::from_markdown(&std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?)
    }

    pub fn delete(topic: &str) -> Result<()> {
        let p = Self::md_path(topic);
        if p.exists() { std::fs::remove_file(p)?; }
        Ok(())
    }

    pub fn list() -> Result<Vec<String>> {
        let dir = knowledge_dir();
        if !dir.exists() { return Ok(vec![]); }
        let mut topics: Vec<String> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            .filter_map(|e| {
                let raw = std::fs::read_to_string(e.path()).ok()?;
                Some(Self::from_markdown(&raw).ok()?.topic)
            })
            .collect();
        topics.sort();
        Ok(topics)
    }

    pub fn search(query: &str) -> Result<Vec<Self>> {
        let q = query.to_lowercase();
        let dir = knowledge_dir();
        if !dir.exists() { return Ok(vec![]); }
        let mut results = vec![];
        for entry in std::fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|x| x.to_str()) != Some("md") { continue; }
            if let Ok(raw) = std::fs::read_to_string(entry.path()) {
                if raw.to_lowercase().contains(&q) {
                    if let Ok(k) = Self::from_markdown(&raw) { results.push(k); }
                }
            }
        }
        Ok(results)
    }

    pub fn as_context(&self) -> String {
        format!("[KNOWLEDGE: {}]\n{}\n", self.topic, self.notes.join("; "))
    }

    // ── Import ────────────────────────────────────────────────────────────────
    pub fn import_from_file(path: &Path) -> Result<String> {
        let k = Self::load_from_path(path)?;
        k.save()?;
        Ok(k.topic)
    }

    pub fn import_from_dir(dir: &Path) -> Result<Vec<String>> {
        let mut imported = vec![];
        for entry in std::fs::read_dir(dir)?.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.extension().and_then(|x| x.to_str()) == Some("md") {
                if let Ok(topic) = Self::import_from_file(&p) { imported.push(topic); }
            }
        }
        Ok(imported)
    }

    // ── Export ────────────────────────────────────────────────────────────────
    pub fn export_to_file(topic: &str, dest: &Path) -> Result<()> {
        let k = Self::load(topic)?;
        if let Some(d) = dest.parent() { std::fs::create_dir_all(d)?; }
        std::fs::write(dest, k.to_markdown())?;
        Ok(())
    }

    pub fn export_all(dest_dir: &Path) -> Result<Vec<PathBuf>> {
        std::fs::create_dir_all(dest_dir)?;
        let mut exported = vec![];
        for topic in Self::list()? {
            let k    = Self::load(&topic)?;
            let file = dest_dir.join(format!("{}.md", Self::filename(&topic)));
            std::fs::write(&file, k.to_markdown())?;
            exported.push(file);
        }
        Ok(exported)
    }

    pub fn export_bundle(dest: &Path) -> Result<usize> {
        let topics = Self::list()?;
        let count  = topics.len();
        let mut out = format!("# matis-mem Knowledge Bundle\n\n> Exported: {}\n\n---\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M"));
        for topic in &topics {
            if let Ok(k) = Self::load(topic) {
                out += &k.to_markdown();
                out += "\n---\n\n";
            }
        }
        if let Some(d) = dest.parent() { std::fs::create_dir_all(d)?; }
        std::fs::write(dest, out)?;
        Ok(count)
    }
}
