use std::path::PathBuf;

use chrono::NaiveDateTime;

use crate::db::repo::asset::ExportAssetDto;
use crate::foundation::cocoa;
use crate::model::album::Album;
use crate::model::FromDbModel;
use crate::model::uti::Uti;

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

impl FromDbModel<ExportAssetDto> for ExportAsset {
    fn from_db_model(model: &ExportAssetDto) -> Result<Self, String> {
        Ok(ExportAsset {
            id: model.id,
            uuid: model.uuid.clone(),
            dir: model.dir.clone(),
            filename: model.filename.clone(),
            original_uti: match model.compact_uti {
                // First one is a fallback for offline libraries as the compact uti is not available
                // in that case. It should work but is not as accurate as the second one.
                None => Uti::from_filename(&model.filename),
                Some(uti) => Uti::from_compact(uti)
            }?,
            derivate_uti: Uti::from_name(model.uniform_type_identifier.as_str())?,
            datetime: cocoa::parse_cocoa_timestamp(model.timestamp)?,
            favorite: model.favorite,
            hidden: model.hidden,
            original_filename: model.original_filename.clone(),
            has_adjustments: model.has_adjustments,
            album: match &model.album {
                None => None,
                Some(a) => Some(Album::from_db_model(a)?),
            }
        })
    }
}