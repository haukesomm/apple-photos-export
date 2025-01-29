use chrono::NaiveDateTime;
use clap::builder::TypedValueParser;
use rusqlite::fallible_iterator::FallibleIterator;
use crate::foundation::ParseCocoaTimestamp;
use crate::model::album::{Album, AlbumKind};

/// Queries the database for all albums and returns them as a vector.
pub fn get_all_albums(conn: &rusqlite::Connection) -> crate::Result<Vec<Album>> {
    let mut stmt = conn.prepare("\
        SELECT a.Z_PK
             , a.ZKIND
             , a.ZTITLE
             , a.ZPARENTFOLDER
             , a.ZSTARTDATE
        FROM ZGENERICALBUM a
        WHERE a.ZKIND IN (3999, 4000, 2)
          AND a.ZTRASHEDSTATE = FALSE
        ORDER BY a.ZSTARTDATE; 
    ")?;

    let albums: crate::Result<Vec<Album>> = stmt.query_and_then([], |row| {
        Ok(
            Album {
                id: row.get(0)?,
                kind: { 
                    let id = row.get(1)?;
                    AlbumKind::by_value(id).ok_or(format!("Unknown album kind: {}", id))?
                },
                name: row.get(2)?,
                parent_id: row.get(3)?,
                start_date: {
                    let timestamp: Option<f32> = row.get(4)?;
                    timestamp.map(|t| NaiveDateTime::from_cocoa_timestamp(t)).transpose()?
                },
            }
        )
    })?.collect();
    
    albums
}
