pub mod default;
pub mod combining;
pub mod hidden;

use rusqlite::Result;
use crate::model::asset::AssetWithAlbumInfo;

pub trait AssetRepository {
    fn get_all(&self) -> Result<Vec<AssetWithAlbumInfo>>;
}