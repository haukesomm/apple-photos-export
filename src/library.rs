use std::path::PathBuf;

use derive_new::new;

#[derive(new)]
pub struct PhotosLibrary {
    pub path: String
}

impl PhotosLibrary {

    pub fn db_path(&self) -> String {
        PathBuf::new()
            .join(&self.path)
            .join("database")
            .join("Photos.sqlite")
            .to_string_lossy()
            .to_string()
    }
}