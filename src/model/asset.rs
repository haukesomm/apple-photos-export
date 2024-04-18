use std::path::PathBuf;
use chrono::NaiveDateTime;
use crate::db::repo::asset::ExportableAsset;
use crate::foundation::cocoa;
use crate::model::album::Album;
use crate::model::uti::Uti;
use crate::model::FromDbModel;

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

impl FromDbModel<ExportableAsset> for ExportAsset {
    fn from_db_model(model: ExportableAsset) -> Result<Self, String> {
        let (asset, additional_attribs, internal_res, album) = model;

        Ok(ExportAsset {
            id: asset.id,
            uuid: asset.uuid,
            dir: asset.dir,
            filename: asset.filename,
            original_uti: Uti::from_compact(internal_res.compact_uti)?,
            derivate_uti: Uti::from_name(asset.uniform_type_identifier.as_str())?,
            datetime: cocoa::parse_cocoa_timestamp(asset.date)?,
            favorite: asset.favorite,
            hidden: asset.hidden,
            original_filename: additional_attribs.original_filename,
            has_adjustments: asset.has_adjustments,
            album: match album {
                None => None,
                Some(a) => Some(Album::from_db_model(a)?),
            }
        })
    }
}