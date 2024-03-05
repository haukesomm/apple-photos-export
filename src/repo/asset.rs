use rusqlite::{Connection, OpenFlags, params_from_iter, Result};

use crate::model::asset::AssetWithAlbumInfo;
use crate::model::cocoa_date::parse_cocoa_timestamp;

pub enum FilterMode {
    None,
    IncludeAlbumIds(Vec<i32>),
    ExcludeAlbumIds(Vec<i32>)
}


pub trait AssetWithAlbumInfoRepo {
    fn get_all(&self) -> Result<Vec<AssetWithAlbumInfo>>;
}


pub struct AssetWithAlbumInfoRepoImpl<'a> {
    db_path: &'a String,
    filter_mode: FilterMode
}

impl AssetWithAlbumInfoRepoImpl<'_> {
    pub fn new(db_path: &String, filter_mode: FilterMode) -> AssetWithAlbumInfoRepoImpl {
        AssetWithAlbumInfoRepoImpl { db_path, filter_mode }
    }
}

impl AssetWithAlbumInfoRepo for AssetWithAlbumInfoRepoImpl<'_> {
    fn get_all(&self) -> Result<Vec<AssetWithAlbumInfo>> {
        let conn = Connection::open_with_flags(self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut sql = String::from("\
            WITH RECURSIVE ALBUM_PATH_CTE AS (
                SELECT Z_PK
                     , ZPARENTFOLDER
                     , '' AS path
                FROM ZGENERICALBUM
                WHERE ZGENERICALBUM.ZPARENTFOLDER IS NULL

            UNION ALL

                SELECT child.Z_PK
                     , child.ZPARENTFOLDER
                     , printf('%s%s/', album.path, child.ZTITLE) AS path
                FROM ZGENERICALBUM child
                JOIN ALBUM_PATH_CTE album
                  ON album.Z_PK = child.ZPARENTFOLDER
                WHERE child.ZTRASHEDSTATE = 0
            )

            SELECT assets.Z_PK AS ASSET_ID
                 , assets.ZDIRECTORY AS ASSET_DIRECTORY
                 , assets.ZFILENAME AS ASSET_FILENAME
                 , attribs.ZORIGINALFILENAME AS ASSET_ORIGINAL_FILENAME
                 , assets.ZDATECREATED AS ASSET_DATE
                 , album_path.path AS ALBUM_PATH
                 , album.ZSTARTDATE AS ALBUM_START_DATE
            FROM ZASSET assets
            LEFT JOIN ZADDITIONALASSETATTRIBUTES attribs ON assets.Z_PK = attribs.ZASSET
            LEFT JOIN Z_28ASSETS album_mapping ON assets.Z_PK = album_mapping.Z_3ASSETS
            LEFT JOIN ZGENERICALBUM album ON album_mapping.Z_28ALBUMS = album.Z_PK
            LEFT JOIN ALBUM_PATH_CTE album_path ON album.Z_PK = album_path.Z_PK
            WHERE (album.ZKIND IS NULL OR album.ZKIND IN (2, 3999, 4000))"
        );

        let (filter_clause, ids) = match &self.filter_mode {
            FilterMode::None => (String::new(), Vec::<i32>::new()),
            FilterMode::IncludeAlbumIds(ids) => (format!(
                "AND album.Z_PK IS NOT NULL AND album.Z_PK IN ({})",
                repeat_vars(ids.len())
            ), ids.clone()),
            FilterMode::ExcludeAlbumIds(ids) => (format!(
                "AND (album.Z_PK IS NULL OR album.Z_PK NOT IN ({}))",
                repeat_vars(ids.len())
            ), ids.clone())
        };

        sql.push('\n');
        sql.push_str(filter_clause.as_str());

        let mut statement = conn.prepare(sql.as_str())?;

        let iter = statement.query_map(params_from_iter(ids.iter()), |row| {
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
                    album_path: row.get(5)?,
                    album_start_date: {
                        let timestamp: Option<f32> = row.get(6).unwrap();
                        match timestamp {
                            None => None,
                            Some(t) => Some(parse_cocoa_timestamp(t).date())
                        }
                    }
                }
            )
        })?;

        Ok(iter.map(|res| res.unwrap()).collect())
    }
}


fn repeat_vars(count: usize) -> String {
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}