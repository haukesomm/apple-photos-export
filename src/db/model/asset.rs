use diesel::{Identifiable, Queryable, Selectable};

use crate::db::repo::asset::ExportAssetDto;
use crate::foundation::cocoa;
use crate::model::asset::ExportAsset;
use crate::model::FromDbModel;
use crate::model::uti::Uti;

#[derive(Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::db::schema::assets)]
pub struct AssetDto {
    pub id: i32,
    pub uuid: String,
    pub dir: String,
    pub filename: String,
    pub uniform_type_identifier: String,
    pub date: f32,
    pub hidden: bool,
    pub favorite: bool,
    pub trashed: bool,
    pub visibility_state: i32,
    pub duplicate_asset_visibility_state: i32,
    pub adjustments_state: i32,
}

#[derive(Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::db::schema::asset_attributes)]
pub struct AssetAttributesDto {
    pub id: i32,
    pub asset_id: i32,
    pub original_filename: String,
}

#[allow(dead_code)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::album_assets)]
pub struct AlbumAssetDto {
    pub asset_id: i32,
    pub album_id: i32,
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
                Some(a) => Some(crate::model::album::Album::from_db_model(a)?),
            }
        })
    }
}