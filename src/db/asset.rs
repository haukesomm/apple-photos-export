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

fn get_exportable_assets_query(album_z_ent_key: i32, asset_z_ent_key: i32) -> String {
    format!(
        r"
        SELECT asset.Z_PK                           AS ID,
               asset.ZUUID                          AS UUID,
               asset.ZDIRECTORY                     AS DIR,
               asset.ZFILENAME                      AS FILENAME,
               asset.ZUNIFORMTYPEIDENTIFIER         AS UTI,
               asset.ZDATECREATED                   AS DATETIME,
               asset.ZHIDDEN                        AS HIDDEN,
               asset.ZTRASHEDSTATE                  AS TRASHED,
               asset.ZVISIBILITYSTATE               AS VISIBLE,
               asset.ZDUPLICATEASSETVISIBILITYSTATE AS DUPLICATE_VISIBILITY,
               asset.ZADJUSTMENTSSTATE > 0          AS HAS_ADJUSTMENTS,
               asset_attribs.ZORIGINALFILENAME      AS ORIGINAL_FILENAME,
               int_res.ZCOMPACTUTI                  AS COMPACT_UTI,
               GROUP_CONCAT(album.Z_PK, ', ')       AS ALBUM_IDS
        FROM ZASSET asset
                 INNER JOIN ZADDITIONALASSETATTRIBUTES asset_attribs
                            ON asset.Z_PK = asset_attribs.ZASSET
                 LEFT JOIN ZINTERNALRESOURCE int_res
                           ON int_res.ZASSET = asset_attribs.ZASSET
                               AND int_res.ZDATASTORESUBTYPE = 1
                 LEFT JOIN Z_{}ASSETS album_mapping
                           ON album_mapping.Z_{}ASSETS = asset.Z_PK
                 LEFT JOIN ZGENERICALBUM album
                           ON album_mapping.Z_{}ALBUMS = album.Z_PK
        WHERE asset.ZTRASHEDSTATE = false
          AND asset.ZVISIBILITYSTATE = 0
          AND asset.ZDUPLICATEASSETVISIBILITYSTATE = 0
          -- Field may not be filled in the database, depending on whether it is an iCloud-enabled or
          -- offline library
          AND (int_res.ZLOCALAVAILABILITY = 1 OR int_res.ZLOCALAVAILABILITY IS NULL)
          -- Album kind values:
          -- - 3999: Root album
          -- - 4000: User-created folder
          -- - 2: User-created album
          AND (album.ZKIND IS NULL OR (album.ZTRASHEDSTATE = false AND album.ZKIND IN (3999, 4000, 2)))
        GROUP BY asset.Z_PK
        ORDER BY asset.Z_PK
        ",
        album_z_ent_key, asset_z_ent_key, album_z_ent_key
    )
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
    let raw_sql = get_exportable_assets_query(album_z_ent, asset_z_ent);

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
