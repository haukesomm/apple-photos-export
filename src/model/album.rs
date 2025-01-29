use crate::foundation::macros::value_enum;

/// Represents the kind of album in the album hierarchy of the Photos app.
value_enum! {
    AlbumKind: i32 {
        /// Root album.
        ///
        /// This album is not visible to the user and serves only as a common root node for all 
        /// other albums.
        ROOT = 3999,
        
        /// A user-created folder.
        ///
        /// Albums of this type are created whenever the user creates a folder via the Photos app.
        /// Folder can itself contain both other folders and actual albums.
        USER_FOLDER = 4000,
        
        /// A user-created album.
        ///
        /// Albums of this type are the actual albums as the user knows them. They are created 
        /// whenever an actual album is created via the Photos app.
        USER_ALBUM = 2,
    }
}

/// Represents an album stored in the Photos database.
pub struct Album {
    
    /// Unique integer ID used to identify the album in the database.
    pub id: i32,
    
    /// Album [AlbumKind] representing the internal type of the album.
    pub kind: AlbumKind,
    
    /// Optional name of the album.
    /// 
    /// In practice, almost all albums do have a name. The only known exception is the 
    /// [root][Kind::ROOT] album.
    pub name: Option<String>,
    
    /// Optional parent id of the album.
    /// 
    /// In practice, almost all albums have a parent. The only known exception is the 
    /// [root][Kind::ROOT] album, as its name implies.
    pub parent_id: Option<i32>,
    
    /// Optional date referencing the earliest capture date of the assets associated with an album.
    /// 
    /// This may be used to sort albums by capture date.
    pub start_date: Option<chrono::NaiveDateTime>,
}