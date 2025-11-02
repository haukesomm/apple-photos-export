use crate::cocoa_time::ParseCocoaTimestamp;
use crate::model::album::Album;
use chrono::NaiveDateTime;

/// Queries the database for all albums and returns them as a vector.
pub fn get_all_albums(conn: &rusqlite::Connection) -> crate::Result<Vec<Album>> {
    let raw_sql = include_str!("../../queries/get_albums.sql");
    let mut stmt = conn.prepare(raw_sql)?;

    let albums: crate::Result<Vec<Album>> = stmt
        .query_and_then([], |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                start_date: {
                    let timestamp: Option<f32> = row.get(3)?;
                    timestamp
                        .map(|t| NaiveDateTime::from_cocoa_timestamp(t))
                        .transpose()?
                },
            })
        })?
        .collect();

    albums
}
