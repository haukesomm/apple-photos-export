use crate::model::Asset;
use std::path::PathBuf;

/// Represents a macOS Photos library.
/// 
/// Once initialized with the path of the library on disk, it can be used to compute asset paths and
/// similar information.
pub struct Library {
    pub path: PathBuf,
}

impl Library {
    
    /// Create a new library instance with the given path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    
    /// Returns the absolute path of the internal SQLite database of the library.
    pub fn db_path(&self) -> PathBuf {
        PathBuf::new()
            .join(&self.path)
            .join("database/Photos.sqlite")
    }

    /// Returns the absolute path to the original asset file.
    pub fn get_asset_original_path(&self, asset: &Asset) -> PathBuf {
        PathBuf::new()
            .join(&self.path)
            .join("originals")
            .join(&asset.dir)
            .join(&asset.filename)
    }
    
    /// Returns the absolute path to the derivate asset file.
    /// 
    /// If the asset has no adjustments, this method returns `None`.
    pub fn get_asset_derivate_path(&self, asset: &Asset) -> Option<PathBuf> {
        if !asset.has_adjustments {
            return None;
        }

        let derivate_filename = {
            let uti = &asset.derivate_uti;
            format!(
                "{}{}.{}",
                asset.uuid,
                uti.derivate_suffix,
                uti.ext
            )
        };

        let derivate_path = PathBuf::new()
            .join(&self.path)
            .join("resources/renders")
            .join(&asset.dir)
            .join(&derivate_filename);

        Some(derivate_path)
    }
}
