use crate::model::asset::Asset;
use crate::uti::FileType;
use std::path::PathBuf;

/// Represents a macOS Photos library.
///
/// Once initialized with the path of the library on disk, it can be used to compute asset paths and
/// similar information.
#[derive(Clone)]
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
            let suffix = match uti.file_type {
                FileType::Image => file_suffixes::IMAGE_DERIVATE,
                FileType::Video => file_suffixes::VIDEO_DERIVATE,
            };
            format!("{}{}.{}", asset.uuid, suffix, uti.ext)
        };

        let derivate_path = PathBuf::new()
            .join(&self.path)
            .join("resources/renders")
            .join(&asset.dir)
            .join(&derivate_filename);

        Some(derivate_path)
    }
}

pub mod file_suffixes {
    //! Known suffixes added to asset (image/video) files depending on their characteristics.

    /// Suffix appended to all derived _image_ assets when stored in the Photos library.
    pub const IMAGE_DERIVATE: &str = "_1_201_a";

    /// Suffix appended to all derived _video_ assets when stored in the Photos library.
    pub const VIDEO_DERIVATE: &str = "_2_0_a";
}
