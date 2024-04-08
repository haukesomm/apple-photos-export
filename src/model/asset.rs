use std::path::PathBuf;

use chrono::NaiveDateTime;

use crate::db::repo::exportable_assets::ExportableAssetInfo;
use crate::foundation::cocoa;
use crate::model::album::Album;
use crate::model::FromDbModel;

pub struct ExportAsset {
    pub id: i32,
    pub dir: String,
    pub filename: String,
    pub date: NaiveDateTime,
    pub favorite: bool,
    pub original_filename: String,
    pub album: Option<Album>,
}

impl ExportAsset {
    pub fn path(&self) -> PathBuf {
        PathBuf::new()
            .join(&self.dir)
            .join(&self.filename)
    }
}

impl FromDbModel<ExportableAssetInfo> for ExportAsset {
    fn from_db_model(model: ExportableAssetInfo) -> Result<Self, String> {
        let (asset, additional_attribs, album) = model;

        Ok(ExportAsset {
            id: asset.id,
            dir: asset.dir,
            filename: asset.filename,
            date: cocoa::parse_cocoa_timestamp(asset.date)?,
            favorite: asset.favorite,
            original_filename: additional_attribs.original_filename,
            album: match album {
                None => None,
                Some(a) => Some(Album::from_db_model(a)?),
            }
        })
    }
}