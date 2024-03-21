use derive_new::new;
use rusqlite::Result;

use crate::model::asset::AssetWithAlbumInfo;
use crate::repo::asset::AssetRepository;

#[derive(new)]
pub struct CombiningAssetRepository {
    asset_repos: Vec<Box<dyn AssetRepository>>,
}

impl AssetRepository for CombiningAssetRepository {
    fn get_all(&self) -> Result<Vec<AssetWithAlbumInfo>> {
        let mut assets = Vec::new();
        for repo in &self.asset_repos {
            assets.extend(repo.get_all()?);
        }
        Ok(assets)
    }
}