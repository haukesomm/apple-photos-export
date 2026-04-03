//! This module contains the actual mappers used to create the export tasks.
//!
//! General mappers are part of this very module.
//! Specialized mappers are part of specialized submodules.

mod album;
pub mod filename;
pub mod filter;
pub mod path;
pub mod raw;
pub mod sync;
pub mod time;

pub use album::ByAlbum;
pub use raw::IncludeAssociatedRawImage;
pub use sync::OutputFileTrackingAssetMapper;
pub use time::ByYearAndMonth;

use crate::export::task::mapping::{MapExportTask, TaskMapperResult};
use crate::export::task::{AssetMapping, ExportTask};

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
