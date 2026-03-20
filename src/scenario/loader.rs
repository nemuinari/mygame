use crate::scenario::types::Scene;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn load_scene(path: &Path) -> Result<Scene> {
    let content = fs::read_to_string(path)?;
    let scene: Scene = ron::from_str(&content)?;
    Ok(scene)
}
