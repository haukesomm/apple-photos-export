//! This module contains dedicated mappers to include and exclude assets based on different criteria
//! such as their album ids.

use crate::export::task::ExportTask;
use crate::export::task::mapping::{MapExportTask, TaskMapperResult};
use derive_new::new;

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
        if let ExportTask::Copy(m) = &task {
            return match m.album_id {
                // A mapping with a concrete album id: keep or drop based on
                // whether that album matches the filter set.
                Some(album_id) => {
                    let matches_filter = self.ids.contains(&album_id);
                    let keep = match self.mode {
                        AlbumFilterMode::Include => matches_filter,
                        AlbumFilterMode::Exclude => !matches_filter,
                    };
                    if keep {
                        TaskMapperResult::Map(task)
                    } else {
                        TaskMapperResult::Remove
                    }
                }
                // A mapping without an album id has not been split into
                // per-album copies (e.g. the asset is not part of any album,
                // or OneTaskPerAlbum has not run). Its membership cannot be
                // decided by inspecting a single album, so handle it per mode:
                //   - Include: an asset in no album is not in any included
                //     album, so drop it.
                //   - Exclude: an asset in no album is not in any excluded
                //     album, so keep it.
                None => match self.mode {
                    AlbumFilterMode::Include => TaskMapperResult::Remove,
                    AlbumFilterMode::Exclude => TaskMapperResult::Map(task),
                },
            };
        }
        TaskMapperResult::Map(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::task::AssetMapping;
    use crate::model::asset::Asset;
    use crate::uti::Uti;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn task(album_ids: &[i32], album_id: Option<i32>) -> ExportTask {
        ExportTask::Copy(AssetMapping {
            asset: Asset {
                id: 1,
                uuid: "uuid".to_string(),
                dir: "directory".to_string(),
                filename: "image.jpeg".to_string(),
                derivate_uti: Uti::JPEG,
                datetime: chrono::DateTime::UNIX_EPOCH.naive_utc(),
                hidden: false,
                original_filename: "image.jpeg".to_string(),
                has_adjustments: false,
                data_store_subtypes: vec![],
                album_ids: HashSet::from_iter(album_ids.iter().copied()),
            },
            source: PathBuf::from("image.jpeg"),
            destination_dir: PathBuf::new(),
            filename_components: vec!["image".to_string()],
            file_extension: "jpeg".to_string(),
            is_derivate: false,
            album_id,
            skip: false,
            is_part_of_raw_pair: false,
        })
    }

    #[test]
    fn includes_matching_album_copy_in_include_mode() {
        let mapper = FilterByAlbumId::new(vec![42], AlbumFilterMode::Include);
        assert!(matches!(
            mapper.map_export_task(task(&[7, 42], Some(42))),
            TaskMapperResult::Map(_)
        ));
    }

    #[test]
    fn removes_non_matching_album_copy_in_include_mode() {
        let mapper = FilterByAlbumId::new(vec![42], AlbumFilterMode::Include);
        assert!(matches!(
            mapper.map_export_task(task(&[7, 42], Some(7))),
            TaskMapperResult::Remove
        ));
    }

    #[test]
    fn removes_matching_album_copy_in_exclude_mode() {
        let mapper = FilterByAlbumId::new(vec![42], AlbumFilterMode::Exclude);
        assert!(matches!(
            mapper.map_export_task(task(&[7, 42], Some(42))),
            TaskMapperResult::Remove
        ));
    }

    #[test]
    fn keeps_non_matching_album_copy_in_exclude_mode() {
        let mapper = FilterByAlbumId::new(vec![42], AlbumFilterMode::Exclude);
        assert!(matches!(
            mapper.map_export_task(task(&[7], Some(7))),
            TaskMapperResult::Map(_)
        ));
    }

    #[test]
    fn drops_mapping_without_album_id_in_include_mode() {
        // An asset in no album is not in any included album, so it must be
        // dropped. This is the case that made --include-by-album a silent
        // no-op for assets not part of any user album.
        let mapper = FilterByAlbumId::new(vec![42], AlbumFilterMode::Include);
        assert!(matches!(
            mapper.map_export_task(task(&[], None)),
            TaskMapperResult::Remove
        ));
    }

    #[test]
    fn keeps_mapping_without_album_id_in_exclude_mode() {
        // An asset in no album is not in any excluded album, so it is kept.
        let mapper = FilterByAlbumId::new(vec![42], AlbumFilterMode::Exclude);
        assert!(matches!(
            mapper.map_export_task(task(&[], None)),
            TaskMapperResult::Map(_)
        ));
    }
}
