use derive_new::new;
use rusqlite::{Connection, OpenFlags, Result};

use crate::cocoa::parse_cocoa_timestamp;
use crate::model::asset::AssetWithAlbumInfo;
use crate::repo::asset::AssetRepository;

const HIDDEN_ALBUM_OUTPUT_DIR: &str = "_hidden";

#[derive(new)]
pub struct HiddenAssetRepository {
    db_path: String
}

impl AssetRepository for HiddenAssetRepository {
    fn get_all(&self) -> Result<Vec<AssetWithAlbumInfo>> {
        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let sql = String::from("\
            select assets.Z_PK AS ASSET_ID
                 , assets.ZDIRECTORY AS ASSET_DIRECTORY
                 , assets.ZFILENAME AS ASSET_FILENAME
                 , attribs.ZORIGINALFILENAME AS ASSET_ORIGINAL_FILENAME
                 , assets.ZDATECREATED AS ASSET_DATE
            from ZASSET assets
            left join ZADDITIONALASSETATTRIBUTES attribs on assets.Z_PK = attribs.ZASSET
            where ZHIDDEN = 1
        ");

        let mut statement = conn.prepare(sql.as_str())?;

        let iter = statement.query_map([], |row| {
            Ok(
                AssetWithAlbumInfo {
                    id: row.get(0)?,
                    dir: row.get(1)?,
                    filename: row.get(2)?,
                    original_filename: row.get(3)?,
                    date: {
                        let timestamp: f32 = row.get(4).unwrap();
                        parse_cocoa_timestamp(timestamp).date()
                    },
                    album_path: Some(HIDDEN_ALBUM_OUTPUT_DIR.to_string()),
                    album_start_date: None
                }
            )
        })?;

        Ok(iter.map(|res| res.unwrap()).collect())
    }
}