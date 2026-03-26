use std::path::{Path, PathBuf};

pub fn recursively_visit_files<P, C>(path: P, callback: &mut C) -> crate::Result<()>
where
    P: AsRef<Path>,
    C: FnMut(PathBuf) -> crate::Result<()>,
{
    for entry in path.as_ref().read_dir()?.into_iter().filter_map(Result::ok) {
        let filetype = &entry.file_type()?;

        if filetype.is_dir() {
            recursively_visit_files(entry.path().as_path(), callback)?;
        } else if filetype.is_file() {
            callback(entry.path())?;
        }
    }

    Ok(())
}
