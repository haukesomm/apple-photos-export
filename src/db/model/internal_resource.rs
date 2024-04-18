use diesel::{Identifiable, Queryable, Selectable};

#[derive(Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::db::schema::internal_resources)]
pub struct InternalResource {
    pub id: i32,
    pub fingerprint: String,
    pub local_availability: i32,
    pub compact_uti: i32,
}