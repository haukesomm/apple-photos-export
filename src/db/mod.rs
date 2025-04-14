pub mod album;
mod version;
pub mod asset;

use std::path::Path;
pub use version::{VersionRange, get_version_number, CURRENTLY_SUPPORTED_VERSION};
pub use album::get_all_albums;
pub use asset::get_visible_count;
pub use asset::get_exportable_assets;

/// Execute a closure with a database connection.
/// 
/// This function is a helper to open a database connection, execute a closure with the connection
/// and then close the connection again.
pub fn with_connection<P, F, R>(db_path: P, execute: F) -> crate::Result<R>
where
    P: AsRef<Path>,
    F: FnOnce(&rusqlite::Connection) -> crate::Result<R>,
{
    let conn = rusqlite::Connection::open_with_flags(
        db_path, 
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
    )?;
    
    let result = execute(&conn);
    
    conn.close()?;
    
    result
}