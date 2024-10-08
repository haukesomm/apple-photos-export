use diesel::{deserialize::Queryable, Selectable};

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::metadata)]
pub struct MetadataDto {
    pub version: i32,
    pub plist: Vec<u8>
}