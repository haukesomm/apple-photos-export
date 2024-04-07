use derive_new::new;
use diesel::dsl::count;
use diesel::prelude::*;

use crate::db::connection::establish_connection;
use crate::db::model::album::{Album, Kind};
use crate::db::model::asset::{AlbumAsset, Asset, AssetAttributes};
use crate::db::model::internal_resource::InternalResource;
use crate::db::schema::*;

#[derive(Clone)]
pub enum AlbumFilter {
    Include(Vec<i32>),
    Exclude(Vec<i32>),
    None
}

#[derive(new)]
pub struct ExportableAssetsRepository {
    db_path: String,
    include_hidden: bool,
    album_filter: AlbumFilter
}

impl ExportableAssetsRepository {

    pub fn get_total_count(&self) -> i64 {
        let mut conn = establish_connection(&self.db_path);

        let query = assets::table
            .inner_join(asset_attributes::table)
            .inner_join(
                internal_resources::table
                    .on(internal_resources::fingerprint.eq(asset_attributes::master_fingerprint))
            )
            .filter(
                assets::trashed.eq(false)
                    .and(assets::hidden.eq_any([false, self.include_hidden]))
                    .and(assets::visibility_state.eq(0))
            )
            .select(count(assets::id));

        query.first(&mut conn).unwrap()
    }

    pub fn get_offloaded_count(&self) -> i64 {
        let mut conn = establish_connection(&self.db_path);

        let query = assets::table
            .inner_join(asset_attributes::table)
            .inner_join(
                internal_resources::table
                    .on(internal_resources::fingerprint.eq(asset_attributes::master_fingerprint))
            )
            .filter(
                assets::trashed.eq(false)
                    .and(assets::hidden.eq_any([false, self.include_hidden]))
                    .and(assets::visibility_state.eq(0))
                    .and(internal_resources::local_availability.ne(1))
            )
            .select(count(assets::id));

        query.first(&mut conn).unwrap()
    }

    pub fn get_exportable_assets(&self) -> Vec<(Asset, AssetAttributes, Option<Album>)> {
        let mut conn = establish_connection(&self.db_path);

        let album_kinds = [Kind::Root, Kind::UserAlbum, Kind::UserFolder]
            .map(|k| k as i32);

        let mut query = assets::table
            .inner_join(asset_attributes::table)
            .inner_join(
                internal_resources::table
                    .on(internal_resources::fingerprint.eq(asset_attributes::master_fingerprint))
            )
            .left_join(
                album_assets::table.left_join(albums::table)
            )
            .filter(
                assets::trashed.eq(false)
                    .and(assets::hidden.eq_any([false, self.include_hidden]))
                    .and(assets::visibility_state.eq(0))
                    .and(internal_resources::local_availability.eq(1))
                    .and(
                        albums::kind.is_null()
                            .or(
                                albums::trashed.eq(false)
                                    .and(albums::kind.eq_any(album_kinds))
                            )
                    )
            )
            .select((
                Asset::as_select(), AssetAttributes::as_select(), InternalResource::as_select(),
                Option::<AlbumAsset>::as_select(), Option::<Album>::as_select()
            ))
            .into_boxed();

        query = match self.album_filter.clone() {
            AlbumFilter::Include(ids) => query.filter(
                albums::id.eq_any(ids)
            ),
            AlbumFilter::Exclude(ids) => query.filter(
                albums::id.ne_all(ids).or(albums::id.is_null())
            ),
            AlbumFilter::None => query
        };

        let result = query
            .load::<(Asset, AssetAttributes, InternalResource, Option<AlbumAsset>, Option<Album>)>(
                &mut conn
            ).unwrap();

        result.iter().map(|(asset, attributes, _, _, albums)| {
            (asset.clone(), attributes.clone(), albums.clone())
        }).collect()
    }
}