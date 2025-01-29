pub mod album;
mod version;

use std::path::Path;
pub use version::{VersionRange, get_version_number};
pub use album::get_all_albums;

/// Execute a closure with a database connection.
/// 
/// This function is a helper to open a database connection, execute a closure with the connection
/// and then close the connection again.
pub fn with_connection<P, F, R, E>(db_path: P, execute: F) -> Result<R, String>
where
    P: AsRef<Path>,
    F: FnOnce(&rusqlite::Connection) -> Result<R, E>,
    E: ToString,
{
    let conn = rusqlite::Connection::open_with_flags(
        db_path, 
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
    ).map_err(|e| format!("Error connecting to database: {}", e.to_string()))?;
    
    let result = execute(&conn)
        .map_err(|e| format!("Error executing db query closure: {}", e.to_string()));
    
    conn.close()
        .map_err(|(_, e)| format!("Error closing database-connection: {}", e.to_string()))?;
    
    result
}