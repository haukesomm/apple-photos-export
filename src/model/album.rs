/// Represents an album stored in the Photos database.
pub struct Album {
    /// Unique integer ID used to identify the album in the database.
    pub id: i32,

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

impl Album {
    /// Returns `true` is the album is the root album, `false` otherwise.
    ///
    /// An album is considered the root album if it does not have a parent.
    pub fn is_root_album(&self) -> bool {
        self.parent_id.is_none()
    }
}
