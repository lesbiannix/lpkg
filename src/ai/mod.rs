use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

/// Loads assistant persona metadata from `ai/personas.json`.
pub fn load_personas(base_dir: impl AsRef<Path>) -> Result<Vec<Persona>> {
    let path = resolve(base_dir, "personas.json");
    read_json(path)
}

/// Loads the tracked task board from `ai/tasks.json`.
pub fn load_tasks(base_dir: impl AsRef<Path>) -> Result<TaskBoard> {
    let path = resolve(base_dir, "tasks.json");
    read_json(path)
}

/// Loads the current bug ledger from `ai/bugs.json`.
pub fn load_bugs(base_dir: impl AsRef<Path>) -> Result<Vec<Bug>> {
    let path = resolve(base_dir, "bugs.json");
    read_json(path)
}

fn resolve(base_dir: impl AsRef<Path>, file: &str) -> PathBuf {
    base_dir.as_ref().join("ai").join(file)
}

fn read_json<T>(path: PathBuf) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let data = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&data)?)
}

#[derive(Debug, Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub strengths: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskBoard {
    pub generated_at: String,
    pub unfinished: Vec<Task>,
    pub solved: Vec<Task>,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub blocked_on: Vec<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Bug {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}
