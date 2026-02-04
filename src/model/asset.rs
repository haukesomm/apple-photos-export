use crate::uti::Uti;

#[derive(Clone, PartialEq, Debug)]
pub struct DataStoreSubtype(pub usize);

macro_rules! data_store_subtype_impl {
    ($($name:ident($id:literal)),+) => {
        impl DataStoreSubtype {
            $(
            pub const $name:Self = Self($id);
            )*
        }

        impl TryFrom<usize> for DataStoreSubtype {
            type Error = ();

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                match value {
                    $($id => Ok(Self::$name),)*
                    _ => Err(())
                }
            }
        }
    };
}

data_store_subtype_impl!(ORIGINAL(1), ASSOCIATED_RAW_IMAGE(17));

/// Represents an asset in the Photos library.
///
/// This struct does not reflect the asset table in the Photos library database! Instead, it is
/// a combined representation of data from multiple tables needed to work with the asset in the
/// export process.
#[derive(Clone)]
pub struct Asset {
    pub id: usize,

    /// The UUID of the asset.
    pub uuid: String,

    /// The directory where the asset is stored in the Photos library.
    pub dir: String,

    /// The filename of the asset.
    pub filename: String,

    /// The UTI of the derived asset (e.g. an edited version of the original asset).
    ///
    /// The derivate_uti is the same as the original_uti if the asset is not a derivative.
    /// The original UTI is stored in `original_uti`.
    pub derivate_uti: Uti,

    /// Date and time when the asset was created.
    pub datetime: chrono::NaiveDateTime,

    /// Describes whether the asset is hidden.
    pub hidden: bool,

    /// The original filename of the asset before it was imported into the Photos library.
    pub original_filename: String,

    /// Describes whether the asset has been adjusted, i.e. edited.
    pub has_adjustments: bool,

    pub data_store_subtypes: Vec<DataStoreSubtype>,

    /// List of ids of the albums the asset is part of.
    pub album_ids: Vec<i32>,
}

impl Asset {
    pub fn has_associated_raw_image(&self) -> bool {
        self.data_store_subtypes
            .contains(&DataStoreSubtype::ASSOCIATED_RAW_IMAGE)
    }
}
