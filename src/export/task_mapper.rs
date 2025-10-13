use crate::export::{ExportAssetRelation, ExportTask};
use crate::model::album::Album;
use chrono::Datelike;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use derive_new::new;


/// A trait for mapping export tasks.
/// 
/// This trait is used to transform an `ExportTask` into another `ExportTask` or filter it out.
/// It is used in the export process to apply various transformations to the tasks before they are
/// executed, e.g. to group them by album, year, or month, or to exclude hidden assets.
/// 
/// Upon export, all registered mappers are called in the order they were registered.
pub trait MapExportTask {
    fn map(&self, task: ExportTask) -> Option<ExportTask>;
}


/// A mapper that excludes hidden assets from the export.
#[derive(new)]
pub struct ExcludeHidden;

impl MapExportTask for ExcludeHidden {
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        if task.asset.hidden {
            None
        } else {
            Some(task)
        }
    }
}


/// A mapper that prefixes the destination path with "_hidden" for hidden assets.
#[derive(new)]
pub struct PrefixHidden;

impl MapExportTask for PrefixHidden {
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        Some(if task.asset.hidden {
            ExportTask {
                destination: PathBuf::from("_hidden").join(&task.destination),
                ..task
            }
        } else {
            task
        })
    }
}


/// A mapper that appends `.original` or `.derivate` to the destination file name based on whether
/// the asset is a derivative or not.
#[derive(new)]
pub struct MarkOriginalsAndDerivates;

impl MapExportTask for MarkOriginalsAndDerivates {
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        let mut dest = task.destination;
        let ext = String::from(
            dest.extension()
                .unwrap_or(&OsStr::new(""))
                .to_string_lossy(),
        );

        dest.set_extension(if task.meta.derivate {
            format!("derivate.{}", ext)
        } else {
            format!("original.{}", ext)
        });

        Some(ExportTask {
            destination: dest,
            ..task
        })
    }
}


/// A mapper that restores the original file name of the asset in the destination path.
#[derive(new)]
pub struct RestoreOriginalFilenames;

impl MapExportTask for RestoreOriginalFilenames {
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        let original_extension = task.destination.extension().clone();

        let mut destination = PathBuf::from(&task.destination);

        destination.set_file_name(&task.asset.original_filename);
        // Restore original extension or remove it if the original destination did not have one
        destination.set_extension(&original_extension.unwrap_or(OsStr::new("")));

        Some(ExportTask {
            destination,
            ..task
        })
    }
}


/// A mapper that groups assets by album.
pub struct GroupByAlbum<'a> {
    albums: &'a HashMap<i32, Album>,
    max_depth: u8
}

impl<'a> GroupByAlbum<'a> {
    
    pub fn flat(albums: &'a HashMap<i32, Album>) -> Self {
        Self { albums, max_depth: 1 }
    }
    
    pub fn recursive(albums: &'a HashMap<i32, Album>) -> Self {
        Self { albums, max_depth: 255 }
    }

    fn build_album_path_recursively(&self, id: i32, depth: u8) -> PathBuf {
        let album_optional = self.albums.get(&id);

        if depth == 0 || album_optional.is_none() || album_optional.unwrap().parent_id.is_none() {
            return PathBuf::new();
        }

        let album = album_optional.unwrap();
        let parent = self.build_album_path_recursively(album.parent_id.unwrap(), depth - 1);

        parent.join(album.name.clone().unwrap_or("_unknown_".to_string()))
    }
}

impl<'a> MapExportTask for GroupByAlbum<'a> {
    
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        if let ExportAssetRelation::AlbumMember { album_id, .. } = task.meta.relation {
            let album_path = self.build_album_path_recursively(album_id, self.max_depth);
            Some(ExportTask {
                destination: PathBuf::from(album_path).join(&task.destination),
                ..task
            })
        } else {
            Some(task)
        }
    }
}


/// A mapper that groups assets by year and month.
#[derive(new)]
pub struct GroupByYearAndMonth;

impl MapExportTask for GroupByYearAndMonth {
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        let mut prefix = PathBuf::new();
        prefix.push(task.asset.datetime.year().to_string());
        prefix.push(format!("{:>02}", task.asset.datetime.month()));

        Some(ExportTask {
            destination: PathBuf::from(prefix).join(&task.destination),
            ..task
        })
    }
}


/// A mapper that groups assets by year, month, and album.
#[derive(new)]
pub struct GroupByYearMonthAndAlbum<'a> {
    albums: &'a HashMap<i32, Album>,
}

impl<'a> MapExportTask for GroupByYearMonthAndAlbum<'a> {
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        let fallback = GroupByYearAndMonth {};

        match &task.meta.relation {
            ExportAssetRelation::None => fallback.map(task),
            ExportAssetRelation::AlbumMember { album_id, .. } => {
                let album = self.albums.get(&album_id)?;

                let mut prefix = PathBuf::new();
                if let Some(date) = album.start_date {
                    prefix.push(date.year().to_string());
                    prefix.push(format!("{:>02}", date.month()))
                }

                Some(ExportTask {
                    destination: PathBuf::from(prefix).join(&task.destination),
                    ..task
                })
            }
        }
    }
}


pub enum AlbumFilterMode {
    Include,
    Exclude,
}

/// A mapper that filters assets by album ID.
#[derive(new)]
pub struct FilterByAlbumId {
    ids: Vec<i32>,
    mode: AlbumFilterMode,
}

impl MapExportTask for FilterByAlbumId {
    fn map(&self, task: ExportTask) -> Option<ExportTask> {
        let matches_filter = if let ExportAssetRelation::AlbumMember { album_id, .. } = task.meta.relation {
            self.ids.contains(&album_id)
        } else {
            false
        };

        let include = match self.mode {
            AlbumFilterMode::Include => matches_filter,
            AlbumFilterMode::Exclude => !matches_filter,
        };

        if include {
            Some(task)
        } else {
            None
        }
    }
}
