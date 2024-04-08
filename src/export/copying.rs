use std::fs::{copy, create_dir_all};
use std::path::Path;

use colored::Colorize;
use derive_new::new;

pub enum FinishState {
    Success(usize),
    Failure(Vec<String>),
}

pub trait AssetCopyStrategy {

    fn copy_asset(&self, src: &Path, dest: &Path) -> Result<(), String>;

    fn on_finish(&self, state: FinishState);
}


#[derive(new)]
pub struct DryRunAssetCopyStrategy;

impl AssetCopyStrategy for DryRunAssetCopyStrategy {

    fn copy_asset(&self, _: &Path, _: &Path) -> Result<(), String> {
        // do nothing - dry run
        Ok(())
    }

    fn on_finish(&self, _: FinishState) {
        println!("{}", "Done. This was a dry run - no files have been exported and all errors have \
        been ignored.".magenta())
    }
}


#[derive(new)]
pub struct DefaultAssetCopyStrategy;

impl AssetCopyStrategy for DefaultAssetCopyStrategy {

    fn copy_asset(&self, src: &Path, dest: &Path) -> Result<(), String> {
        if let Some(parent) = dest.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("Error creating directory: {}", e))?;
        }
        copy(src, dest)
            .map(|_| ())
            .map_err(|e| format!("Error copying file: {}", e))
    }

    fn on_finish(&self, state: FinishState) {
        match state {
            FinishState::Success(counts) => {
                println!(
                    "{} {} assets have been exported successfully!",
                    "Success:".green(),
                    counts,
                );
            },
            FinishState::Failure(paths) => {
                println!(
                    "{} {} assets could not be exported!",
                    "Error:".red(),
                    paths.len(),
                );
                for path in paths {
                    println!("Error exporting asset: {}", path);
                }
            }
        }
    }
}