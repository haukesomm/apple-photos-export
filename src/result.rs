#[derive(Debug)]
pub struct PhotosExportError {
    pub messages: Vec<String>,
}

impl PhotosExportError {
    pub fn empty() -> Self {
        PhotosExportError { messages: vec![] }
    }
}


pub type PhotosExportResult<T> = Result<T, PhotosExportError>;


impl From<String> for PhotosExportError {
    fn from(message: String) -> Self {
        PhotosExportError { messages: vec![message] }
    }
}

impl From<std::io::Error> for PhotosExportError {
    fn from(error: std::io::Error) -> Self {
        PhotosExportError { messages: vec![error.to_string()] }
    }
}

impl From<diesel::result::Error> for PhotosExportError {
    fn from(error: diesel::result::Error) -> Self {
        PhotosExportError { messages: vec![error.to_string()] }
    }
}

impl From<termimad::Error> for PhotosExportError {
    fn from(error: termimad::Error) -> Self {
        PhotosExportError { messages: vec![error.to_string()] }
    }
}