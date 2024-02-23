use chrono::NaiveDateTime;

#[derive(PartialEq)]
pub enum Kind {
    Root,
    UserFolder,
    UserAlbum
}

impl TryFrom<i32> for Kind {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            3999 => Ok(Self::Root),
            4000 => Ok(Self::UserFolder),
            2 => Ok(Self::UserAlbum),
            unknown => Err(format!("Unknown album kind id: {}", unknown))
        }
    }
}


pub struct Album {
    pub id: i32,
    pub kind: Kind,
    pub parent_id: Option<i32>,
    pub name: String,
    pub start_date: Option<NaiveDateTime>,
    pub asset_count: i32
}