use crate::export::task::mapping::{MapAsset, MapExportTask, TaskMapperResult};
use crate::export::task::{AssetMapping, ExportTask};
use crate::fs;
use crate::model::album::Album;
use crate::model::asset::DataStoreSubtype;
use chrono::Datelike;
use derive_new::new;
use log::error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use unicode_normalization::UnicodeNormalization;

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
                destination_dir: PathBuf::from("_hidden").join(&mapping.destination_dir),
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
        let mut marked = mapping.clone();
        marked.filename_components.push(match mapping.is_derivate {
            true => "derivate".to_string(),
            false => "original".to_string(),
        });

        marked
    }
}

/// A mapper that restores the original file name of the asset in the destination path.
pub struct RestoreOriginalFilenames;

impl MapAsset for RestoreOriginalFilenames {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        AssetMapping {
            filename_components: vec![PathBuf::from(&mapping.asset.original_filename)
                .file_stem()
                .expect("Fatal: Encountered library asset without file stem!")
                .to_string_lossy()
                .to_string()],
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
                destination_dir: PathBuf::from(album_path).join(&mapping.destination_dir),
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
            destination_dir: PathBuf::from(prefix).join(&mapping.destination_dir),
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
                        destination_dir: PathBuf::from(prefix).join(&mapping.destination_dir),
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
        AssetMapping {
            destination_dir: self.output_dir.join(task.destination_dir),
            ..task
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

        TaskMapperResult::Split(vec![
            ExportTask::Copy(AssetMapping {
                is_part_of_raw_pair: true,
                ..mapping.clone()
            }),
            ExportTask::Copy(AssetMapping {
                is_part_of_raw_pair: true,
                source: raw_source,
                file_extension: raw_image_uti.ext.to_string(),
                ..mapping
            }),
        ])
    }
}

#[derive(Clone)]
pub struct OutputFileTrackingAssetMapper {
    output_dir: PathBuf,
    files_to_remove: Rc<RefCell<HashMap<String, PathBuf>>>,
    skip_existing_tasks: bool,
}

impl OutputFileTrackingAssetMapper {
    pub fn new<P: Into<PathBuf>>(output_dir: P, skip_existing_tasks: bool) -> Self {
        Self {
            output_dir: output_dir.into(),
            files_to_remove: Rc::new(RefCell::new(HashMap::new())),
            skip_existing_tasks,
        }
    }

    pub fn initialize(&self) -> crate::Result<()> {
        let mut files = self.files_to_remove.borrow_mut();

        fs::recursively_visit_files(&self.output_dir, &mut |entry| {
            files.insert(self.get_normalized_unicode_key(&entry)?, entry);
            Ok(())
        })?;

        Ok(())
    }

    pub fn create_delete_tasks_for_remaining_files(&self) -> Vec<ExportTask> {
        self.files_to_remove
            .borrow()
            .iter()
            .map(|(_, p)| ExportTask::Delete(PathBuf::from(&self.output_dir.join(p))))
            .collect()
    }

    pub(self) fn get_normalized_unicode_key(&self, absolute: &Path) -> crate::Result<String> {
        Ok(absolute
            .strip_prefix(&self.output_dir)
            .map_err(|_| format!("Failed to strip prefix of path: {:?}", absolute))?
            .to_path_buf()
            .to_string_lossy()
            .nfc()
            .to_string())
    }
}

impl MapAsset for OutputFileTrackingAssetMapper {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let relative = self
            .get_normalized_unicode_key(&mapping.destination_path())
            .unwrap();

        let file_exists = self
            .files_to_remove
            .borrow_mut()
            .remove(&relative)
            .is_some();

        if self.skip_existing_tasks && file_exists {
            AssetMapping {
                skip: true,
                ..mapping
            }
        } else {
            mapping
        }
    }
}
