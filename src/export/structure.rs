use std::ffi::OsStr;
use std::path::PathBuf;

use chrono::NaiveDate;

use crate::model::asset::AssetWithAlbumInfo;

pub trait OutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> PathBuf;
}


pub struct PlainOutputStructureStrategy;

impl PlainOutputStructureStrategy {
    pub fn new() -> PlainOutputStructureStrategy {
        PlainOutputStructureStrategy
    }
}

impl OutputStructureStrategy for PlainOutputStructureStrategy {

    fn get_relative_output_dir(&self, _: &AssetWithAlbumInfo) -> PathBuf {
        PathBuf::new()
    }
}


pub struct AlbumOutputStructureStrategy {
    pub flatten: bool
}

impl AlbumOutputStructureStrategy {
    pub fn new(flatten: bool) -> AlbumOutputStructureStrategy {
        AlbumOutputStructureStrategy { flatten }
    }
}

impl OutputStructureStrategy for AlbumOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> PathBuf {
        let path = PathBuf::new()
            .join(asset.album_path.clone().unwrap_or(String::new()));

        if self.flatten {
            let filename = path.file_name().unwrap_or(OsStr::new(""));
            PathBuf::new().join(filename)
        } else {
            path
        }
    }
}


type DateSelectorFunc = dyn Fn(&AssetWithAlbumInfo) -> NaiveDate;

pub struct YearMonthOutputStructureStrategy<'a> {
    date_selector: &'a DateSelectorFunc
}

impl YearMonthOutputStructureStrategy<'_> {

    pub fn asset_date_based<'a>() -> YearMonthOutputStructureStrategy<'a> {
        YearMonthOutputStructureStrategy {
            date_selector: &(|a| a.date)
        }
    }

    pub fn album_date_based<'a>() -> YearMonthOutputStructureStrategy<'a> {
        YearMonthOutputStructureStrategy {
            date_selector: &(|a| a.album_start_date.unwrap_or(a.date))
        }
    }
}

impl OutputStructureStrategy for YearMonthOutputStructureStrategy<'_> {
    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> PathBuf {
        let datetime = (self.date_selector)(asset);
        let formatted = format!("{}", datetime.format("%Y/%m/"));
        PathBuf::new().join(formatted)
    }
}


pub struct JoiningOutputStructureStrategy {
    strategies: Vec<Box<dyn OutputStructureStrategy>>
}

impl JoiningOutputStructureStrategy {
    pub fn new(strategies: Vec<Box<dyn OutputStructureStrategy>>) -> JoiningOutputStructureStrategy {
        JoiningOutputStructureStrategy { strategies }
    }
}

impl OutputStructureStrategy for JoiningOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> PathBuf {
        self.strategies
            .iter()
            .fold(PathBuf::new(), |path, strategy| {
                let dir = strategy.get_relative_output_dir(asset);
                path.join(dir)
            })
    }
}