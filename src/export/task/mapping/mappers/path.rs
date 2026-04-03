//! This module contains mappers that modify an export task's output path, i.e. converting to
//! absolute paths or moving hidden assets into a dedicated directory.

use crate::export::task::mapping::MapAsset;
use crate::export::task::AssetMapping;
use std::path::PathBuf;

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
