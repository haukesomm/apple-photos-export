use chrono::NaiveDateTime;
use crate::foundation::{ParseCocoaTimestamp, Uti};
use crate::model::Asset;

/// Get the count of all assets in the database that are _visible_, meaning they are not
/// part of the "hidden" album or moved to the trash.
pub fn get_visible_count(conn: &rusqlite::Connection) -> crate::Result<usize> {
    let raw_sql = include_str!("../../queries/count_visible_assets.sql");
    let mut stmt = conn.prepare(raw_sql)?;
    Ok(stmt.query_row([], |row| row.get(0))?)
}

/// Get the count of all assets in the database that are _exportable_, meaning they are not
/// part of the "hidden" album or moved to the trash, and are locally available in the library 
/// file.
pub fn get_exportable_assets(conn: &rusqlite::Connection) -> crate::Result<Vec<Asset>> {
    let raw_sql = include_str!("../../queries/get_exportable_assets.sql");
    let mut stmt = conn.prepare(raw_sql)?;
    
    let assets: crate::Result<Vec<Asset>> = stmt.query_and_then([], |row| {
        Ok(
            Asset {
                uuid: row.get("UUID")?,
                dir: row.get("DIR")?,
                filename: row.get("FILENAME")?,
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