use chrono::NaiveDateTime;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, PartialEq, EnumIter)]
pub enum Kind {
    Root = 3999,
    UserFolder= 4000,
    UserAlbum = 2,
}

impl Kind {
    pub fn int_values() -> Vec<i32> {
        Kind::iter().map(|k| k as i32).collect()
    }
}

impl TryFrom<i32> for Kind {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            3999 => Ok(Kind::Root),
            4000 => Ok(Kind::UserFolder),
            2 => Ok(Kind::UserAlbum),
            _ => Err(format!("Invalid album kind: {}", value)),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Album {
    pub id: i32,
    pub kind: Kind,
    pub name: Option<String>,
    pub parent_id: Option<i32>,
    pub start_date: Option<NaiveDateTime>,
    pub trashed: bool,
}