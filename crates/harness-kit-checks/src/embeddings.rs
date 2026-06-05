use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const FORMAT_VERSION: u64 = 1;
pub const MODEL: &str = "gemini-embedding-2-preview";
pub const DEFAULT_DIMS: usize = 768;
const DEFAULT_TTL_SECONDS: u64 = 86_400;
const LOCAL_SOURCE: &str = "misty-step/harness-kit";
const BATCH_SIZE: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub name: String,
    pub source: String,
    pub fqn: String,
    pub description: String,
    pub search_document: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embedding: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingsData {
    pub format_version: u64,
    pub model: String,
    pub dimensions: usize,
    pub sources: Vec<String>,
    pub generated: String,
    pub local_content_hash: String,
    pub count: usize,
    pub items: Vec<EmbeddingItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingsMetadata {
    pub format_version: u64,
    pub model: String,
    pub dimensions: usize,
    pub index_sha256: String,
    pub registry_sha256: String,
    pub local_content_hash: String,
    pub generated: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenerateOptions {
    pub repo_root: PathBuf,
    pub dimensions: usize,
    pub dry_run: bool,
    pub local_only: bool,
    pub output: Option<PathBuf>,
    pub metadata_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchOptions {
    pub repo_root: PathBuf,
    pub query: Option<String>,
    pub project_dir: Option<PathBuf>,
    pub top: usize,
    pub item_type: Option<String>,
    pub json: bool,
    pub dimensions: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct ExternalSource {
    source: String,
    layout: String,
    skills_path: String,
}

pub fn cache_root() -> PathBuf {
    if let Some(value) = env::var_os("HARNESS_KIT_CACHE_DIR") {
        return PathBuf::from(value).expand_home();
    }
    if let Some(value) = env::var_os("CODEX_HOME") {
        return PathBuf::from(value).expand_home().join("cache/harness-kit");
    }
    if let Some(value) = env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(value).expand_home().join("harness-kit");
    }
    home_dir().join(".cache/harness-kit")
}

pub fn discovery_cache_paths() -> (PathBuf, PathBuf) {
    let cache_dir = cache_root().join("discovery");
    (
        cache_dir.join("embeddings.json"),
        cache_dir.join("embeddings-meta.json"),
    )
}

pub fn ttl_seconds() -> u64 {
    env::var("HARNESS_KIT_EMBEDDINGS_TTL_SECONDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_TTL_SECONDS)
}

pub fn sha256_bytes(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub fn sha256_text(text: &str) -> String {
    sha256_bytes(text.as_bytes())
}

pub fn repo_hashes(repo_root: &Path) -> Result<BTreeMap<String, String>> {
    let mut hashes = BTreeMap::new();
    hashes.insert(
        "index_sha256".to_string(),
        sha256_bytes(&fs::read(repo_root.join("index.yaml")).context("read index.yaml")?),
    );
    hashes.insert(
        "registry_sha256".to_string(),
        sha256_bytes(&fs::read(repo_root.join("registry.yaml")).context("read registry.yaml")?),
    );
    Ok(hashes)
}

pub fn metadata_matches(
    metadata: &EmbeddingsMetadata,
    model: &str,
    dimensions: usize,
    index_sha256: &str,
    registry_sha256: &str,
) -> bool {
    metadata.format_version == FORMAT_VERSION
        && metadata.model == model
        && metadata.dimensions == dimensions
        && metadata.index_sha256 == index_sha256
        && metadata.registry_sha256 == registry_sha256
}

pub fn is_stale(path: &Path, now: Option<SystemTime>) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return true;
    };
    let Ok(modified) = metadata.modified() else {
        return true;
    };
    let now = now.unwrap_or_else(SystemTime::now);
    now.duration_since(modified)
        .map(|age| age > Duration::from_secs(ttl_seconds()))
        .unwrap_or(false)
}

pub fn parse_frontmatter(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(captures) = Regex::new(r"(?s)^---\s*\n(.*?)\n---")
        .unwrap()
        .captures(text)
    else {
        return out;
    };
    let frontmatter = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
    for line in frontmatter.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                out.insert(key.trim().to_string(), value.to_string());
            }
        }
    }
    if (!out.contains_key("description")
        || out
            .get("description")
            .is_some_and(|description| description == "|"))
        && let Some(desc) = multiline_description(frontmatter)
    {
        out.insert("description".to_string(), desc);
    }
    out
}

pub fn synthesize_search_document(
    name: &str,
    frontmatter: &BTreeMap<String, String>,
    kind: &str,
    source: &str,
) -> String {
    let mut parts = vec![format!("Name: {name}."), format!("Source: {source}.")];
    if kind == "skill" {
        parts.push("Type: agent skill.".to_string());
    } else {
        parts.push("Type: agent persona.".to_string());
    }
    if let Some(description) = frontmatter.get("description")
        && !description.is_empty()
    {
        parts.push(format!("Description: {description}"));
    }
    parts.join(" ")
}

pub fn collect_local_items(repo_root: &Path) -> Result<Vec<EmbeddingItem>> {
    let mut items = Vec::new();
    let skills = repo_root.join("skills");
    if skills.exists() {
        let mut dirs = fs::read_dir(&skills)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        dirs.sort();
        for skill_dir in dirs {
            let skill_md = skill_dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let name = skill_dir
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let text = fs::read_to_string(&skill_md)?;
            let frontmatter = parse_frontmatter(&text);
            let Some(description) = frontmatter
                .get("description")
                .filter(|value| !value.is_empty())
            else {
                eprintln!("  SKIP skill {name}: no description");
                continue;
            };
            items.push(EmbeddingItem {
                item_type: "skill".to_string(),
                name: name.clone(),
                source: LOCAL_SOURCE.to_string(),
                fqn: format!("{LOCAL_SOURCE}@{name}"),
                description: description.clone(),
                search_document: synthesize_search_document(
                    &name,
                    &frontmatter,
                    "skill",
                    LOCAL_SOURCE,
                ),
                embedding: Vec::new(),
            });
        }
    }
    let agents = repo_root.join("agents");
    if agents.exists() {
        let mut files = fs::read_dir(&agents)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        files.sort();
        for agent_file in files {
            let name = agent_file
                .file_stem()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let text = fs::read_to_string(&agent_file)?;
            let frontmatter = parse_frontmatter(&text);
            let Some(description) = frontmatter
                .get("description")
                .filter(|value| !value.is_empty())
            else {
                eprintln!("  SKIP agent {name}: no description");
                continue;
            };
            items.push(EmbeddingItem {
                item_type: "agent".to_string(),
                name: name.clone(),
                source: LOCAL_SOURCE.to_string(),
                fqn: format!("{LOCAL_SOURCE}@{name}"),
                description: description.clone(),
                search_document: synthesize_search_document(
                    &name,
                    &frontmatter,
                    "agent",
                    LOCAL_SOURCE,
                ),
                embedding: Vec::new(),
            });
        }
    }
    Ok(items)
}

pub fn local_content_hash(repo_root: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut paths = Vec::new();
    let skills = repo_root.join("skills");
    if skills.exists() {
        for entry in fs::read_dir(&skills)? {
            let path = entry?.path().join("SKILL.md");
            if path.exists() {
                paths.push(path);
            }
        }
    }
    let agents = repo_root.join("agents");
    if agents.exists() {
        for entry in fs::read_dir(&agents)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    for path in paths {
        hasher.update(fs::read(path)?);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    let mag_a = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

pub fn synthesize_project_context(project_dir: &Path) -> String {
    let mut parts = Vec::new();
    for name in ["CLAUDE.md", "README.md"] {
        let path = project_dir.join(name);
        if path.exists() {
            if let Ok(text) = fs::read_to_string(path) {
                parts.push(take_chars(&text, 2000));
            }
            break;
        }
    }
    let package = project_dir.join("package.json");
    if package.exists()
        && let Ok(data) = fs::read_to_string(&package)
            .and_then(|text| serde_json::from_str::<Value>(&text).map_err(Into::into))
    {
        let deps = object_keys(data.get("dependencies"))
            .into_iter()
            .take(30)
            .collect::<Vec<_>>();
        let dev = object_keys(data.get("devDependencies"))
            .into_iter()
            .take(20)
            .collect::<Vec<_>>();
        if !deps.is_empty() {
            parts.push(format!("Dependencies: {}", deps.join(", ")));
        }
        if !dev.is_empty() {
            parts.push(format!("Dev dependencies: {}", dev.join(", ")));
        }
    }
    for (manifest, label) in [
        ("go.mod", "Go module"),
        ("mix.exs", "Elixir project"),
        ("Cargo.toml", "Rust project"),
        ("requirements.txt", "Python deps"),
        ("pyproject.toml", "Python project"),
    ] {
        let path = project_dir.join(manifest);
        if path.exists()
            && let Ok(text) = fs::read_to_string(path)
        {
            parts.push(format!("{label}: {}", take_chars(&text, 1000)));
        }
    }
    if let Ok(entries) = fs::read_dir(project_dir) {
        let mut dirs = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_dir())
            .filter_map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .filter(|name| !name.starts_with('.'))
            .collect::<Vec<_>>();
        dirs.sort();
        dirs.truncate(20);
        if !dirs.is_empty() {
            parts.push(format!("Directories: {}", dirs.join(", ")));
        }
    }
    if parts.is_empty() {
        "General software project".to_string()
    } else {
        parts.join("\n")
    }
}

pub fn generate_embeddings(options: &GenerateOptions) -> Result<()> {
    let (default_output, default_metadata) = discovery_cache_paths();
    let output_file = options.output.clone().unwrap_or(default_output);
    let metadata_file = options.metadata_path.clone().unwrap_or_else(|| {
        options
            .output
            .as_ref()
            .map(|output| {
                output.with_file_name(format!(
                    "{}-meta.json",
                    output.file_stem().unwrap_or_default().to_string_lossy()
                ))
            })
            .unwrap_or(default_metadata)
    });
    println!("Harness Kit Embeddings Generator");
    println!("  Model: {MODEL}");
    println!("  Dimensions: {}", options.dimensions);
    println!("  Output: {}", output_file.display());
    println!();

    println!("Local source: {LOCAL_SOURCE}");
    let mut items = collect_local_items(&options.repo_root)?;
    let local_skills = items
        .iter()
        .filter(|item| item.item_type == "skill")
        .count();
    let local_agents = items
        .iter()
        .filter(|item| item.item_type == "agent")
        .count();
    println!("  {local_skills} skills, {local_agents} agents");

    if !options.local_only {
        let sources = load_external_sources(&options.repo_root)?;
        println!();
        println!("External sources ({} from registry.yaml):", sources.len());
        for source in sources {
            items.extend(collect_external_source(&source)?);
        }
        println!();
    }

    items = dedupe_items(items);
    let sources = source_counts(&items);
    println!("Summary:");
    for (source, count) in &sources {
        println!("  {source}: {count}");
    }
    println!("  Total: {}", items.len());

    if options.dry_run {
        println!("\n--dry-run: would embed these items:\n");
        for item in &items {
            println!("  [{:5}] {}", item.item_type, item.fqn);
        }
        return Ok(());
    }

    println!(
        "\nEmbedding {} items in batches of {BATCH_SIZE}...",
        items.len()
    );
    let mut all_embeddings = Vec::new();
    for (index, batch) in items.chunks(BATCH_SIZE).enumerate() {
        let texts = batch
            .iter()
            .map(|item| item.search_document.clone())
            .collect::<Vec<_>>();
        let vectors = embed_texts(
            MODEL,
            &texts,
            options.dimensions,
            "RETRIEVAL_DOCUMENT",
            "harness-kit-generate-embeddings",
        )?;
        all_embeddings.extend(vectors);
        let done = usize::min((index + 1) * BATCH_SIZE, items.len());
        println!("  {done}/{} embedded", items.len());
        if done < items.len() {
            std::thread::sleep(Duration::from_millis(500));
        }
    }
    for (item, embedding) in items.iter_mut().zip(all_embeddings) {
        item.embedding = embedding;
    }
    let local_hash = local_content_hash(&options.repo_root)?;
    let generated = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let data = EmbeddingsData {
        format_version: FORMAT_VERSION,
        model: MODEL.to_string(),
        dimensions: options.dimensions,
        sources: sources.keys().cloned().collect(),
        generated: generated.clone(),
        local_content_hash: local_hash.clone(),
        count: items.len(),
        items,
    };
    let hashes = repo_hashes(&options.repo_root)?;
    let metadata = EmbeddingsMetadata {
        format_version: FORMAT_VERSION,
        model: MODEL.to_string(),
        dimensions: options.dimensions,
        index_sha256: hashes["index_sha256"].clone(),
        registry_sha256: hashes["registry_sha256"].clone(),
        local_content_hash: local_hash,
        generated,
        count: data.count,
    };
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = metadata_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_file, serde_json::to_string_pretty(&data)?)?;
    fs::write(&metadata_file, serde_json::to_string_pretty(&metadata)?)?;
    let size_kb = fs::metadata(&output_file)?.len() as f64 / 1024.0;
    println!(
        "\nWrote {}: {} items, {:.0} KB",
        output_file.display(),
        data.count,
        size_kb
    );
    println!("Wrote metadata: {}", metadata_file.display());
    Ok(())
}

pub fn search_embeddings(options: &SearchOptions) -> Result<()> {
    let data = ensure_embeddings(options)?;
    let mut items = data.items;
    if let Some(kind) = &options.item_type {
        items.retain(|item| &item.item_type == kind);
    }
    let query_text = if let Some(project_dir) = &options.project_dir {
        let text = synthesize_project_context(project_dir);
        if !options.json {
            eprintln!("Project context ({} chars):", text.len());
            eprintln!("  {}...", take_chars(&text, 200));
        }
        text
    } else {
        options.query.clone().unwrap_or_default()
    };
    let query_vec = embed_query(&query_text, data.dimensions)?;
    let mut scored = items
        .into_iter()
        .map(|item| (cosine_similarity(&query_vec, &item.embedding), item))
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    if options.json {
        let results = scored
            .iter()
            .take(options.top)
            .map(|(score, item)| {
                json!({
                    "score": (score * 10_000.0).round() / 10_000.0,
                    "type": item.item_type,
                    "name": item.name,
                    "source": item.source,
                    "fqn": item.fqn,
                    "description": take_chars(&item.description, 200),
                })
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }
    println!(
        "\nTop {} matches for: {}{}\n",
        options.top,
        take_chars(&query_text, 80),
        if query_text.chars().count() > 80 {
            "..."
        } else {
            ""
        }
    );
    for (rank, (score, item)) in scored.iter().take(options.top).enumerate() {
        let marker = if *score > 0.7 {
            "*"
        } else if *score > 0.5 {
            " "
        } else {
            "."
        };
        println!(
            "  {marker} {:2}. [{:5}] {}",
            rank + 1,
            item.item_type,
            item.fqn
        );
        println!(
            "       score: {:.4}  — {}",
            score,
            take_chars(&item.description, 100)
        );
        println!();
    }
    Ok(())
}

fn ensure_embeddings(options: &SearchOptions) -> Result<EmbeddingsData> {
    let (embeddings_file, metadata_file) = discovery_cache_paths();
    let hashes = repo_hashes(&options.repo_root)?;
    if embeddings_file.exists() && metadata_file.exists() {
        let metadata =
            serde_json::from_str::<EmbeddingsMetadata>(&fs::read_to_string(&metadata_file)?)?;
        if metadata_matches(
            &metadata,
            MODEL,
            options.dimensions,
            &hashes["index_sha256"],
            &hashes["registry_sha256"],
        ) && !is_stale(&embeddings_file, None)
        {
            return Ok(serde_json::from_str(&fs::read_to_string(
                &embeddings_file,
            )?)?);
        }
    }
    eprintln!("Embeddings cache missing or stale. Regenerating locally...");
    let generate = GenerateOptions {
        repo_root: options.repo_root.clone(),
        dimensions: options.dimensions,
        dry_run: false,
        local_only: false,
        output: Some(embeddings_file.clone()),
        metadata_path: Some(metadata_file),
    };
    if generate_embeddings(&generate).is_ok() && embeddings_file.exists() {
        return Ok(serde_json::from_str(&fs::read_to_string(
            &embeddings_file,
        )?)?);
    }
    if embeddings_file.exists() {
        eprintln!("Generation failed, using stale local cache.");
        return Ok(serde_json::from_str(&fs::read_to_string(
            &embeddings_file,
        )?)?);
    }
    bail!("unable to build local embeddings cache")
}

fn load_external_sources(repo_root: &Path) -> Result<Vec<ExternalSource>> {
    let registry = fs::read_to_string(repo_root.join("registry.yaml"))?;
    let yaml = serde_yaml::from_str::<Value>(&registry)?;
    let mut sources = Vec::new();
    for src in yaml
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let repo = src.get("repo").and_then(Value::as_str).unwrap_or_default();
        if repo.is_empty() || repo == LOCAL_SOURCE {
            continue;
        }
        sources.push(ExternalSource {
            source: repo.to_string(),
            layout: src
                .get("layout")
                .and_then(Value::as_str)
                .unwrap_or("flat")
                .to_string(),
            skills_path: src
                .get("skills_path")
                .and_then(Value::as_str)
                .unwrap_or("skills")
                .to_string(),
        });
    }
    Ok(sources)
}

fn collect_external_source(source: &ExternalSource) -> Result<Vec<EmbeddingItem>> {
    println!("  Fetching {}...", source.source);
    match source.layout.as_str() {
        "root" => collect_external_root(source),
        "multi-root" => collect_external_multi_root(source),
        _ => collect_external_flat(source),
    }
}

fn collect_external_root(source: &ExternalSource) -> Result<Vec<EmbeddingItem>> {
    let Some(text) = github_raw(&source.source, "SKILL.md")? else {
        eprintln!("    No SKILL.md found at root");
        return Ok(Vec::new());
    };
    let mut frontmatter = parse_frontmatter(&text);
    let name = frontmatter
        .get("name")
        .cloned()
        .unwrap_or_else(|| source.source.rsplit('/').next().unwrap_or("").to_string());
    if !frontmatter.contains_key("description") {
        frontmatter.insert(
            "description".to_string(),
            fallback_description(&text, &name),
        );
    }
    println!("    Found 1 skill: {name}");
    Ok(vec![external_item(source, &name, &frontmatter)])
}

fn collect_external_multi_root(source: &ExternalSource) -> Result<Vec<EmbeddingItem>> {
    let url = format!("https://api.github.com/repos/{}/contents/", source.source);
    let Some(Value::Array(entries)) = github_get(&url)? else {
        eprintln!("    Cannot list repo root");
        return Ok(Vec::new());
    };
    let mut dirs = entries
        .iter()
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("dir"))
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .filter(|name| !name.starts_with('.'))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    dirs.sort();
    let mut items = Vec::new();
    for dirname in dirs {
        let Some(text) = github_raw(&source.source, &format!("{dirname}/SKILL.md"))? else {
            continue;
        };
        let frontmatter = parse_frontmatter(&text);
        let name = frontmatter
            .get("name")
            .cloned()
            .unwrap_or_else(|| dirname.clone());
        if frontmatter.get("description").is_none_or(String::is_empty) {
            continue;
        }
        items.push(external_item(source, &name, &frontmatter));
    }
    println!("    Indexed {} skills with descriptions", items.len());
    Ok(items)
}

fn collect_external_flat(source: &ExternalSource) -> Result<Vec<EmbeddingItem>> {
    let url = format!(
        "https://api.github.com/repos/{}/contents/{}",
        source.source, source.skills_path
    );
    let Some(Value::Array(entries)) = github_get(&url)? else {
        eprintln!("    No skills directory at {}", source.skills_path);
        return Ok(Vec::new());
    };
    let mut dirs = entries
        .iter()
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("dir"))
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    dirs.sort();
    println!("    Found {} skill directories", dirs.len());
    let mut items = Vec::new();
    for dirname in dirs {
        let path = format!("{}/{dirname}/SKILL.md", source.skills_path);
        let Some(text) = github_raw(&source.source, &path)? else {
            continue;
        };
        let frontmatter = parse_frontmatter(&text);
        let name = frontmatter
            .get("name")
            .cloned()
            .unwrap_or_else(|| dirname.clone());
        if frontmatter.get("description").is_none_or(String::is_empty) {
            continue;
        }
        items.push(external_item(source, &name, &frontmatter));
    }
    println!("    Indexed {} skills with descriptions", items.len());
    Ok(items)
}

fn external_item(
    source: &ExternalSource,
    name: &str,
    frontmatter: &BTreeMap<String, String>,
) -> EmbeddingItem {
    let description = frontmatter.get("description").cloned().unwrap_or_default();
    EmbeddingItem {
        item_type: "skill".to_string(),
        name: name.to_string(),
        source: source.source.clone(),
        fqn: format!("{}@{name}", source.source),
        description,
        search_document: synthesize_search_document(name, frontmatter, "skill", &source.source),
        embedding: Vec::new(),
    }
}

fn embed_query(text: &str, dims: usize) -> Result<Vec<f64>> {
    Ok(embed_texts(
        MODEL,
        &[text.to_string()],
        dims,
        "RETRIEVAL_QUERY",
        "harness-kit-search",
    )?
    .remove(0))
}

fn embed_texts(
    model: &str,
    texts: &[String],
    output_dimensionality: usize,
    task_type: &str,
    user_agent: &str,
) -> Result<Vec<Vec<f64>>> {
    let key = env::var("GEMINI_API_KEY")
        .or_else(|_| env::var("GOOGLE_API_KEY"))
        .context("GEMINI_API_KEY or GOOGLE_API_KEY required")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:batchEmbedContents?key={}",
        percent_encode(&key)
    );
    let requests = texts
        .iter()
        .map(|text| {
            json!({
                "model": format!("models/{model}"),
                "content": {"parts": [{"text": text}]},
                "taskType": task_type,
                "outputDimensionality": output_dimensionality,
            })
        })
        .collect::<Vec<_>>();
    let body = curl_json(
        &url,
        &[
            ("Content-Type", "application/json"),
            ("User-Agent", user_agent),
        ],
        Some(&serde_json::to_vec(&json!({ "requests": requests }))?),
    )
    .with_context(|| "Gemini embeddings request failed")?;
    let embeddings = body
        .get("embeddings")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("Unexpected embeddings response: {body}"))?;
    let mut values = Vec::new();
    for embedding in embeddings {
        let vector = embedding
            .get("values")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("Unexpected embedding payload: {embedding}"))?
            .iter()
            .map(|value| {
                value
                    .as_f64()
                    .ok_or_else(|| anyhow::anyhow!("embedding value is not numeric"))
            })
            .collect::<Result<Vec<_>>>()?;
        values.push(vector);
    }
    Ok(values)
}

fn github_token() -> Option<String> {
    if let Ok(token) = env::var("GITHUB_TOKEN")
        && !token.is_empty()
    {
        return Some(token);
    }
    let output = Command::new("gh").args(["auth", "token"]).output().ok()?;
    if output.status.success() {
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

fn github_get(url: &str) -> Result<Option<Value>> {
    let headers = github_request_headers();
    match curl_bytes(url, &headers, None) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(CurlFailure::Status(404, _)) => Ok(None),
        Err(CurlFailure::Status(code, _)) => {
            eprintln!("  GitHub API error {code}: {url}");
            Ok(None)
        }
        Err(error) => {
            eprintln!("  Fetch error: {error}");
            Ok(None)
        }
    }
}

fn github_raw(source: &str, path: &str) -> Result<Option<String>> {
    let headers = github_request_headers();
    for branch in ["main", "master"] {
        let url = format!("https://raw.githubusercontent.com/{source}/{branch}/{path}");
        match curl_bytes(&url, &headers, None) {
            Ok(bytes) => return Ok(Some(String::from_utf8(bytes)?)),
            Err(CurlFailure::Status(404, _)) => continue,
            Err(_) => return Ok(None),
        }
    }
    Ok(None)
}

fn github_request_headers() -> Vec<(String, String)> {
    let mut headers = vec![(
        "Accept".to_string(),
        "application/vnd.github.v3+json".to_string(),
    )];
    if let Some(token) = github_token() {
        headers.push(("Authorization".to_string(), format!("token {token}")));
    }
    headers
}

#[derive(Debug)]
enum CurlFailure {
    Status(i32, String),
    Io(std::io::Error),
}

impl std::fmt::Display for CurlFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurlFailure::Status(code, detail) => write!(formatter, "HTTP {code}: {detail}"),
            CurlFailure::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CurlFailure {}

fn curl_json(url: &str, headers: &[(&str, &str)], body: Option<&[u8]>) -> Result<Value> {
    let owned_headers = headers
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect::<Vec<_>>();
    Ok(serde_json::from_slice(&curl_bytes(
        url,
        &owned_headers,
        body,
    )?)?)
}

fn curl_bytes(
    url: &str,
    headers: &[(String, String)],
    body: Option<&[u8]>,
) -> std::result::Result<Vec<u8>, CurlFailure> {
    let mut command = Command::new("curl");
    command.args(["-sS", "-L", "--max-time", "60", "-w", "\n%{http_code}", url]);
    for (key, value) in headers {
        command.args(["-H", &format!("{key}: {value}")]);
    }
    if body.is_some() {
        command.args(["-X", "POST", "--data-binary", "@-"]);
        command.stdin(std::process::Stdio::piped());
    }
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let mut child = command.spawn().map_err(CurlFailure::Io)?;
    if let Some(body) = body
        && let Some(stdin) = child.stdin.as_mut()
    {
        stdin.write_all(body).map_err(CurlFailure::Io)?;
    }
    let output = child.wait_with_output().map_err(CurlFailure::Io)?;
    let stdout = output.stdout;
    let split_at = stdout
        .iter()
        .rposition(|byte| *byte == b'\n')
        .unwrap_or(stdout.len());
    let (payload, status_bytes) = stdout.split_at(split_at);
    let status = String::from_utf8_lossy(status_bytes)
        .trim()
        .parse::<i32>()
        .unwrap_or(0);
    if output.status.success() && (200..300).contains(&status) {
        Ok(payload.to_vec())
    } else {
        let detail = if payload.is_empty() {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::from_utf8_lossy(payload).to_string()
        };
        Err(CurlFailure::Status(status, detail))
    }
}

fn source_counts(items: &[EmbeddingItem]) -> BTreeMap<String, usize> {
    let mut sources = BTreeMap::new();
    for item in items {
        *sources.entry(item.source.clone()).or_insert(0) += 1;
    }
    sources
}

fn dedupe_items(items: Vec<EmbeddingItem>) -> Vec<EmbeddingItem> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for item in items {
        if seen.insert(item.fqn.clone()) {
            out.push(item);
        }
    }
    out
}

fn fallback_description(text: &str, name: &str) -> String {
    let body = Regex::new(r"(?s)^---.*?---\s*")
        .unwrap()
        .replace(text, "")
        .trim()
        .to_string();
    let mut first = body
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .replace('\n', " ")
        .trim()
        .to_string();
    if first.starts_with('#') {
        first.clear();
    }
    if first.is_empty() {
        name.to_string()
    } else {
        take_chars(&first, 500)
    }
}

fn multiline_description(frontmatter: &str) -> Option<String> {
    let mut lines = Vec::new();
    let mut in_description = false;
    for line in frontmatter.lines() {
        if in_description {
            if line.starts_with(' ') || line.starts_with('\t') {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    lines.push(trimmed.to_string());
                }
                continue;
            }
            break;
        }
        if line.trim_start().starts_with("description:") {
            in_description = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

fn object_keys(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_object)
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
}

fn take_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

fn percent_encode(text: &str) -> String {
    text.bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

trait ExpandHome {
    fn expand_home(self) -> PathBuf;
}

impl ExpandHome for PathBuf {
    fn expand_home(self) -> PathBuf {
        let text = self.to_string_lossy();
        if text == "~" {
            return home_dir();
        }
        if let Some(rest) = text.strip_prefix("~/") {
            return home_dir().join(rest);
        }
        self
    }
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[allow(dead_code)]
fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_extracts_multiline_description() {
        let fm = parse_frontmatter(
            "---\nname: demo\ndescription: |\n  First line\n  second line\n---\nbody",
        );
        assert_eq!(fm["name"], "demo");
        assert_eq!(fm["description"], "First line second line");
    }

    #[test]
    fn cache_metadata_and_staleness_match_python_contract() {
        let metadata = EmbeddingsMetadata {
            format_version: FORMAT_VERSION,
            model: MODEL.to_string(),
            dimensions: 768,
            index_sha256: "i".to_string(),
            registry_sha256: "r".to_string(),
            local_content_hash: "l".to_string(),
            generated: "now".to_string(),
            count: 1,
        };
        assert!(metadata_matches(&metadata, MODEL, 768, "i", "r"));
        assert!(!metadata_matches(&metadata, MODEL, 256, "i", "r"));
        assert!(is_stale(Path::new("/definitely/missing"), None));
    }

    #[test]
    fn local_collection_and_hash_are_sorted_and_skip_missing_descriptions() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("skills/a")).unwrap();
        fs::create_dir_all(temp.path().join("skills/b")).unwrap();
        fs::write(
            temp.path().join("skills/a/SKILL.md"),
            "---\ndescription: Alpha skill\n---\n",
        )
        .unwrap();
        fs::write(temp.path().join("skills/b/SKILL.md"), "---\nname: b\n---\n").unwrap();
        let items = collect_local_items(temp.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].fqn, "misty-step/harness-kit@a");
        assert_eq!(
            items[0].search_document,
            "Name: a. Source: misty-step/harness-kit. Type: agent skill. Description: Alpha skill"
        );
        assert_eq!(local_content_hash(temp.path()).unwrap().len(), 64);
    }

    #[test]
    fn project_context_reads_manifests_dependencies_and_dirs() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("README.md"), "Readme").unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"dependencies":{"react":"1"},"devDependencies":{"vite":"1"}}"#,
        )
        .unwrap();
        fs::write(temp.path().join("Cargo.toml"), "[package]\nname='x'").unwrap();
        let context = synthesize_project_context(temp.path());
        assert!(context.contains("Readme"));
        assert!(context.contains("Dependencies: react"));
        assert!(context.contains("Dev dependencies: vite"));
        assert!(context.contains("Rust project:"));
        assert!(context.contains("Directories: src"));
    }

    #[test]
    fn cosine_similarity_handles_zero_and_ranks_unit_vectors() {
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
        assert!((cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn dedupe_keeps_first_fqn_and_source_counts_are_sorted() {
        let item = |fqn: &str, source: &str| EmbeddingItem {
            item_type: "skill".to_string(),
            name: fqn.to_string(),
            source: source.to_string(),
            fqn: fqn.to_string(),
            description: String::new(),
            search_document: String::new(),
            embedding: Vec::new(),
        };
        let items = dedupe_items(vec![item("a", "z"), item("a", "x"), item("b", "x")]);
        assert_eq!(items.len(), 2);
        let counts = source_counts(&items);
        assert_eq!(counts.keys().cloned().collect::<Vec<_>>(), vec!["x", "z"]);
    }

    #[test]
    fn percent_encode_escapes_api_key_bytes() {
        assert_eq!(percent_encode("a b/+"), "a%20b%2F%2B");
    }
}
