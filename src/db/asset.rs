use chrono::NaiveDateTime;
use crate::foundation::{ParseCocoaTimestamp, Uti};
use crate::model::Asset;

pub fn get_visible_count(conn: &rusqlite::Connection) -> crate::Result<usize> {
    let raw_sql = include_str!("../../queries/count_visible_assets.sql");
    let mut stmt = conn.prepare(raw_sql)?;
    Ok(stmt.query_row([], |row| row.get(0))?)
}

pub fn get_exportable_assets(conn: &rusqlite::Connection) -> crate::Result<Vec<Asset>> {
    let raw_sql = include_str!("../../queries/get_exportable_assets.sql");
    let mut stmt = conn.prepare(raw_sql)?;
    
    let assets: crate::Result<Vec<Asset>> = stmt.query_and_then([], |row| {
        Ok(
            Asset {
                id: row.get("ID")?,
                uuid: row.get("UUID")?,
                dir: row.get("DIR")?,
                filename: row.get("FILENAME")?,
                original_uti: {
                    let uti: Option<String> = row.get("COMPACT_UTI")?;
                    let filename: String = row.get("FILENAME")?;
                    match uti {
                        Some(id) => Uti::from_cid_and_filename(id.as_str(), filename.as_str()),
                        // Fallback for offline libraries as the compact uti is not available
                        // in that case. It should work but is not as accurate as the second one.
                        None => Uti::from_filename(filename.as_str()),
                    }?
                },
                derivate_uti: { 
                    let uti_name: String = row.get("UTI")?;
                    Uti::from_id(uti_name.as_str())?
                },
                datetime: NaiveDateTime::from_cocoa_timestamp(row.get("DATETIME")?)?,
                hidden: row.get("HIDDEN")?,
                original_filename: row.get("ORIGINAL_FILENAME")?,
                has_adjustments: row.get("HAS_ADJUSTMENTS")?,
                album_ids: {
                    let serialized_ids: Option<String> = row.get("ALBUM_IDS")?;
                    serialized_ids
                        .map(|string| {
                            string
                                .split(", ")
                                .map(|id| id.parse::<i32>())
                                .collect::<Result<Vec<i32>, _>>()
                                .ok()
                        })
                        .flatten()
                        .unwrap_or(vec![])
                }
            }
        )
    })?.collect();
    
    assets
}