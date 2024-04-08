use diesel::{Queryable, Selectable};

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