use std::fmt::Display;
use std::io::Write;


/// App specific error type representing different kinds of errors that can occur while using
/// the application.
pub enum Error {
    
    /// A general error occurred.
    /// 
    /// This type is used for errors that do not fit into any of the other categories.
    General(String),
    
    /// An error occurred during the export process.
    /// 
    /// This type is used for errors that occur during the export process, e.g. when copying files
    /// or creating directories.
    /// 
    /// It contains a list of tuples with the source of the asset that caused the
    /// error and a description of the error.
    // TODO Include copy of the actual export asset
    Export(Vec<(String, String)>),
}

/// Type alias for a result that can return the app-internal `Error` type defined in the `result` 
/// module.
pub type Result<T> = std::result::Result<T, Error>;


/// Internal marker trait to allow conversion of different types to the `Error` type.
/// 
/// This marker is needed so that the `From` trait does not clash with Display implementation of the
/// `Error` type.
trait ToError {}

impl ToError for &str {}
impl ToError for String {}
impl ToError for rusqlite::Error {}
impl ToError for std::fmt::Error {}


impl<S: ToString + ToError> From<S> for Error {
    fn from(value: S) -> Self {
        Self::General(value.to_string())
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::General(msg) => write!(f, "A general error occurred: {}", msg),
            // TODO Format as table with source, destination and error description
            Error::Export(_) => unimplemented!()
        }
    }
}