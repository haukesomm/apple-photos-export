use crate::export::task::mapping::{MapAsset, MapExportTask, TaskMapperResult};
use crate::export::task::{AssetMapping, ExportTask};
use crate::model::album::Album;
use crate::model::asset::DataStoreSubtype;
use chrono::Datelike;
use derive_new::new;
use log::error;
use soft_canonicalize::soft_canonicalize;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::rc::Rc;

/// A mapper that excludes hidden assets from the export.
pub struct ExcludeHidden;

impl MapExportTask for ExcludeHidden {
    fn map_export_task(&self, task: ExportTask) -> TaskMapperResult {
        if let ExportTask::Copy(m) = &task {
            if m.asset.hidden {
                return TaskMapperResult::Remove;
            }
        }
        TaskMapperResult::Map(task)
    }
}

/// A mapper that prefixes the destination path with "_hidden" for hidden assets.
pub struct PrefixHidden;

impl MapAsset for PrefixHidden {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        if mapping.asset.hidden {
            AssetMapping {
                destination: PathBuf::from("_hidden").join(&mapping.destination),
                ..mapping
            }
        } else {
            mapping
        }
    }
}

/// A mapper that appends `.original` or `.derivate` to the destination file name based on whether
/// the asset is a derivative or not.
pub struct MarkOriginalsAndDerivates;

impl MapAsset for MarkOriginalsAndDerivates {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let mut dest = mapping.destination;
        let ext = String::from(
            dest.extension()
                .unwrap_or(&OsStr::new(""))
                .to_string_lossy(),
        );

        dest.set_extension(if mapping.is_derivate {
            format!("derivate.{}", ext)
        } else {
            format!("original.{}", ext)
        });

        AssetMapping {
            destination: dest,
            ..mapping
        }
    }
}

/// A mapper that restores the original file name of the asset in the destination path.
pub struct RestoreOriginalFilenames;

impl MapAsset for RestoreOriginalFilenames {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let original_extension = mapping.destination.extension().clone();

        let mut destination = PathBuf::from(&mapping.destination);

        destination.set_file_name(&mapping.asset.original_filename);
        // Restore original extension or remove it if the original destination did not have one
        destination.set_extension(&original_extension.unwrap_or(OsStr::new("")));

        AssetMapping {
            destination,
            ..mapping
        }
    }
}

/// A mapper that groups assets by album.
pub struct GroupByAlbum<'a> {
    albums: &'a HashMap<i32, Album>,
    max_depth: u8,
}

impl<'a> GroupByAlbum<'a> {
    pub fn flat(albums: &'a HashMap<i32, Album>) -> Self {
        Self {
            albums,
            max_depth: 1,
        }
    }

    pub fn recursive(albums: &'a HashMap<i32, Album>) -> Self {
        Self {
            albums,
            max_depth: 255,
        }
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

impl<'a> MapAsset for GroupByAlbum<'a> {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        if let Some(album_id) = mapping.album_id {
            let album_path = self.build_album_path_recursively(album_id, self.max_depth);
            AssetMapping {
                destination: PathBuf::from(album_path).join(&mapping.destination),
                ..mapping
            }
        } else {
            mapping
        }
    }
}

/// A mapper that groups assets by year and month.
pub struct GroupByYearAndMonth;

impl MapAsset for GroupByYearAndMonth {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let mut prefix = PathBuf::new();
        prefix.push(mapping.asset.datetime.year().to_string());
        prefix.push(format!("{:>02}", mapping.asset.datetime.month()));

        AssetMapping {
            destination: PathBuf::from(prefix).join(&mapping.destination),
            ..mapping
        }
    }
}

/// A mapper that groups assets by year, month, and album.
#[derive(new)]
pub struct GroupByYearMonthAndAlbum<'a> {
    albums: &'a HashMap<i32, Album>,
}

impl<'a> MapAsset for GroupByYearMonthAndAlbum<'a> {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let fallback = GroupByYearAndMonth {};

        match &mapping.album_id {
            None => fallback.map_asset(mapping),
            Some(album_id) => {
                if let Some(album) = self.albums.get(&album_id) {
                    let mut prefix = PathBuf::new();
                    if let Some(date) = album.start_date {
                        prefix.push(date.year().to_string());
                        prefix.push(format!("{:>02}", date.month()))
                    }

                    AssetMapping {
                        destination: PathBuf::from(prefix).join(&mapping.destination),
                        ..mapping
                    }
                } else {
                    mapping
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
    fn map_export_task(&self, task: ExportTask) -> TaskMapperResult {
        if let ExportTask::Copy(AssetMapping {
            album_id: Some(album_id),
            ..
        }) = &task
        {
            let matches_filter = self.ids.contains(&album_id);

            let include = match self.mode {
                AlbumFilterMode::Include => matches_filter,
                AlbumFilterMode::Exclude => !matches_filter,
            };

            if !include {
                return TaskMapperResult::Remove;
            }
        }
        TaskMapperResult::Map(task)
    }
}

/// A mapper that creates one export task per album the asset is part of.
///
/// This is needed because an asset can be part of multiple albums, but the export task
/// structure only maps one source to one destination. This mapper splits the task
/// into multiple tasks, one for each album the asset is part of.
pub struct OneTaskPerAlbum;

// TODO Fixme
impl MapExportTask for OneTaskPerAlbum {
    fn map_export_task(&self, task: ExportTask) -> TaskMapperResult {
        if let ExportTask::Copy(m) = &task {
            if m.album_id.is_none() && !m.asset.album_ids.is_empty() {
                let mut tasks: Vec<ExportTask> = vec![];

                for album_id in &m.asset.album_ids {
                    tasks.push(ExportTask::Copy(AssetMapping {
                        album_id: Some(album_id.clone()),
                        ..m.clone()
                    }))
                }

                return TaskMapperResult::Split(tasks);
            }
        }

        TaskMapperResult::Map(task)
    }
}

/// A mapper that converts the destination path to an absolute path using the given output directory.
pub struct ConvertToAbsolutePath {
    output_dir: PathBuf,
}

impl ConvertToAbsolutePath {
    pub fn new<P: Into<PathBuf>>(output_dir: P) -> ConvertToAbsolutePath {
        Self {
            output_dir: output_dir.into(),
        }
    }
}

impl MapAsset for ConvertToAbsolutePath {
    fn map_asset(&self, task: AssetMapping) -> AssetMapping {
        let absolute_path = self.output_dir.join(&task.destination);

        // Try to canonicalize paths in order to be able to compare them across multiple file
        // systems, e.g. when working with mounted SAMBA shares in combination with the --skip or
        // --delete flags.
        let destination = soft_canonicalize(&absolute_path).unwrap_or_else(|_| {
            error!(
                "Unable to canonicalize path!: {}",
                task.destination.to_string_lossy()
            );
            self.output_dir.clone()
        });

        AssetMapping {
            destination,
            ..task
        }
    }
}

#[derive(new)]
pub struct RemoveFromCacheIfExists {
    output_dir_files: Rc<RefCell<HashSet<PathBuf>>>,
}

impl MapAsset for RemoveFromCacheIfExists {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let destination: &PathBuf = &mapping.destination;

        let mut output_dir_files = self.output_dir_files.borrow_mut();
        output_dir_files.remove(destination);

        mapping
    }
}

#[derive(new)]
pub struct SkipIfExists {
    output_dir_files: Rc<RefCell<HashSet<PathBuf>>>,
}

impl MapAsset for SkipIfExists {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let destination: &PathBuf = &mapping.destination;

        let output_dir_files = self.output_dir_files.borrow_mut();
        if output_dir_files.contains(destination) {
            AssetMapping {
                skip: true,
                ..mapping
            }
        } else {
            mapping
        }
    }
}

#[derive(new)]
pub struct IncludeAssociatedRawImage<'a> {
    db_connection: &'a rusqlite::Connection,
}

impl<'a> MapExportTask for IncludeAssociatedRawImage<'a> {
    fn map_export_task(&self, task: ExportTask) -> TaskMapperResult {
        // This monstrosity acts as an important guard:
        let mapping = match &task {
            // If a task is going to be deleted anyway, it is not for this mapper.
            ExportTask::Delete(_) => return TaskMapperResult::Map(task),

            // A copy task is also omitted if:
            // 1) It is a derivate (only the primary asset can have an associated raw image)
            // 2) It has already been mapped by this task and thus is marked as a raw-pair
            // 3) It does not have an associated raw image to begin with
            ExportTask::Copy(m)
                if m.is_derivate
                    || m.is_part_of_raw_pair
                    || !m.asset.has_associated_raw_image() =>
            {
                return TaskMapperResult::Map(task)
            }

            // In all other cases, we may proceed to extract and modify the mapping
            ExportTask::Copy(m) => m.clone(),
        };

        let raw_image_uti = {
            let result = crate::db::asset::get_data_store_subtype_uti(
                self.db_connection,
                mapping.asset.id,
                DataStoreSubtype::ASSOCIATED_RAW_IMAGE,
            );
            if let Err(error) = result {
                error!(
                    "Could not get the associated raw image's UTI for asset '{}' despite the \
                    database indicating it has one! The respective asset pair will be ignored! \
                    Error: {}",
                    mapping.asset.id, error
                );
                return TaskMapperResult::Remove;
            }
            result.unwrap()
        };

        let raw_source = {
            let mut source = PathBuf::from(&mapping.source);

            if let Some(filename) = mapping.source.file_stem() {
                let mut raw_file_filename = OsString::new();
                raw_file_filename.push(filename);
                raw_file_filename.push(crate::model::library::file_suffixes::ASSOCIATED_RAW_IMAGE);

                source.set_file_name(raw_file_filename);
                source.set_extension(raw_image_uti.ext);
            } else {
                error!(
                    "The source file under '{}' appears to have no file name! This should not be \
                    possible. The respective asset pair will be ignored!",
                    mapping.source.to_string_lossy()
                );
                return TaskMapperResult::Remove;
            }

            source
        };

        let mut raw_destination = PathBuf::from(&mapping.destination);
        raw_destination.set_extension(raw_image_uti.ext);

        TaskMapperResult::Split(vec![
            ExportTask::Copy(AssetMapping {
                is_part_of_raw_pair: true,
                ..mapping.clone()
            }),
            ExportTask::Copy(AssetMapping {
                is_part_of_raw_pair: true,
                source: raw_source,
                destination: raw_destination,
                ..mapping
            }),
        ])
    }
}
