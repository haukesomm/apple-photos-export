use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn recursively_get_files<P: Into<PathBuf>>(directory: P) -> HashSet<PathBuf> {
    WalkDir::new(directory.into())
        .into_iter()
        .filter_map(|entry| entry.map(|entry| entry.path().to_path_buf()).ok())
        .filter_map(|p| p.is_file().then_some(p))
        .collect()
}
