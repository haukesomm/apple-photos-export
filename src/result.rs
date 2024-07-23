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


impl<E> From<E> for PhotosExportError
where
    E: ToString + Sized,
{
    fn from(error: E) -> Self {
        PhotosExportError { messages: vec![error.to_string()] }
    }
}