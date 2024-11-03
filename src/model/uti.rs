const UTI_HEIC: &str = "public.heic";
const UTI_JPEG: &str = "public.jpeg";
const UTI_PNG: &str = "public.png";
const UTI_GIF: &str = "com.compuserve.gif";
const UTI_BMP: &str = "com.microsoft.bmp";
const UTI_DNG: &str = "com.adobe.raw-image";
const UTI_RAF: &str = "com.fuji.raw-image";
const UTI_MP4: &str = "public.mpeg-4";
const UTI_MOV: &str = "com.apple.quicktime-movie";

// Reverse-engineered compact UTIs
// These are probably some kind of serialized internal representation.
// In some cases, in the database, we only have these compact UTIs instead of the full UTI
const COMPACT_UTI_HEIC: &str = "3";
const COMPACT_UTI_JPEG: &str = "1";
const COMPACT_UTI_PNG: &str = "6";
const COMPACT_UTI_GIF: &str = "7";
const COMPACT_UTI_BMP: &str = "_com.microsoft.bmp";
const COMPACT_UTI_DNG: &str = "9";
const COMPACT_UTI_RAF: &str = "21";
const COMPACT_UTI_MP4: &str = "24";
const COMPACT_UTI_MOV: &str = "23";

const EXTENSION_HEIC: &str = "heic";
const EXTENSION_JPEG: &str = "jpeg";
const EXTENSION_JPG: &str = "jpg";
const EXTENSION_PNG: &str = "png";
const EXTENSION_GIF: &str = "gif";
const EXTENSION_BMP: &str = "bmp";
const EXTENSION_DNG: &str = "dng";
const EXTENSION_RAF: &str = "raf";
const EXTENSION_MP4: &str = "mp4";
const EXTENSION_MOV: &str = "mov";

const PICTURE_DERIVATE_SUFFIX: &str = "_1_201_a";
const VIDEO_DERIVATE_SUFFIX: &str = "_2_0_a";

static HEIC: Uti = Uti::new(UTI_HEIC, COMPACT_UTI_HEIC, PICTURE_DERIVATE_SUFFIX, EXTENSION_HEIC);
static JPEG: Uti = Uti::new(UTI_JPEG, COMPACT_UTI_JPEG, PICTURE_DERIVATE_SUFFIX, EXTENSION_JPEG);
static JPG: Uti = Uti::new(UTI_JPEG, COMPACT_UTI_JPEG, PICTURE_DERIVATE_SUFFIX, EXTENSION_JPG);
static PNG: Uti = Uti::new(UTI_PNG, COMPACT_UTI_PNG, PICTURE_DERIVATE_SUFFIX, EXTENSION_PNG);
static GIF: Uti = Uti::new(UTI_GIF, COMPACT_UTI_GIF, PICTURE_DERIVATE_SUFFIX, EXTENSION_GIF);
static BMP: Uti = Uti::new(UTI_BMP, COMPACT_UTI_BMP, PICTURE_DERIVATE_SUFFIX, EXTENSION_BMP);
static DNG: Uti = Uti::new(UTI_DNG, COMPACT_UTI_DNG, PICTURE_DERIVATE_SUFFIX, EXTENSION_DNG);
static RAF: Uti = Uti::new(UTI_RAF, COMPACT_UTI_RAF, PICTURE_DERIVATE_SUFFIX, EXTENSION_RAF);
static MP4: Uti = Uti::new(UTI_MP4, COMPACT_UTI_MP4, VIDEO_DERIVATE_SUFFIX, EXTENSION_MP4);
static MOV: Uti = Uti::new(UTI_MOV, COMPACT_UTI_MOV, VIDEO_DERIVATE_SUFFIX, EXTENSION_MOV);

#[derive(PartialEq)]
pub struct Uti {
    pub uti: &'static str,
    pub compact_uti: &'static str,
    pub uuid_suffix: &'static str,
    pub extension: &'static str,
}

impl Uti {
    pub const fn new(
        uti: &'static str,
        compact_uti: &'static str,
        uuid_suffix: &'static str,
        extension: &'static str,
    ) -> Self {
        Self { uti, compact_uti, uuid_suffix, extension }
    }

    pub fn from_name(name: &str) -> Result<&'static Uti, String> {
        match name {
            UTI_HEIC => Ok(&HEIC),
            UTI_JPEG => Ok(&JPEG),
            UTI_PNG => Ok(&PNG),
            UTI_GIF => Ok(&GIF),
            UTI_BMP => Ok(&BMP),
            UTI_DNG => Ok(&DNG),
            UTI_RAF => Ok(&RAF),
            UTI_MP4 => Ok(&MP4),
            UTI_MOV => Ok(&MOV),
            _ => Err(format!("Unknown UTI: {}", name))
        }
    }

    pub fn from_compact_and_filename(compact: &str, filename: &str) -> Result<&'static Uti, String> {
        let extension = Self::extension_from_filename(filename)?;

        match (compact, extension) {
            (COMPACT_UTI_HEIC, _) => Ok(&HEIC),
            (COMPACT_UTI_JPEG, EXTENSION_JPEG) => Ok(&JPEG),
            (COMPACT_UTI_JPEG, EXTENSION_JPG) => Ok(&JPG),
            (COMPACT_UTI_PNG, _) => Ok(&PNG),
            (COMPACT_UTI_GIF, _) => Ok(&GIF),
            (COMPACT_UTI_BMP, _) => Ok(&BMP),
            (COMPACT_UTI_DNG, _) => Ok(&DNG),
            (COMPACT_UTI_RAF, _) => Ok(&RAF),
            (COMPACT_UTI_MP4, _) => Ok(&MP4),
            (COMPACT_UTI_MOV, _) => Ok(&MOV),
            _ => Err(format!("Unknown compact UTI: {}", compact))
        }
    }

    pub fn from_filename(filename: &String) -> Result<&'static Uti, String> {
        let extension = Self::extension_from_filename(filename)?;

        match extension {
            EXTENSION_HEIC => Ok(&HEIC),
            EXTENSION_JPEG => Ok(&JPEG),
            EXTENSION_PNG => Ok(&PNG),
            EXTENSION_GIF => Ok(&GIF),
            EXTENSION_BMP => Ok(&BMP),
            EXTENSION_DNG => Ok(&DNG),
            EXTENSION_RAF => Ok(&RAF),
            EXTENSION_MP4 => Ok(&MP4),
            EXTENSION_MOV => Ok(&MOV),
            _ => Err(format!("Unknown extension: {}", extension))
        }
    }

    fn extension_from_filename(filename: &str) -> Result<&str, String> {
        filename
            .split('.')
            .last()
            .ok_or(format!("File {} seems to have no extension!", filename))
    }
}