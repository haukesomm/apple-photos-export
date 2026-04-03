//! This module contains the mapping logic used to detect and export associated RAW files (also
//! called RAW-pairs) of an asset.

use crate::export::task::mapping::{MapExportTask, TaskMapperResult};
use crate::export::task::{AssetMapping, ExportTask};
use crate::model::asset::DataStoreSubtype;
use crate::uti::Uti;
use derive_new::new;
use log::error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(new)]
pub struct IncludeAssociatedRawImage<'a> {
    db_connection: &'a rusqlite::Connection,
}

impl<'a> MapExportTask for IncludeAssociatedRawImage<'a> {
    fn map_export_task(&self, task: ExportTask) -> TaskMapperResult {
        // General guard - only process tasks that have unprocessed RAW pairs
        let mapping = 'guard: {
            if let ExportTask::Copy(m) = &task {
                if can_extract_raw_image(m) {
                    break 'guard m;
                }
            }
            return TaskMapperResult::Map(task);
        };

        // Try to determine the associated RAW pairs UTI
        let raw_image_uti =
            if let Ok(uti) = get_associated_raw_file_uti(self.db_connection, mapping.asset.id) {
                uti
            } else {
                return TaskMapperResult::Remove;
            };

        // Construct the path of the associated RAW image
        let mut raw_file_source = PathBuf::from(&mapping.source);
        raw_file_source.set_file_name(get_associated_raw_file_name(&mapping.source));
        raw_file_source.set_extension(raw_image_uti.ext);

        // Split the original task into two distinct ones: One for the original and for it's
        // associated RAW image.
        // Both are marked as a RAW pair so they won't be processed again, potentially resulting in
        // an infinite loop if mappers are applied recursively.
        TaskMapperResult::Split(vec![
            ExportTask::Copy(AssetMapping {
                is_part_of_raw_pair: true,
                ..mapping.clone()
            }),
            ExportTask::Copy(AssetMapping {
                is_part_of_raw_pair: true,
                source: raw_file_source,
                file_extension: raw_image_uti.ext.to_string(),
                ..mapping.clone()
            }),
        ])
    }
}

/// This helper function decides whether an `AssetMapping` has an exportable associated RAW image.
///
/// Whether this is the case depends on a number of criteria:
/// 1) The asset must not be a derivate (only original assets can have an associated RAW image)
/// 2) The mapping must not have been marked as processed by the RAW file mapper before
/// 3) The asset must have an associated RAW image to begin with.
#[inline(always)]
fn can_extract_raw_image(mapping: &AssetMapping) -> bool {
    !mapping.is_derivate && !mapping.is_part_of_raw_pair && mapping.asset.has_associated_raw_image()
}

fn get_associated_raw_file_uti(conn: &rusqlite::Connection, asset_id: i32) -> crate::Result<Uti> {
    let result = crate::db::asset::get_data_store_subtype_uti(
        conn,
        asset_id,
        DataStoreSubtype::ASSOCIATED_RAW_IMAGE,
    );
    if let Err(error) = &result {
        error!(
            "Could not get the associated raw image's UTI for asset '{}' despite the \
            database indicating it has one! The respective asset pair will be ignored! \
            Error: {}",
            asset_id, error
        );
    };
    result
}

fn get_associated_raw_file_name(original_file: &Path) -> OsString {
    let mut filename = original_file
        .file_stem()
        .map(|s| s.to_owned())
        .unwrap_or(OsString::new());

    filename.push(crate::model::library::file_suffixes::ASSOCIATED_RAW_IMAGE);

    filename
}
