use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::model::asset::AssetWithAlbumInfo;

pub trait OutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> String;
}


pub struct PlainOutputStructureStrategy;

impl PlainOutputStructureStrategy {
    pub fn new() -> PlainOutputStructureStrategy {
        PlainOutputStructureStrategy {}
    }
}

impl OutputStructureStrategy for PlainOutputStructureStrategy {

    fn get_relative_output_dir(&self, _: &AssetWithAlbumInfo) -> String {
        "".to_string()
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
    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> String {
        let path_raw = match &asset.album_path {
            None => String::new(),
            Some(p) => p.clone()
        };
        let path = Path::new(&path_raw);

        if self.flatten {
            path.file_name().unwrap().to_string_lossy().to_string()
        } else {
            path.to_string_lossy().to_string()
        }
    }
}


pub type DateSelectorFunc = dyn Fn(&AssetWithAlbumInfo) -> NaiveDate;

pub struct YearMonthOutputStructureStrategy<'a> {
    pub date_selector: &'a DateSelectorFunc
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
    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> String {
        let datetime = (self.date_selector)(asset);
        format!("{}", datetime.format("%Y/%m/"))
    }
}


pub struct JoiningOutputStructureStrategy {
    pub strategies: Vec<Box<dyn OutputStructureStrategy>>
}

impl JoiningOutputStructureStrategy {
    pub fn new(strategies: Vec<Box<dyn OutputStructureStrategy>>) -> JoiningOutputStructureStrategy {
        JoiningOutputStructureStrategy { strategies }
    }
}

impl OutputStructureStrategy for JoiningOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> String {
        self.strategies
            .iter()
            .fold(PathBuf::new(), |path, strategy| {
                let dir = strategy.get_relative_output_dir(asset);
                let p = Path::new(&dir);
                path.join(p)
            })
            .as_path()
            .to_str()
            .unwrap()
            .to_string()
    }
}