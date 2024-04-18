use derive_new::new;
use diesel::dsl;
use diesel::dsl::count;
use diesel::prelude::*;

use crate::db::connection::establish_connection;
use crate::db::model::album::Album;
use crate::db::model::asset::{AlbumAsset, Asset, AssetAttributes};
use crate::db::model::internal_resource::InternalResource;
use crate::db::repo::asset::LocalAvailability::Offloaded;
use crate::db::schema::*;
use crate::model::album::Kind;


pub enum HiddenAssets {
    Include,
    Require,
    Exclude
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

fn filter_visible(hidden_assets: &HiddenAssets) -> AssetVisibilityFilter {
    assets::trashed.eq(false)
        .and(assets::hidden.eq_any(
            match hidden_assets {
                HiddenAssets::Include => vec![true, false],
                HiddenAssets::Require => vec![true],
                HiddenAssets::Exclude => vec![false]
            }
        ))
        .and(assets::visibility_state.eq(0))
        .and(assets::duplicate_asset_visibility_state.eq(0))
}


pub enum LocalAvailability {
    Any,
    Offloaded
}


pub enum AlbumFilter {
    Include(Vec<i32>),
    Exclude(Vec<i32>),
    None
}


pub type ExportableAsset = (Asset, AssetAttributes, InternalResource, Option<Album>);

#[derive(new)]
pub struct AssetRepository {
    db_path: String,
    hidden_assets: HiddenAssets,
    album_filter: AlbumFilter
}

impl AssetRepository {

    pub fn get_visible_count(&self, availability: LocalAvailability) -> QueryResult<i64> {
        let mut conn = establish_connection(&self.db_path);
        let mut boxed_select = assets::table
            .inner_join(asset_attributes::table)
            .inner_join(
                internal_resources::table
                    .on(internal_resources::fingerprint.eq(asset_attributes::master_fingerprint))
            )
            .filter(filter_visible(&HiddenAssets::Include))
            .select(count(assets::id))
            .into_boxed();

        if let Offloaded = availability {
            boxed_select = boxed_select
                .filter(internal_resources::local_availability.ne(1));
        }

        Ok(boxed_select.first(&mut conn)?)
    }

    pub fn get_exportable(&self) -> QueryResult<Vec<ExportableAsset>> {
        let mut conn = establish_connection(&self.db_path);

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
                filter_visible(&self.hidden_assets)
                    .and(internal_resources::local_availability.eq(1))
                    .and(
                        albums::kind.is_null()
                            .or(
                                albums::trashed.eq(false)
                                    .and(albums::kind.eq_any(Kind::int_values()))
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
                .map(|(asset, attributes, internal_resources, _, albums)| {
                    (asset.clone(), attributes.clone(), internal_resources.clone(), albums.clone())
                })
                .collect::<Vec<ExportableAsset>>()
        )
    }
}