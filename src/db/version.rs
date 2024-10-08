use std::io::Cursor;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use plist::Value;
use termimad::crossterm::style::Stylize;

use crate::result::{PhotosExportError, PhotosExportResult};

use super::{connection, model::metadata::MetadataDto, schema::metadata};


#[allow(dead_code)]
mod ranges {
    pub struct Range {
        pub min: u64,
        pub max: u64,
        pub desc: &'static str
    }

    pub const CURRENT_SUPPORTED: Range = PHOTOS_9_MACOS_14_6;

    pub const PHOTOS_9: Range = Range { min: 17000, max: 17599, desc: "Photos 9.0, macOS 14.0 to 14.5 Sonoma" };
    pub const PHOTOS_9_MACOS_14_6: Range = Range { min: 17600, max: 17999, desc: "Photos 9.0, macOS 14.6 Sonoma" };
}


pub fn check_library_version(database_path: &String) -> PhotosExportResult<()> {
    let model_version: u64 = get_library_version(database_path)?;

    let min = ranges::CURRENT_SUPPORTED.min;
    let max = ranges::CURRENT_SUPPORTED.max;

    if model_version >= min && model_version <= max {
        Ok(())
    } else {
        Err(
            PhotosExportError::from(
                format!(
                    "Unsupported library version!\n\
                    - Your version is: {}\n\
                    - The currently supported library format is: {} (versions {} to {})\n\
                    - See the project's README for more version information.",
                    model_version,
                    format!("{}", ranges::CURRENT_SUPPORTED.desc).italic(),
                    min,
                    max,
                )
            )
        )
    }
}

fn get_library_version(database_path: &String) -> Result<u64, String> {
    let mut conn = connection::establish_connection(database_path);

    let result = metadata::table
        .select(MetadataDto::as_select())
        .order_by(metadata::version.desc())
        .first(&mut conn)
        .map_err(|_| "Unable to query metadata table")?;

    let cursor = Cursor::new(result.plist);

    let version = Value::from_reader(cursor)
        .map_err(|e| format!("Unable to parse binary version plist: {}", e.to_string()))?
        .as_dictionary()
        .and_then(|dict| dict.get("PLModelVersion"))
        .and_then(|version| version.as_unsigned_integer())
        .ok_or("Unable to read model version from plist")?;

    Ok(version)
}