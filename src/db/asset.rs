use crate::cocoa_time::ParseCocoaTimestamp;
use crate::model::Asset;
use crate::uti::Uti;
use chrono::NaiveDateTime;

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
    // Get the Z_ENT value for Asset entity
    let asset_z_ent_sql = include_str!("../../queries/get_asset_z_ent.sql");
    let asset_z_ent: i32 = conn.query_row(asset_z_ent_sql, [], |row| row.get(0))?;

    // Get the Z_ENT value for Album entity
    let album_z_ent_sql = include_str!("../../queries/get_album_z_ent.sql");
    let album_z_ent: i32 = conn.query_row(album_z_ent_sql, [], |row| row.get(0))?;

    // Load the SQL template and replace the placeholders with actual Z_ENT values
    let raw_sql_template = include_str!("../../queries/get_exportable_assets.sql");
    let raw_sql = raw_sql_template
        .replace("_ALBUM_Z_ENT_", &album_z_ent.to_string())
        .replace("_ASSET_Z_ENT_", &asset_z_ent.to_string());

    let mut stmt = conn.prepare(&raw_sql)?;

    let assets: crate::Result<Vec<Asset>> = stmt
        .query_and_then([], |row| {
            Ok(Asset {
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
                },
            })
        })?
        .collect();

    assets
}
