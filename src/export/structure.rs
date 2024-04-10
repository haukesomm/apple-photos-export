use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use derive_new::new;

use crate::model::album::Album;
use crate::model::asset::ExportAsset;


pub trait OutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String>;
}


#[derive(new)]
pub struct PlainOutputStructureStrategy;

impl OutputStructureStrategy for PlainOutputStructureStrategy {

    fn get_relative_output_dir(&self, _: &ExportAsset) -> Result<PathBuf, String> {
        Ok(PathBuf::new())
    }
}


#[derive(new)]
pub struct AlbumOutputStructureStrategy {
    flatten: bool,
    albums_by_id: HashMap<i32, Album>,
}

impl OutputStructureStrategy for AlbumOutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        let path = match asset.album.clone() {
            None => PathBuf::new(),
            Some(a) => {
                if self.flatten {
                    PathBuf::from(a.name.unwrap_or(String::from("unnamed")))
                } else {
                    a.get_path(&self.albums_by_id)?
                }
            }
        };
        Ok(path)
    }
}


type DateSelectorFunc = Box<dyn Fn(&ExportAsset) -> NaiveDateTime>;

pub struct YearMonthOutputStructureStrategy {
    datetime_selector: DateSelectorFunc
}

impl YearMonthOutputStructureStrategy {

    pub fn asset_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            datetime_selector: Box::new(|asset| asset.datetime)
        }
    }

    pub fn album_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            datetime_selector: Box::new(|asset| {
                match asset.album.clone() {
                    None => asset.datetime,
                    Some(album) => album.start_date.unwrap_or(asset.datetime)
                }
            })
        }
    }
}

impl OutputStructureStrategy for YearMonthOutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        let datetime = (self.datetime_selector)(asset);
        let formatted = format!("{}", datetime.format("%Y/%m/"));
        Ok(PathBuf::from(formatted))
    }
}


#[derive(new)]
pub struct JoiningOutputStructureStrategy {
    strategies: Vec<Box<dyn OutputStructureStrategy>>
}

impl OutputStructureStrategy for JoiningOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        self.strategies
            .iter()
            .try_fold(PathBuf::new(), |path, strategy| {
                let dir = strategy.get_relative_output_dir(asset)?;
                Ok(path.join(dir))
            })
    }
}