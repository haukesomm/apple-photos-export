//! This module contains the mapping logic used to detect and export associated RAW files (also
//! called RAW-pairs) of an asset.

use crate::export::task::mapping::{MapExportTask, TaskMapperResult};
use crate::export::task::{AssetMapping, ExportTask};
use crate::model::asset::DataStoreSubtype;
use derive_new::new;
use log::error;
use std::ffi::OsString;
use std::path::PathBuf;

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
