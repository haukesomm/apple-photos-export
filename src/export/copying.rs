use std::fs::{copy, create_dir_all};
use std::path::Path;

use colored::Colorize;
use derive_new::new;

pub enum FinishState {

    /// Represents a success state with the count of assets that have been exported.
    Success(usize),

    /// Represents a failure state with the total count of assets that could not be exported and
    /// a list of error messages.
    Failure(i64, Vec<String>),
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
        println!("{}", "Done. This was a dry run - no files have been exported and all potential \
        errors have been ignored.".magenta())
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
            .map_err(|e| e.to_string())
    }

    fn on_finish(&self, state: FinishState) {
        match state {
            FinishState::Success(total_count) => {
                println!(
                    "{} {} assets have been exported successfully!",
                    "Success:".green(),
                    total_count,
                );
            }
            FinishState::Failure(total_count, messages) => {
                for message in &messages {
                    println!(
                        "{} {}",
                        "Error exporting asset:".red(),
                        message
                    );
                }
                println!(
                    "{} {} of {} assets could not be exported (see messages above)!",
                    "Error:".red(),
                    messages.len(),
                    total_count,
                );
            }
        }
    }
}