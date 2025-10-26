use crate::export::ExportTask;
use crate::model::album::Album;
use chrono::Datelike;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use derive_new::new;


/// The result of mapping an export task.
/// 
/// This enum represents the possible outcomes of mapping an `ExportTask`:
/// 1. `Remove`: The task should be removed and not exported.
/// 2. `Map(ExportTask)`: The task has been transformed into a new `ExportTask`.
/// 3. `Split(Vec<ExportTask>)`: The task has been split into multiple `ExportTask`s.
/// 
/// This allows for flexible handling of export tasks, enabling filtering, transformation,
/// and splitting of tasks as needed.
pub enum TaskMapperResult {
    Remove,
    Map(ExportTask),
    Split(Vec<ExportTask>),
}


/// A trait for mapping export tasks.
/// 
/// This trait is used to transform an `ExportTask` into another `ExportTask` or filter it out.
/// It is used in the export process to apply various transformations to the tasks before they are
/// executed, e.g. to group them by album, year, or month, or to exclude hidden assets.
/// 
/// Upon export, all registered mappers are called in the order they were registered.
pub trait MapExportTask {
    fn map(&self, task: ExportTask) -> TaskMapperResult;
}


/// A mapper that excludes hidden assets from the export.
#[derive(new)]
pub struct ExcludeHidden;

impl MapExportTask for ExcludeHidden {
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        if task.asset.hidden {
            TaskMapperResult::Remove
        } else {
            TaskMapperResult::Map(task)
        }
    }
}


/// A mapper that prefixes the destination path with "_hidden" for hidden assets.
#[derive(new)]
pub struct PrefixHidden;

impl MapExportTask for PrefixHidden {
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        TaskMapperResult::Map(if task.asset.hidden {
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
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        let mut dest = task.destination;
        let ext = String::from(
            dest.extension()
                .unwrap_or(&OsStr::new(""))
                .to_string_lossy(),
        );

        dest.set_extension(if task.is_derivate {
            format!("derivate.{}", ext)
        } else {
            format!("original.{}", ext)
        });

        TaskMapperResult::Map(ExportTask {
            destination: dest,
            ..task
        })
    }
}


/// A mapper that restores the original file name of the asset in the destination path.
#[derive(new)]
pub struct RestoreOriginalFilenames;

impl MapExportTask for RestoreOriginalFilenames {
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        let original_extension = task.destination.extension().clone();

        let mut destination = PathBuf::from(&task.destination);

        destination.set_file_name(&task.asset.original_filename);
        // Restore original extension or remove it if the original destination did not have one
        destination.set_extension(&original_extension.unwrap_or(OsStr::new("")));

        TaskMapperResult::Map(ExportTask {
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
    
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        if let Some(album_id) = task.album_id {
            let album_path = self.build_album_path_recursively(album_id, self.max_depth);
            TaskMapperResult::Map(ExportTask {
                destination: PathBuf::from(album_path).join(&task.destination),
                ..task
            })
        } else {
            TaskMapperResult::Map(task)
        }
    }
}


/// A mapper that groups assets by year and month.
#[derive(new)]
pub struct GroupByYearAndMonth;

impl MapExportTask for GroupByYearAndMonth {
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        let mut prefix = PathBuf::new();
        prefix.push(task.asset.datetime.year().to_string());
        prefix.push(format!("{:>02}", task.asset.datetime.month()));

        TaskMapperResult::Map(ExportTask {
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
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        let fallback = GroupByYearAndMonth {};

        match &task.album_id {
            None => fallback.map(task),
            Some(album_id) => {
                if let Some(album) = self.albums.get(&album_id) {
                    let mut prefix = PathBuf::new();
                    if let Some(date) = album.start_date {
                        prefix.push(date.year().to_string());
                        prefix.push(format!("{:>02}", date.month()))
                    }

                    TaskMapperResult::Map(ExportTask {
                        destination: PathBuf::from(prefix).join(&task.destination),
                        ..task
                    })
                } else {
                    TaskMapperResult::Remove
                }
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
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        let matches_filter = if let Some(album_id) = task.album_id {
            self.ids.contains(&album_id)
        } else {
            false
        };

        let include = match self.mode {
            AlbumFilterMode::Include => matches_filter,
            AlbumFilterMode::Exclude => !matches_filter,
        };

        if include {
            TaskMapperResult::Map(task)
        } else {
            TaskMapperResult::Remove
        }
    }
}


/// A mapper that creates one export task per album the asset is part of.
/// 
/// This is needed because an asset can be part of multiple albums, but the export task
/// structure only maps one source to one destination. This mapper splits the task
/// into multiple tasks, one for each album the asset is part of.
#[derive(new)]
pub struct OneTaskPerAlbum;

impl MapExportTask for OneTaskPerAlbum {
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        if task.asset.album_ids.is_empty() || task.album_id.is_some() {
            return TaskMapperResult::Map(task);
        }
        
        let mut tasks: Vec<ExportTask> = vec![];
        
        for album_id in &task.asset.album_ids {
            tasks.push(
                ExportTask {
                    album_id: Some(album_id.clone()),
                    ..task.clone()
                }
            )
        }
        
        TaskMapperResult::Split(tasks)
    }
}

/// A mapper that converts the destination path to an absolute path using the given output directory.
pub struct ConvertToAbsolutePath {
    output_dir: PathBuf,
}

impl ConvertToAbsolutePath {
    pub fn new<P : Into<PathBuf>>(output_dir: P) -> ConvertToAbsolutePath {
        Self { output_dir: output_dir.into() }
    }
}

impl MapExportTask for ConvertToAbsolutePath {
    fn map(&self, task: ExportTask) -> TaskMapperResult {
        TaskMapperResult::Map(ExportTask {
            destination: self.output_dir.join(task.destination),
            ..task
        })
    }
}
