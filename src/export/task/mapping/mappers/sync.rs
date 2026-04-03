//! This module contains the output directory syncing logic of the `--skip` and `--delete` flags.

use crate::export::task::mapping::MapAsset;
use crate::export::task::{AssetMapping, ExportTask};
use crate::fs;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone)]
pub struct OutputFileTrackingAssetMapper {
    output_dir: PathBuf,
    files_to_remove: Rc<RefCell<HashMap<String, PathBuf>>>,
    ignored_filenames: HashSet<String>,
    skip_existing_tasks: bool,
}

impl OutputFileTrackingAssetMapper {
    pub fn new<P: Into<PathBuf>>(output_dir: P, skip_existing_tasks: bool) -> Self {
        Self {
            output_dir: output_dir.into(),
            files_to_remove: Rc::new(RefCell::new(HashMap::new())),
            ignored_filenames: HashSet::from([".DS_Store".to_string()]),
            skip_existing_tasks,
        }
    }

    pub fn initialize(&self) -> crate::Result<()> {
        let mut files = self.files_to_remove.borrow_mut();

        fs::recursively_visit_files(&self.output_dir, &mut |entry| {
            if let Some(name) = &entry.file_name() {
                if !self
                    .ignored_filenames
                    .contains(&name.to_string_lossy().to_string())
                {
                    files.insert(self.get_normalized_unicode_key(&entry)?, entry);
                }
            }
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
