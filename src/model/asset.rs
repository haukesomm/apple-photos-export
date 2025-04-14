use crate::foundation::Uti;

/// Represents an asset in the Photos library.
/// 
/// This struct does not reflect the asset table in the Photos library database! Instead, it is
/// a combined representation of data from multiple tables needed to work with the asset in the
/// export process.
pub struct Asset {
    
    /// Internal id of the asset in the Photos database
    pub id: i32,
    
    /// The UUID of the asset.
    pub uuid: String,
    
    /// The directory where the asset is stored in the Photos library.
    pub dir: String,
    
    /// The filename of the asset.
    pub filename: String,
    
    /// The original UTI of the asset.
    /// 
    /// The original is the UTI of the _unmodified original_ asset.
    /// The UTI of the derivative asset is stored in `derivate_uti`.
    pub original_uti: Uti,
    
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
    
    /// List of ids of the albums the asset is part of.
    pub album_ids: Vec<i32>,
}