use std::io::Cursor;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use plist::Value;
use termimad::crossterm::style::Stylize;

use crate::result::{PhotosExportError, PhotosExportResult};

use super::{connection, model::metadata::MetadataDto, schema::metadata};


const MIN_SUPPORTED: u64 = 18000;
const MAX_SUPPORTED: u64 = 18999;

fn is_supported(model_version: u64) -> bool {
    model_version >= MIN_SUPPORTED && model_version <= MAX_SUPPORTED
}


struct VersionInfo {
    pub name: &'static str,
}

fn get_version_info(model_version: u64) -> VersionInfo {
    match model_version {
        0 ..= 16999 => VersionInfo { name: "Pre macOS 14.0 Sonoma" },
        17000 ..= 17599 => VersionInfo { name: "Photos 9.0, macOS 14.0 to 14.5 Sonoma" },
        17600 ..= 17999 => VersionInfo { name: "Photos 9.0, macOS 14.6 Sonoma" },
        18000 ..= 18999 => VersionInfo { name: "Photos 10.0, macOS 15 Sequoia" },
        _ => VersionInfo { name: "Unknown" }
    }
}


pub fn check_library_version(database_path: &String) -> PhotosExportResult<()> {
    let model_number: u64 = get_library_version(database_path)?;

    let library_version = get_version_info(model_number);
    let minimum_version = get_version_info(MIN_SUPPORTED);

    if is_supported(model_number) {
        Ok(())
    } else {
        Err(
            PhotosExportError::from(
                format!(
                    "Unsupported library version!\n\
                    - Your version is: {}\n\
                    - The minimum supported version is: {}\n\
                    - See the project's README for more version information.",
                    format!("{}", library_version.name).italic(),
                    format!("{}", minimum_version.name).italic()
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