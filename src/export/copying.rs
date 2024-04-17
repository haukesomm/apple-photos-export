use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};

use colored::Colorize;
use derive_new::new;
use crate::export::structure::OutputStrategy;
use crate::model::asset::ExportAsset;


#[derive(new)]
pub struct CopyOperation {
    pub source_path: PathBuf,
    pub output_filename: String,
    pub output_filename_suffix: Option<String>,
    pub output_folder: Option<PathBuf>,
}

impl CopyOperation {
    pub fn get_output_path(&self) -> PathBuf {
        PathBuf::new()
            .join(self.output_folder.clone().unwrap_or(PathBuf::new()))
            .join(
                format!(
                    "{}{}",
                    self.output_filename,
                    self.output_filename_suffix.clone().unwrap_or("".to_string())
                )
            )
    }
}


pub trait CopyOperationFactory {
    fn build(&self, asset: &ExportAsset) -> Vec<CopyOperation>;
}

#[derive(new)]
pub struct OriginalsCopyOperationFactory;
impl CopyOperationFactory for OriginalsCopyOperationFactory {
    fn build(&self, asset: &ExportAsset) -> Vec<CopyOperation> {
        vec![CopyOperation::new(
            asset.path(),
            asset.filename.clone(),
            None,
            None,
        )]
    }
}

#[derive(new)]
pub struct FilenameRestoringCopyOperationFactoryDecorator {
    inner: Box<dyn CopyOperationFactory>,
}
impl CopyOperationFactory for FilenameRestoringCopyOperationFactoryDecorator {
    fn build(&self, asset: &ExportAsset) -> Vec<CopyOperation> {
        self.inner
            .build(asset)
            .into_iter()
            .map(|op| {
                CopyOperation::new(
                    op.source_path,
                    asset.original_filename.clone(),
                    op.output_filename_suffix,
                    op.output_folder,
                )
            })
            .collect()
    }
}

#[derive(new)]
pub struct OutputStructureCopyOperationFactoryDecorator {
    inner: Box<dyn CopyOperationFactory>,
    strategy: Box<dyn OutputStrategy>,
}
impl CopyOperationFactory for OutputStructureCopyOperationFactoryDecorator {
    fn build(&self, asset: &ExportAsset) -> Vec<CopyOperation> {
        self.inner
            .build(asset)
            .into_iter()
            .map(|op| {
                CopyOperation::new(
                    op.source_path,
                    op.output_filename,
                    op.output_filename_suffix,
                    self.strategy.get_relative_output_dir(asset).ok(),
                )
            })
            .collect()
    }
}


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