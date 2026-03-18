use std::fs::DirEntry;
use std::path::{Path, PathBuf};

pub fn recursively_visit_files<P, C>(path: P, callback: &mut C) -> crate::Result<()>
where
    P: AsRef<Path>,
    C: FnMut(PathBuf) -> crate::Result<()>,
{
    // This is inefficient but needed in order to achieve consistent results between runs
    // (fs::read_dir is non-deterministic in terms of sorting order)
    let mut entries: Vec<DirEntry> = path
        .as_ref()
        .read_dir()?
        .collect::<std::result::Result<Vec<DirEntry>, _>>()?;

    entries.sort_by_key(DirEntry::path);

    for entry in entries {
        let filetype = &entry.file_type()?;

        if filetype.is_dir() {
            recursively_visit_files(entry.path().as_path(), callback)?;
        } else if filetype.is_file() {
            callback(entry.path())?;
        }
    }

    Ok(())
}
