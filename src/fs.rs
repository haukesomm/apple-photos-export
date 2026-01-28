use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn recursively_get_files<P: Into<PathBuf>>(directory: P) -> HashSet<PathBuf> {
    WalkDir::new(directory.into())
        .into_iter()
        // Convert entries to paths
        .filter_map(|entry| entry.map(|entry| entry.path().to_path_buf()).ok())
        // Filter out directories
        .filter_map(|p| p.is_file().then_some(p))
        // Canonicalize paths in order to be able to compare them across multiple file
        // systems, e.g. when working with mounted SAMBA shares in combination with the --skip or
        // --delete flags.
        // Since we obtained the path by iterating over the output directory, this should never fail
        // unless a file or directory has been deleted while the export is running.
        .filter_map(|p| p.canonicalize().ok())
        .collect()
}
