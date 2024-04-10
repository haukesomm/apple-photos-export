use derive_new::new;
use diesel::dsl;
use diesel::dsl::count;
use diesel::prelude::*;

use crate::db::connection::establish_connection;
use crate::db::model::album::Album;
use crate::db::model::asset::{AlbumAsset, Asset, AssetAttributes};
use crate::db::model::internal_resource::InternalResource;
use crate::db::schema::*;
use crate::model::album::Kind;


pub enum HiddenAssetFilter {
    IncludeHidden,
    OnlyHidden,
    None
}

type AssetVisibilityFilter = dsl::And<
    dsl::And<
        dsl::And<
            dsl::Eq<assets::columns::trashed, bool>,
            dsl::EqAny<assets::columns::hidden, Vec<bool>>
        >,
        dsl::Eq<assets::columns::visibility_state, i32>
    >,
    dsl::Eq<assets::columns::duplicate_asset_visibility_state, i32>
>;

fn asset_visibility(hidden_asset_filter: &HiddenAssetFilter) -> AssetVisibilityFilter {
    assets::trashed.eq(false)
        .and(assets::hidden.eq_any(
            match hidden_asset_filter {
                HiddenAssetFilter::IncludeHidden => vec![true, false],
                HiddenAssetFilter::OnlyHidden => vec![true],
                HiddenAssetFilter::None => vec![false]
            }
        ))
        .and(assets::visibility_state.eq(0))
        .and(assets::duplicate_asset_visibility_state.eq(0))
}


pub enum AlbumFilter {
    Include(Vec<i32>),
    Exclude(Vec<i32>),
    None
}


pub type ExportableAsset = (Asset, AssetAttributes, Option<Album>);

#[derive(new)]
pub struct AssetRepository {
    db_path: String,
    hidden_asset_filter: HiddenAssetFilter,
    album_filter: AlbumFilter
}

impl AssetRepository {

    pub fn get_visible_count(&self) -> QueryResult<i64> {
        let mut conn = establish_connection(&self.db_path);
        assets::table
            .inner_join(asset_attributes::table)
            .inner_join(
                internal_resources::table
                    .on(internal_resources::fingerprint.eq(asset_attributes::master_fingerprint))
            )
            .filter(asset_visibility(&HiddenAssetFilter::IncludeHidden))
            .select(count(assets::id))
            .first(&mut conn)
    }

    pub fn get_visible_offloaded_count(&self) -> QueryResult<i64> {
        let mut conn = establish_connection(&self.db_path);
        assets::table
            .inner_join(asset_attributes::table)
            .inner_join(
                internal_resources::table
                    .on(internal_resources::fingerprint.eq(asset_attributes::master_fingerprint))
            )
            .filter(
                asset_visibility(&HiddenAssetFilter::IncludeHidden)
                    .and(internal_resources::local_availability.ne(1))
            )
            .select(count(assets::id))
            .first(&mut conn)
    }

    pub fn get_exportable(&self) -> QueryResult<Vec<ExportableAsset>> {
        let mut conn = establish_connection(&self.db_path);

        let album_kinds = [Kind::Root, Kind::UserAlbum, Kind::UserFolder]
            .map(|k| k as i32);

        let mut query = assets::table
            .inner_join(
                asset_attributes::table.inner_join(
                    internal_resources::table
                        .on(internal_resources::fingerprint.eq(asset_attributes::master_fingerprint))
                )
            )
            .left_join(
                album_assets::table.inner_join(albums::table)
            )
            .filter(
                asset_visibility(&self.hidden_asset_filter)
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

        query = match &self.album_filter {
            AlbumFilter::Include(ids) => query.filter(
                albums::id.eq_any(ids)
            ),
            AlbumFilter::Exclude(ids) => query.filter(
                albums::id.ne_all(ids).or(albums::id.is_null())
            ),
            AlbumFilter::None => query
        };

        let result = query
            .load::<(Asset, AssetAttributes, InternalResource, Option<AlbumAsset>, Option<Album>)>(&mut conn)?;

        Ok(
            result
                .iter()
                .map(|(asset, attributes, _, _, albums)| {
                    (asset.clone(), attributes.clone(), albums.clone())
                })
                .collect::<Vec<ExportableAsset>>()
        )
    }
}