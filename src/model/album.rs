use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::foundation::cocoa;
use crate::model::FromDbModel;

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

#[derive(Clone)]
pub struct Album {
    pub id: i32,
    pub kind: Kind,
    pub name: Option<String>,
    pub parent_id: Option<i32>,
    pub start_date: Option<NaiveDateTime>,
    pub trashed: bool,
}

impl Album {

    pub fn get_path(&self, albums: &HashMap<i32, Album>) -> Result<PathBuf, String> {
        self.get_path_recursively(self.id, albums)
    }

    fn get_path_recursively(&self, album_id: i32, albums_by_id: &HashMap<i32, Album>) -> Result<PathBuf, String> {
        let album = albums_by_id
            .get(&album_id)
            .ok_or(format!("Album with ID {} not found", album_id))?;

        match album.parent_id {
            None => {
                let mut buffer = PathBuf::new();
                if let Some(name) = &album.name {
                    buffer.push(name);
                }
                Ok(buffer)
            },
            Some(parent_id) => {
                let path = self.get_path_recursively(parent_id, albums_by_id)?;
                Ok(path.join(album.name.clone().unwrap_or(String::from("unnamed"))))
            }
        }
    }
}

pub type AlbumDbModel = crate::db::model::album::Album;

impl FromDbModel<AlbumDbModel> for Album {
    fn from_db_model(model: AlbumDbModel) -> Result<Self, String> {
        Ok(Album {
            id: model.id,
            kind: Kind::try_from(model.kind)?,
            name: model.name,
            parent_id: model.parent_id,
            start_date: match model.start_date {
                None => None,
                Some(d) => {
                    let date = cocoa::parse_cocoa_timestamp(d)?;
                    Some(date)
                }
            },
            trashed: model.trashed,
        })
    }
}