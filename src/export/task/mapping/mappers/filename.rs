//! This module contains file-name modifying mappers, e.g. such that restore an asset's original
//! filename.

use crate::export::task::AssetMapping;
use crate::export::task::mapping::MapAsset;
use std::path::PathBuf;

/// A mapper that restores the original file name of the asset in the destination path.
pub struct RestoreOriginalFilenames;

impl MapAsset for RestoreOriginalFilenames {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        AssetMapping {
            filename_components: vec![
                PathBuf::from(&mapping.asset.original_filename)
                    .file_stem()
                    .expect("Fatal: Encountered library asset without file stem!")
                    .to_string_lossy()
                    .to_string(),
            ],
            ..mapping
        }
    }
}

/// A mapper that includes the asset's id in the output filename.
///
/// This is useful in order to avoid conflicts if the original filenames are restored and multiple
/// files have the same filename.
pub struct IncludeAssetId;

impl MapAsset for IncludeAssetId {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        let mut clone = mapping.clone();
        clone.filename_components.push(mapping.asset.id.to_string());
        clone
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
