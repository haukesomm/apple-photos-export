const UTI_HEIC: &str = "public.heic";
const UTI_JPEG: &str = "public.jpeg";
const UTI_PNG: &str = "public.png";
const UTI_GIF: &str = "com.compuserve.gif";
const UTI_RAW: &str = "com.adobe.raw-image";
const UTI_MP4: &str = "public.mpeg-4";
const UTI_MOV: &str = "com.apple.quicktime-movie";

// Reverse-engineered compact UTIs
// These are probably some kind of serialized internal representation.
// In some cases, in the database, we only have these compact UTIs instead of the full UTI
const COMPACT_UTI_HEIC: i32 = 3;
const COMPACT_UTI_JPEG: i32 = 1;
const COMPACT_UTI_PNG: i32 = 6;
const COMPACT_UTI_GIF: i32 = 7;
const COMPACT_UTI_RAW: i32 = 9;
const COMPACT_UTI_MP4: i32 = 24;
const COMPACT_UTI_MOV: i32 = 23;

const PICTURE_DERIVATE_SUFFIX: &str = "_1_201_a";
const VIDEO_DERIVATE_SUFFIX: &str = "_2_0_a";

static HEIC: Uti = Uti::new(UTI_HEIC, COMPACT_UTI_HEIC, PICTURE_DERIVATE_SUFFIX, "heic");
static JPEG: Uti = Uti::new(UTI_JPEG, COMPACT_UTI_JPEG, PICTURE_DERIVATE_SUFFIX, "jpeg");
static PNG: Uti = Uti::new(UTI_PNG, COMPACT_UTI_PNG, PICTURE_DERIVATE_SUFFIX, "png");
static GIF: Uti = Uti::new(UTI_GIF, COMPACT_UTI_GIF, PICTURE_DERIVATE_SUFFIX, "gif");
static RAW: Uti = Uti::new(UTI_RAW, COMPACT_UTI_RAW, PICTURE_DERIVATE_SUFFIX, "dng");
static MP4: Uti = Uti::new(UTI_MP4, COMPACT_UTI_MP4, VIDEO_DERIVATE_SUFFIX, "mp4");
static MOV: Uti = Uti::new(UTI_MOV, COMPACT_UTI_MOV, VIDEO_DERIVATE_SUFFIX, "mov");

#[derive(PartialEq)]
pub struct Uti {
    pub uti: &'static str,
    pub compact_uti: i32,
    pub uuid_suffix: &'static str,
    pub extension: &'static str,
}

impl Uti {
    pub const fn new(uti: &'static str, compact_uti: i32, uuid_suffix: &'static str,
                     extension: &'static str) -> Self {
        Self { uti, compact_uti, uuid_suffix, extension }
    }

    pub fn from_name(name: &str) -> Result<&'static Uti, String> {
        match name {
            UTI_HEIC => Ok(&HEIC),
            UTI_JPEG => Ok(&JPEG),
            UTI_PNG => Ok(&PNG),
            UTI_GIF => Ok(&GIF),
            UTI_RAW => Ok(&RAW),
            UTI_MP4 => Ok(&MP4),
            UTI_MOV => Ok(&MOV),
            _ => Err(format!("Unknown UTI: {}", name))
        }
    }

    pub fn from_compact(compact: i32) -> Result<&'static Uti, String> {
        match compact {
            COMPACT_UTI_HEIC => Ok(&HEIC),
            COMPACT_UTI_JPEG => Ok(&JPEG),
            COMPACT_UTI_PNG => Ok(&PNG),
            COMPACT_UTI_GIF => Ok(&GIF),
            COMPACT_UTI_RAW => Ok(&RAW),
            COMPACT_UTI_MP4 => Ok(&MP4),
            COMPACT_UTI_MOV => Ok(&MOV),
            _ => Err(format!("Unknown compact UTI: {}", compact))
        }
    }
}