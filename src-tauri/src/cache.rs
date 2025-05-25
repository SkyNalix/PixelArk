use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub fn load_cache_set(cache_dir: &PathBuf) -> HashSet<PathBuf> {
    if !cache_dir.is_dir() {
        log::warn!("Provided cache path is not a directory: {:?}", cache_dir);
        return HashSet::new();
    }

    let entries = fs::read_dir(cache_dir);
    if let Err(e) = entries {
        log::warn!("Failed to read cache directory: {:?}", e);
        return HashSet::new();
    }

    entries.unwrap().filter_map(
        |entry| {
            match entry {
                Err(e) => {
                    log::error!("Failed to read a file entry in {:?}: {}", cache_dir, e);
                    None
                }
                Ok(entry) => {
                    Some(entry.path())
                }
            }
        }
    ).collect()
}

pub(crate) fn find_cached_file<'a>(file_name: &str, cache_set: &'a HashSet<PathBuf>) -> Option<&'a PathBuf> {
    cache_set.iter().find(|p| {
        p.file_stem()
            .and_then(|f| f.to_str())
            .map(|f| f == file_name)
            .unwrap_or(false)
    })
}