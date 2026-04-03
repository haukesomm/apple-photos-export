use crate::export::task::mapping::MapAsset;
use crate::export::task::AssetMapping;
use crate::model::album::Album;
use chrono::{Datelike, NaiveDateTime};
use std::collections::HashMap;
use std::path::PathBuf;

trait GetOutputDatetime {
    fn get_output_date(&self, mapping: &AssetMapping) -> chrono::NaiveDateTime;
}

struct AssetBasedDatetimeGetter;

impl GetOutputDatetime for AssetBasedDatetimeGetter {
    fn get_output_date(&self, mapping: &AssetMapping) -> NaiveDateTime {
        mapping.asset.datetime
    }
}

struct AlbumBasedDatetimeGetter {
    album_cache: HashMap<i32, Album>,
}

impl GetOutputDatetime for AlbumBasedDatetimeGetter {
    fn get_output_date(&self, mapping: &AssetMapping) -> NaiveDateTime {
        mapping
            .album_id
            .map(|id| self.album_cache.get(&id))
            .flatten()
            .map(|a| a.start_date)
            .flatten()
            .unwrap_or(mapping.asset.datetime)
    }
}

/// A mapper that groups assets by year and month.
pub struct ByYearAndMonth {
    datetime_getter: Box<dyn GetOutputDatetime>,
}

impl ByYearAndMonth {
    pub fn of_asset() -> Self {
        Self {
            datetime_getter: Box::new(AssetBasedDatetimeGetter),
        }
    }

    pub fn of_album(album_cache: HashMap<i32, Album>) -> Self {
        Self {
            datetime_getter: Box::new(AlbumBasedDatetimeGetter { album_cache }),
        }
    }
}

impl MapAsset for ByYearAndMonth {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let datetime = self.datetime_getter.get_output_date(&mapping);

        let mut prefix = PathBuf::new();
        prefix.push(datetime.year().to_string());
        prefix.push(format!("{:>02}", datetime.month()));

        AssetMapping {
            destination_dir: PathBuf::from(prefix).join(&mapping.destination_dir),
            ..mapping
        }
    }
}
