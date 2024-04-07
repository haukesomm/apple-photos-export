use diesel::{Queryable, Selectable};

#[derive(Debug, PartialEq)]
pub enum Kind {
    Root = 3999,
    UserFolder = 4000,
    UserAlbum = 2,
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::albums)]
pub struct Album {
    pub id: i32,
    pub kind: i32,
    pub parent_id: Option<i32>,
    pub name: Option<String>,
    pub start_date: Option<f32>,
    pub trashed: bool,
}

impl Album {
    pub fn is_of_kind(&self, kind: Kind) -> bool {
        self.kind == kind as i32
    }
}