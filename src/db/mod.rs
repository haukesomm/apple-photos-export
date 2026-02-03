use std::path::Path;

pub mod album;
pub mod asset;
pub mod version;

/// Execute a closure with a database connection.
///
/// This function is a helper to open a database connection, execute a closure with the connection
/// and then close the connection again.
pub fn with_connection<P, F, R>(db_path: P, execute: F) -> crate::Result<R>
where
    P: AsRef<Path>,
    F: FnOnce(&rusqlite::Connection) -> crate::Result<R>,
{
    let conn =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let result = execute(&conn);

    conn.close()?;

    result
}
