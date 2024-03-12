use std::path::PathBuf;

pub struct PhotosLibrary {
    pub path: String
}

impl PhotosLibrary {

    pub fn new(path: String) -> PhotosLibrary {
        PhotosLibrary { path }
    }

    pub fn db_path(&self) -> String {
        PathBuf::new()
            .join(&self.path)
            .join("database")
            .join("Photos.sqlite")
            .to_string_lossy()
            .to_string()
    }

    pub fn original_assets_path(&self) -> String {
        PathBuf::new()
            .join(&self.path)
            .join("originals")
            .to_string_lossy()
            .to_string()
    }
}