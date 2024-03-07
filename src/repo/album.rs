use rusqlite::{Connection, OpenFlags, Result};

use crate::model::album::{Album, Kind};
use crate::model::cocoa_date::parse_cocoa_timestamp;

pub struct AlbumRepository {
    db_path: String
}

impl AlbumRepository {

    pub fn new(db_path: String) -> AlbumRepository {
        AlbumRepository { db_path }
    }

    pub fn get_all(&self) -> Result<Vec<Album>> {
        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut statement = conn.prepare("\
        SELECT album.Z_PK
             , album.ZKIND
             , album.ZTITLE
             , album.ZSTARTDATE
             , album.ZPARENTFOLDER
             , (
                    SELECT COUNT(*)
                    FROM Z_28ASSETS mapping
                    WHERE mapping.Z_28ALBUMS = album.Z_PK
               ) AS ASSET_COUNT
        FROM ZGENERICALBUM album
        WHERE album.ZKIND IN (2, 3999, 4000) AND album.ZTRASHEDSTATE = 0
        ORDER BY album.ZSTARTDATE;
    ")?;

        let album_iter = statement.query_map([], |row| {
            Ok(
                Album {
                    id: row.get(0)?,
                    kind: {
                        let id: i32 = row.get(1).unwrap();
                        Kind::try_from(id).unwrap()
                    },
                    parent_id: row.get(4)?,
                    name: row.get(2).unwrap_or("".to_string()),
                    start_date: {
                        let cocoa_seconds: Option<f32> = row.get(3).unwrap();
                        match cocoa_seconds {
                            None => None,
                            Some(f) => Some(parse_cocoa_timestamp(f))
                        }
                    },
                    asset_count: row.get(5)?,
                }
            )
        })?;

        Ok(album_iter.map(|res| res.unwrap()).collect())
    }
}