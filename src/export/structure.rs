use std::ffi::OsStr;
use std::path::PathBuf;

use chrono::NaiveDate;
use derive_new::new;

use crate::model::asset::AssetWithAlbumInfo;

pub trait OutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> PathBuf;
}


#[derive(new)]
pub struct PlainOutputStructureStrategy;

impl OutputStructureStrategy for PlainOutputStructureStrategy {

    fn get_relative_output_dir(&self, _: &AssetWithAlbumInfo) -> PathBuf {
        PathBuf::new()
    }
}


#[derive(new)]
pub struct AlbumOutputStructureStrategy {
    pub flatten: bool
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


type DateSelectorFunc = Box<dyn Fn(&AssetWithAlbumInfo) -> NaiveDate>;

pub struct YearMonthOutputStructureStrategy {
    date_selector: DateSelectorFunc
}

impl YearMonthOutputStructureStrategy {

    pub fn asset_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            date_selector: Box::new(|a| a.date)
        }
    }

    pub fn album_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            date_selector: Box::new(|a| a.album_start_date.unwrap_or(a.date))
        }
    }
}

impl OutputStructureStrategy for YearMonthOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &AssetWithAlbumInfo) -> PathBuf {
        let datetime = (self.date_selector)(asset);
        let formatted = format!("{}", datetime.format("%Y/%m/"));
        PathBuf::new().join(formatted)
    }
}


#[derive(new)]
pub struct JoiningOutputStructureStrategy {
    strategies: Vec<Box<dyn OutputStructureStrategy>>
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