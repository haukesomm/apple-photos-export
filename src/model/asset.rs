use std::path::PathBuf;

use chrono::NaiveDateTime;

use crate::model::album::Album;
use crate::model::uti::Uti;

#[allow(dead_code)]
pub struct ExportAsset {
    pub id: i32,
    pub uuid: String,
    pub dir: String,
    pub filename: String,
    pub original_uti: &'static Uti,
    /// Note: Same as original_uti if no adjustments
    pub derivate_uti: &'static Uti,
    pub datetime: NaiveDateTime,
    pub favorite: bool,
    pub hidden: bool,
    pub original_filename: String,
    pub has_adjustments: bool,
    pub album: Option<Album>,
}

impl ExportAsset {

    pub fn get_path(&self) -> PathBuf {
        PathBuf::new()
            .join("originals")
            .join(&self.dir)
            .join(&self.filename)
    }

    pub fn get_derivate_path(&self) -> Option<PathBuf> {
        if !self.has_adjustments {
            return None
        }

        let derivate_file_type = self.derivate_uti;
        let derivate_filename = format!(
            "{}{}.{}",
            self.uuid,
            derivate_file_type.uuid_suffix,
            derivate_file_type.extension
        );

        let derivate_path = PathBuf::new()
            .join("resources")
            .join("renders")
            .join(&self.dir)
            .join(&derivate_filename);

        Some(derivate_path)
    }
}