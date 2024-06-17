use derive_new::new;
use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};

use crate::db::connection::establish_connection;
use crate::db::model::album::AlbumDto;
use crate::db::schema::albums::{kind, start_date, trashed};
use crate::db::schema::albums::dsl::albums;
use crate::model::album::Kind;

#[derive(new)]
pub struct AlbumRepository {
    db_path: String
}

impl AlbumRepository {

    pub fn get_all(&self) -> QueryResult<Vec<AlbumDto>> {
        let mut conn = establish_connection(&self.db_path);

        let album_types = [Kind::Root, Kind::UserAlbum, Kind::UserFolder]
            .map(|k| k as i32);

        let result = albums
            .filter(kind.eq_any(&album_types))
            .filter(trashed.eq(false))
            .order_by(start_date.asc())
            .load::<AlbumDto>(&mut conn)?;

        Ok(result)
    }
}