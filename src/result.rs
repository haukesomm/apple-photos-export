/// App specific error type representing different kinds of errors that can occur while using
/// the application.
pub enum Error {
    
    /// A general error occurred.
    /// 
    /// This type is used for errors that do not fit into any of the other categories.
    General(String),
    
    /// An error occurred while reading the database.
    /// 
    /// This type is used for `rusqlite` errors.
    Database(String),
    
    /// An error occurred during the export process.
    /// 
    /// This type is used for errors that occur during the export process, e.g. when copying files
    /// or creating directories.
    /// 
    /// It contains a list of error messages for each failed export.
    Export(Vec<String>),
}

/// Type alias for a result that can return the app-internal `Error` type defined in the `result` 
/// module.
pub type Result<T> = std::result::Result<T, Error>;


/// Internal marker trait to allow conversion of different types to the `Error` type.
/// 
/// This marker is needed so that the `From` trait does not clash with Display implementation of the
/// `Error` type.
trait ToErrorFromString {}

impl ToErrorFromString for &str {}
impl ToErrorFromString for String {}
impl ToErrorFromString for std::fmt::Error {}


impl<S: ToString + ToErrorFromString> From<S> for Error {
    fn from(value: S) -> Self {
        Self::General(value.to_string())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<(rusqlite::Connection, rusqlite::Error)> for Error {
    fn from(value: (rusqlite::Connection, rusqlite::Error)) -> Self {
        Self::Database(value.1.to_string())
    }
}