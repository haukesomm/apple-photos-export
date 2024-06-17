use diesel::{Queryable, Selectable};

use crate::foundation::cocoa;
use crate::model::album::Kind;
use crate::model::FromDbModel;

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::albums)]
pub struct AlbumDto {
    pub id: i32,
    pub kind: i32,
    pub parent_id: Option<i32>,
    pub name: Option<String>,
    pub start_date: Option<f32>,
    pub trashed: bool,
}

impl FromDbModel<AlbumDto> for crate::model::album::Album {
    fn from_db_model(model: &AlbumDto) -> Result<Self, String> {
        Ok(crate::model::album::Album {
            id: model.id,
            kind: Kind::try_from(model.kind)?,
            name: model.name.clone(),
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