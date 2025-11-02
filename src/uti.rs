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
    ($($name:ident($id:expr, $ext:expr, $suffix:expr)),+) => {
        impl Uti {
            $(
            pub const $name:Self = Self { ext: $ext, derivate_suffix: $suffix };
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
         }
    };
}

/// Suffix that is appended to all derived _image_ assets when stored in the Photos library.
const DERIVATE_SUFFIX_IMG: &'static str = "_1_201_a";

/// Suffix that is appended to all derived _video_ assets when stored in the Photos library.
const DERIVATE_SUFFIX_VID: &'static str = "_2_0_a";

uti_constants! {
    JPEG("public.jpeg", "jpeg", DERIVATE_SUFFIX_IMG),
    HEIC("public.heic", "heic", DERIVATE_SUFFIX_IMG),
    PNG("public.png", "png", DERIVATE_SUFFIX_IMG),
    GIF("com.compuserve.gif", "gif", DERIVATE_SUFFIX_IMG),
    DNG("com.adobe.raw-image", "dng", DERIVATE_SUFFIX_IMG),
    RAF("com.fuji.raw-image", "raf", DERIVATE_SUFFIX_IMG),
    MOV("com.apple.quicktime-movie", "mov", DERIVATE_SUFFIX_VID),
    MP4("public.mpeg-4", "mp4", DERIVATE_SUFFIX_VID),
    BMP("com.microsoft.bmp", "bmp", DERIVATE_SUFFIX_IMG),
    M4V("com.apple.m4v-video", "m4v", DERIVATE_SUFFIX_VID),
    GPP("public.3gpp", "3gp", DERIVATE_SUFFIX_VID),
    CR2("com.canon.cr2-raw-image", "cr2", DERIVATE_SUFFIX_IMG)
}
