use std::path::PathBuf;

use chrono::NaiveDate;

pub struct AssetWithAlbumInfo  {
    pub id: i32,
    pub dir: String,
    pub filename: String,
    pub original_filename: String,
    pub date: NaiveDate,
    pub album_path: Option<String>,
    pub album_start_date: Option<NaiveDate>
}

impl AssetWithAlbumInfo {
    pub fn get_path(&self) -> PathBuf {
        PathBuf::new()
            .join(self.dir.clone())
            .join(self.filename.clone())
    }
}