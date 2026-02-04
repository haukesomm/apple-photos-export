/// Provides enum variants to distinguish between UTIs for image and video files.
#[derive(Clone)]
pub enum FileType {
    Image,
    Video,
}

/// Represents a
/// [uniform type identifier](https://developer.apple.com/documentation/uniformtypeidentifiers) that
/// is used to identify the file type of an asset.
///
/// This struct stores the identifier of the UTI, a compact version of the identifier, the file
/// extension associated with the UTI, and a suffix that is appended to all derived assets of that
/// type when stored in the Photos library (e.g. edited photos).
///
/// All known UTIs are stored as constants on the `Uti` struct. While the respective API could be
/// used to handle UTIs, it is not available on other platforms than Apple's operating systems.
///
/// Additionally, methods to determine the UTI from a file extension or an identifier are provided
/// as struct-level methods.
#[derive(Clone)]
pub struct Uti {
    /// File extension associated with the UTI.
    pub ext: &'static str,

    pub file_type: FileType,
}

/// Macro used to define the known UTIs as constants on the `Uti` struct.
///
/// While the respective constants could be defined manually, this macro also generates the getter
/// methods used to determine the UTI from a file extension or an identifier.
///This way, it is not necessary to manually define the getter methods for each UTI and keep them in
/// sync with the constants.
macro_rules! uti_constants {
    ($($name:ident($id:expr, $compact_id:expr, $ext:expr, $filetype:expr)),+) => {
        impl Uti {
            $(
            pub const $name:Self = Self { ext: $ext, file_type: $filetype };
            )*

            /// Determines the UTI from the given identifier.
            ///
            /// > **Note**: The identifier is _not_ the compact identifier, but the full identifier
            /// > of the UTI!
            pub fn from_id(id: &str) -> Result<Self, String> {
                match id {
                    $($id => Ok(Self::$name),)*
                    _ => Err(format!("Unknown UTI id: {}", id))
                }
            }

            /// Determines the UTI from the given _compact_ identifier.
            ///
            /// > **Note**: The identifier _is_ the compact identifier, _and not_ the full
            /// > identifier of the UTI!
            pub fn from_compact_id(id: &str) -> Result<Self, String> {
                match id {
                    $($compact_id => Ok(Self::$name),)*
                    _ => Err(format!("Unknown compact UTI id: {}", id))
                }
            }
         }
    };
}

uti_constants! {
    // Image formats
    BMP("com.microsoft.bmp", "_com.microsoft.bmp", "bmp", FileType::Image),
    GIF("com.compuserve.gif", "7", "gif", FileType::Image),
    HEIC("public.heic", "3", "heic", FileType::Image),
    JPEG("public.jpeg", "1", "jpeg", FileType::Image),
    PNG("public.png", "6", "png", FileType::Image),
    PSD("com.adobe.photoshop-image", "_com.adobe.photoshop-image", "psd", FileType::Image),
    TIFF("public.tiff", "8", "tiff", FileType::Image),
    WEBP("org.webmproject.webp", "_org.webmproject.webp", "webp", FileType::Image),

    // Raw image formats
    ARW("com.sony.arw-raw-image", "10", "arw", FileType::Image),
    CR2("com.canon.cr2-raw-image", "11", "cr2", FileType::Image),
    DNG("com.adobe.raw-image", "9", "dng", FileType::Image),
    RAF("com.fuji.raw-image", "21", "raf", FileType::Image),

    // Movie formats
    GPP("public.3gpp", "26", "3gp", FileType::Video),
    MOV("com.apple.quicktime-movie", "23", "mov", FileType::Video),
    MP4("public.mpeg-4", "24", "mp4", FileType::Video),
    MPEG("public.mpeg", "_public.mpeg", "mpg", FileType::Video),
    M4V("com.apple.m4v-video", "25", "m4v", FileType::Video)
}
