/// Extracts the file extension from a given string.
/// 
/// This trait is used purely as a utility to extract the file extension from a given string.
/// The same could also be achieved by using the `std::path::Path` API directly.
pub trait ExtractFileExtension {
    
    /// Extracts the file extension from the given string.
    /// 
    /// If the string does not end with a file extension, an error is returned.
    fn file_extension(&self) -> Result<String, String>;
}

impl ExtractFileExtension for &str {
    fn file_extension(&self) -> Result<String, String> {
        std::path::Path::new(&self)
            .extension()
            .map(|os_str| String::from(os_str.to_string_lossy()))
            .ok_or(format!("The given String does not end with a file extension: {}", &self))
    }
}
