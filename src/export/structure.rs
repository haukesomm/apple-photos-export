use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use derive_new::new;

use crate::db::model::album::Album;
use crate::db::model::asset::Asset;
use crate::foundation::cocoa;

pub trait OutputStructureStrategy {

    fn get_relative_output_dir(&self, asset: &Asset, album: &Option<Album>) -> PathBuf;
}


#[derive(new)]
pub struct PlainOutputStructureStrategy;

impl OutputStructureStrategy for PlainOutputStructureStrategy {

    fn get_relative_output_dir(&self, _: &Asset, _: &Option<Album>) -> PathBuf {
        PathBuf::new()
    }
}


// FIXME: Getting all albums upon creation and then recursively traversing them is not efficient!
#[derive(new)]
pub struct AlbumOutputStructureStrategy {
    flatten: bool,
    albums_by_id: HashMap<i32, Album>,
}

impl AlbumOutputStructureStrategy {

    fn get_path_recursively(&self, album_id: i32) -> PathBuf {
        let album = self.albums_by_id.get(&album_id)
            .expect(format!("Album not in map: {}", album_id).as_str());

        match album.parent_id {
            None => {
                let mut buffer = PathBuf::new();
                if let Some(name) = &album.name {
                    buffer.push(name);
                }
                buffer
            },
            Some(parent_id) => {
                let path = self.get_path_recursively(parent_id);
                path.join(album.name.clone().unwrap_or(String::from("unnamed")))
            }
        }
    }
}

impl OutputStructureStrategy for AlbumOutputStructureStrategy {

    fn get_relative_output_dir(&self, _: &Asset, album: &Option<Album>) -> PathBuf {
        let mut path = PathBuf::new();

        if let Some(a) = album {
            let album_path = self.get_path_recursively(a.id);
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


type DateSelectorFunc = Box<dyn Fn(&Asset, &Option<Album>) -> f32>;

pub struct YearMonthOutputStructureStrategy {
    date_selector: DateSelectorFunc
}

impl YearMonthOutputStructureStrategy {

    pub fn asset_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            date_selector: Box::new(|asset, _| asset.date)
        }
    }

    pub fn album_date_based() -> YearMonthOutputStructureStrategy {
        YearMonthOutputStructureStrategy {
            date_selector: Box::new(|asset, album| {
                match album {
                    None => asset.date,
                    Some(album) => album.start_date.unwrap_or(asset.date)
                }
            })
        }
    }
}

impl OutputStructureStrategy for YearMonthOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &Asset, album: &Option<Album>) -> PathBuf {
        let datetime_raw = (self.date_selector)(asset, album);
        let datetime = cocoa::parse_cocoa_timestamp(datetime_raw);
        let formatted = format!("{}", datetime.format("%Y/%m/"));
        PathBuf::new().join(formatted)
    }
}


#[derive(new)]
pub struct JoiningOutputStructureStrategy {
    strategies: Vec<Box<dyn OutputStructureStrategy>>
}

impl OutputStructureStrategy for JoiningOutputStructureStrategy {
    fn get_relative_output_dir(&self, asset: &Asset, album: &Option<Album>) -> PathBuf {
        self.strategies
            .iter()
            .fold(PathBuf::new(), |path, strategy| {
                let dir = strategy.get_relative_output_dir(asset, album);
                path.join(dir)
            })
    }
}