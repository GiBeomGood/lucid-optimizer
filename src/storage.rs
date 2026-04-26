use crate::item::Item;
use crate::stats::BaseStats;
use std::fs;
use std::io;
use std::path::Path;

pub fn load(path: &str) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let items: Vec<Item> = serde_json::from_str(trimmed)?;
    Ok(items)
}

pub fn load_stats(path: &str) -> Result<BaseStats, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Ok(BaseStats::default());
    }
    let content = fs::read_to_string(path)?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(BaseStats::default());
    }
    Ok(serde_json::from_str(trimmed)?)
}

pub fn save_stats(path: &str, stats: &BaseStats) -> io::Result<()> {
    if Path::new(path).exists() {
        let bak = format!("{path}.bak");
        let _ = fs::copy(path, bak);
    }
    let json = serde_json::to_string_pretty(stats).map_err(io::Error::other)?;
    fs::write(path, json)
}

pub fn save(path: &str, items: &[Item]) -> io::Result<()> {
    if Path::new(path).exists() {
        let bak = format!("{path}.bak");
        let _ = fs::copy(path, bak);
    }
    let json = serde_json::to_string_pretty(items)
        .map_err(io::Error::other)?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{Item, ItemOption, OptionKind};

    fn test_item() -> Item {
        Item {
            options: [
                ItemOption { kind: OptionKind::Magic, value: 10 },
                ItemOption { kind: OptionKind::CritRate, value: 7 },
            ],
        }
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let result = load("__nonexistent_test_9a3f__.json");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let path = std::env::temp_dir().join("lucid_test_roundtrip.json");
        let path_str = path.to_str().unwrap();
        let items = vec![test_item()];
        save(path_str, &items).unwrap();
        let loaded = load(path_str).unwrap();
        assert_eq!(items, loaded);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{path_str}.bak"));
    }

    #[test]
    fn load_empty_file_returns_empty() {
        let path = std::env::temp_dir().join("lucid_test_empty.json");
        let path_str = path.to_str().unwrap();
        fs::write(path_str, "").unwrap();
        let result = load(path_str).unwrap();
        assert!(result.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_invalid_json_errors() {
        let path = std::env::temp_dir().join("lucid_test_invalid.json");
        let path_str = path.to_str().unwrap();
        fs::write(path_str, "not valid json").unwrap();
        let result = load(path_str);
        assert!(result.is_err());
        let _ = fs::remove_file(&path);
    }
}
