use diesel::{Associations, Identifiable, Queryable, Selectable};

use crate::db::model::album::Album;

#[derive(Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::db::schema::assets)]
pub struct Asset  {
    pub id: i32,
    pub dir: String,
    pub filename: String,
    pub date: f32,
    pub hidden: bool,
    pub favorite: bool,
    pub trashed: bool,
    pub visibility_state: i32,
    pub cloud_local_state: bool,
}

#[derive(Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = crate::db::schema::asset_attributes)]
#[diesel(belongs_to(Asset))]
pub struct AssetAttributes {
    pub id: i32,
    pub asset_id: i32,
    pub original_filename: String,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::db::schema::album_assets)]
#[diesel(belongs_to(Asset), belongs_to(Album))]
pub struct AlbumAsset {
    pub asset_id: i32,
    pub album_id: i32,
}