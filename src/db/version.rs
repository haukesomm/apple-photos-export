use std::io::Cursor;


/// A range of version numbers of the Photos library database.
/// 
/// The macOS Photos library database has a version number that changes with each update.
/// Each Photos version has a range of version numbers that it uses that are (mostly) compatible
/// with each other. This struct represents such a range.
/// 
/// All known version ranges are defined as constants on this struct which are generated by the
/// `version_ranges!` macro.
pub struct VersionRange {
    pub start: u64,
    pub end: u64,
    pub description: &'static str
}

/// Generates the known version ranges as constants on the `VersionRange` struct.
/// 
/// A `from_version_number` method is also generated that returns the version range for a given
/// version number.
macro_rules! version_ranges {
    ($($name:ident($start:literal, $end:literal, $desc:literal)),+) => {
        impl VersionRange {
            $(
            pub const $name:Self = Self { start: $start, end: $end, description: $desc };
            )*

            pub fn from_version_number(version: u64) -> Result<Self, String> {
                match version {
                    $($start ..= $end => Ok(Self::$name),)*
                    _ => Err(format!("Cannot determine version (unknown number): {}", version))
                }
            }
         }
    };
}

version_ranges! {
    PRE_SONOMA(0, 16999, "Older than macOS Sonoma"),
    SONOMA(17000, 17599, "Photos 9.0, macOS 14.0 to 14.5 Sonoma"),
    SONOMA_14_6(17600, 17999, "Photos 9.0, macOS 14.6 Sonoma"),
    SEQUOIA(18000, 18999, "Photos 10.0, macOS 15 Sequoia")
}

/// The currently supported version range
pub const CURRENTLY_SUPPORTED_VERSION: VersionRange = VersionRange::SEQUOIA;


/// Gets the binary encoded version plist from the Photos library database.
/// 
/// The version plist is stored in the `Z_METADATA` table of the Photos library database and, once
/// decoded, contains the version number of the database.
fn get_binary_version_plist(connection: &rusqlite::Connection) -> rusqlite::Result<Vec<u8>> {
    let mut stmt = connection.prepare("SELECT Z_PLIST from Z_METADATA ORDER BY Z_VERSION DESC")?;

    stmt.query_row([], |row| row.get::<usize, Vec<u8>>(0))
}

/// Gets the version number of the Photos library database.
/// 
/// Based on this version number, the version range of the database can be determined using the
/// `VersionRange` struct.
pub fn get_version_number(connection: &rusqlite::Connection) -> crate::Result<u64> {
    let binary_version_plist = get_binary_version_plist(connection).map_err(|e| {
        format!(
            "Unable to get version plist from database: {}",
            e.to_string()
        )
    })?;

    let cursor = Cursor::new(binary_version_plist);

    let version = plist::Value::from_reader(cursor)
        .map_err(|e| format!("Unable to parse binary version plist: {}", e.to_string()))?
        .as_dictionary()
        .and_then(|dict| dict.get("PLModelVersion"))
        .and_then(|version| version.as_unsigned_integer())
        .ok_or("Unable to read model version from plist")?;

    Ok(version)
}