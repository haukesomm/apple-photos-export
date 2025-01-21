/// Represents the kind of album in the album hierarchy of the Photos app.
pub struct Kind {
    
    /// ID used in the database.
    pub id: i32,
}

impl Kind {
    
    /// Root album.
    /// 
    /// This album is not visible to the user and serves only as a common root node for all other
    /// albums.
    pub const ROOT: Self = Self { id: 3999 };
    
    /// A user-created folder.
    /// 
    /// Albums of this type are created whenever the user creates a folder via the Photos app.
    /// Folder can itself contain both other folders and actual albums.
    pub const USER_FOLDER: Self = Self { id: 4000 };
    
    /// A user-created album.
    /// 
    /// Albums of this type are the actual albums as the user knows them. They are created whenever
    /// an actual album is created via the Photos app.
    pub const USER_ALBUM: Self = Self { id: 2 };
}


/// Represents an album stored in the Photos database.
pub struct Album {
    
    /// Unique integer ID used to identify the album in the database.
    pub id: i32,
    
    /// Album [Kind] representing the internal type of the album.
    pub kind: Kind,
    
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
    
    /// Stores whether the album has been deleted or not.
    pub trashed: bool,
}