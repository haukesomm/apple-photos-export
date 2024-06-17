use std::collections::HashMap;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use derive_new::new;

use crate::model::album::Album;
use crate::model::asset::ExportAsset;

pub trait OutputStrategy {

    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String>;
}


#[derive(new)]
pub struct PlainOutputStrategy;

impl OutputStrategy for PlainOutputStrategy {

    fn get_relative_output_dir(&self, _: &ExportAsset) -> Result<PathBuf, String> {
        Ok(PathBuf::new())
    }
}


#[derive(new)]
pub struct AlbumOutputStrategy {
    flatten: bool,
    albums_by_id: HashMap<i32, Album>,
}

impl OutputStrategy for AlbumOutputStrategy {

    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        let path = match &asset.album {
            None => PathBuf::new(),
            Some(a) => {
                if self.flatten {
                    PathBuf::from(a.name.clone().unwrap_or(String::from("unnamed")))
                } else {
                    a.get_path(&self.albums_by_id)?
                }
            }
        };
        Ok(path)
    }
}


type DateSelectorFunc = Box<dyn Fn(&ExportAsset) -> NaiveDateTime>;

pub struct YearMonthOutputStrategy {
    datetime_selector: DateSelectorFunc
}

impl YearMonthOutputStrategy {

    pub fn asset_date_based() -> YearMonthOutputStrategy {
        YearMonthOutputStrategy {
            datetime_selector: Box::new(|asset| asset.datetime)
        }
    }

    pub fn album_date_based() -> YearMonthOutputStrategy {
        YearMonthOutputStrategy {
            datetime_selector: Box::new(|asset| {
                match asset.album.clone() {
                    None => asset.datetime,
                    Some(album) => album.start_date.unwrap_or(asset.datetime)
                }
            })
        }
    }
}

impl OutputStrategy for YearMonthOutputStrategy {

    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        let datetime = (self.datetime_selector)(asset);
        let formatted = format!("{}", datetime.format("%Y/%m/"));
        Ok(PathBuf::from(formatted))
    }
}


#[derive(new)]
pub struct NestingOutputStrategyDecorator {
    strategies: Vec<Box<dyn OutputStrategy>>
}

impl OutputStrategy for NestingOutputStrategyDecorator {
    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        self.strategies
            .iter()
            .try_fold(PathBuf::new(), |path, strategy| {
                let dir = strategy.get_relative_output_dir(asset)?;
                Ok(path.join(dir))
            })
    }
}


#[derive(new)]
pub struct HiddenAssetHandlingOutputStrategyDecorator {
    strategy: Box<dyn OutputStrategy>
}

impl OutputStrategy for HiddenAssetHandlingOutputStrategyDecorator {
    fn get_relative_output_dir(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        let mut path = PathBuf::new();

        if asset.hidden {
            path.push("_hidden");
        }
        path.push(self.strategy.get_relative_output_dir(asset)?);

        Ok(path)
    }
}