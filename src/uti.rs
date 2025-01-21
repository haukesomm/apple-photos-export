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
pub struct Uti {
    
    /// Identifier of the UTI.
    pub id: &'static str,
    
    /// Compact identifier of the UTI. Typically. but not always, a number.
    pub cid: &'static str,
    
    /// File extension associated with the UTI.
    pub ext: &'static str,
    
    /// Suffix that is appended to all derived assets of that type when stored in the Photos 
    /// library.
    pub derivate_suffix: &'static str,
}


/// Macro used to define the known UTIs as constants on the `Uti` struct.
/// 
/// While the respective constants could be defined manually, this macro also generates the getter
/// methods used to determine the UTI from a file extension or an identifier.
///This way, it is not necessary to manually define the getter methods for each UTI and keep them in
/// sync with the constants.
macro_rules! uti_constants {
    ($($name:ident($id:expr, $cid:expr, $ext:expr, $suffix:expr)),+) => {
        impl Uti {
            $(
            pub const $name:Self = Self { id: $id, cid: $cid, ext: $ext, derivate_suffix: $suffix };
            )*

            /// Determines the UTI from the given identifier.
            /// 
            /// > **Note**: The identifier is _not_ the compact identifier, but the full identifier
            /// > of the UTI!
            pub fn from_id(id: &str) -> Result<Self, String> {
                match id {
                    $($id => Ok(Self::$name),)*
                    _ => Err(format!("Cannot determine UTI (unknown ID): {}", id))
                }
            }

            /// Determines the UTI from the given file extension.
            pub  fn from_filename(filename: &str) -> Result<Self, String> {
                use crate::util::ExtractFileExtension;
                let extension = filename.file_extension()?;

                match extension.as_str() {
                    $($ext => Ok(Self::$name),)*
                    _ => Err(format!("Cannot determine UTI (unknown file extension): {}", filename))
                }
            }
         }
    };
}


/// Suffix that is appended to all derived _image_ assets when stored in the Photos library.
const DERIVATE_SUFFIX_IMG: &'static str = "_1_201_a";

/// Suffix that is appended to all derived _video_ assets when stored in the Photos library.
const DERIVATE_SUFFIX_VID: &'static str = "_2_0_a";

uti_constants! {
    JPEG("public.jpeg", "1", "jpeg", DERIVATE_SUFFIX_IMG),
    HEIC("public.heic", "3", "heic", DERIVATE_SUFFIX_IMG),
    PNG("public.png", "6", "png", DERIVATE_SUFFIX_IMG),
    GIF("com.compuserve.gif", "7", "gif", DERIVATE_SUFFIX_IMG),
    DNG("com.adobe.raw-image", "9", "dng", DERIVATE_SUFFIX_IMG),
    RAF("com.fuji.raw-image", "21", "raf", DERIVATE_SUFFIX_IMG),
    MOV("com.apple.quicktime-movie", "23", "mov", DERIVATE_SUFFIX_VID),
    MP4("public.mpeg-4", "24", "mp4", DERIVATE_SUFFIX_VID),
    BMP("com.microsoft.bmp", "_com.microsoft.bmp", "bmp", DERIVATE_SUFFIX_IMG)
}
