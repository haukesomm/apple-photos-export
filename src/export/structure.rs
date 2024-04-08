use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use derive_new::new;

use crate::model::album::Album;
use crate::model::asset::ExportAsset;

pub trait OutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &ExportAsset) -> PathBuf;
}


#[derive(new)]
pub struct PlainOutputStructureStrategy;

impl OutputStructureStrategy for PlainOutputStructureStrategy {

    fn get_relative_output_dir(&self, _: &ExportAsset) -> PathBuf {
        PathBuf::new()
    }
}


// FIXME: Getting all albums upon creation and then recursively traversing them is not efficient!
#[derive(new)]
pub struct AlbumOutputStructureStrategy {
    flatten: bool,
    albums_by_id: HashMap<i32, Album>,
}

impl OutputStructureStrategy for AlbumOutputStructureStrategy {

    // TODO: Use result
    fn get_relative_output_dir(&self, asset: &ExportAsset) -> PathBuf {
        let mut path = PathBuf::new();

        if let Some(a) = asset.album.clone() {
            let album_path = a.get_relative_path(&self.albums_by_id).unwrap();
            path = path.join(album_path)
        }

        if self.flatten {
            let filename = path.file_name().unwrap_or(OsStr::new(""));
            PathBuf::new().join(filename)
        } else {
            path
        }
    }
}


type DateSelectorFunc = Box<dyn Fn(&ExportAsset) -> NaiveDateTime>;

pub struct YearMonthOutputStructureStrategy {
    date_selector: DateSelectorFunc
}

impl YearMonthOutputStructureStrategy {

    pub fn asset_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            date_selector: Box::new(|asset| asset.date)
        }
    }

    pub fn album_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            date_selector: Box::new(|asset| {
                match asset.album.clone() {
                    None => asset.date,
                    Some(album) => album.start_date.unwrap_or(asset.date)
                }
            })
        }
    }
}

impl OutputStructureStrategy for YearMonthOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &ExportAsset) -> PathBuf {
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
    fn get_relative_output_dir(&self, asset: &ExportAsset) -> PathBuf {
        self.strategies
            .iter()
            .fold(PathBuf::new(), |path, strategy| {
                let dir = strategy.get_relative_output_dir(asset);
                path.join(dir)
            })
    }
}