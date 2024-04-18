use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};

use colored::Colorize;
use derive_new::new;
use crate::export::structure::OutputStrategy;
use crate::model::asset::ExportAsset;
use crate::model::uti::Uti;


#[derive(new)]
pub struct CopyOperation {
    pub source_path: PathBuf,
    pub file_type: &'static Uti,
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
                    "{}{}.{}",
                    self.output_filename,
                    self.output_filename_suffix.clone().unwrap_or("".to_string()),
                    self.file_type.extension
                )
            )
    }
}


pub trait CopyOperationFactory {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String>;
}

#[derive(new)]
pub struct OriginalsCopyOperationFactory;
impl CopyOperationFactory for OriginalsCopyOperationFactory {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String> {
        let operation = CopyOperation::new(
            asset.get_path(),
            asset.original_uti,
            asset.uuid.clone(),
            None,
            None,
        );
        Ok(vec![operation])
    }
}

#[derive(new)]
pub struct DerivatesCopyOperationFactory;
impl CopyOperationFactory for DerivatesCopyOperationFactory {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String> {
        let operations = if asset.has_adjustments {
            vec![
                CopyOperation::new(
                    asset.get_derivate_path().ok_or("No derivate path")?,
                    asset.derivate_uti,
                    asset.uuid.clone(),
                    Some("_edited".to_string()),
                    None,
                )
            ]
        } else {
            vec![]
        };
        Ok(operations)
    }
}

#[derive(new)]
pub struct CombiningCopyOperationFactory {
    factories: Vec<Box<dyn CopyOperationFactory>>,
}

impl CopyOperationFactory for CombiningCopyOperationFactory {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String> {
        let mut operations = self.factories
            .iter()
            .map(|factory| factory.build(asset))
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<CopyOperation>>();

        operations.sort_by_key(|op| op.source_path.to_string_lossy().into_owned());

        Ok(operations)
    }
}

#[derive(new)]
pub struct FilenameRestoringCopyOperationFactoryDecorator {
    inner: Box<dyn CopyOperationFactory>,
}
impl CopyOperationFactory for FilenameRestoringCopyOperationFactoryDecorator {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String> {
        self.inner
            .build(asset)?
            .into_iter()
            .map(|op| {
                let original_filename_stem = PathBuf::from(&asset.original_filename)
                    .file_stem()
                    .ok_or("Failed to get file stem")?
                    .to_string_lossy()
                    .to_string();

                Ok(CopyOperation {
                    output_filename: original_filename_stem,
                    ..op
                })
            })
            .collect::<Result<Vec<CopyOperation>, String>>()
    }
}

#[derive(new)]
pub struct OutputStructureCopyOperationFactoryDecorator {
    inner: Box<dyn CopyOperationFactory>,
    strategy: Box<dyn OutputStrategy>,
}
impl CopyOperationFactory for OutputStructureCopyOperationFactoryDecorator {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String> {
        let operations = self.inner
            .build(asset)?
            .into_iter()
            .map(|op| {
                CopyOperation {
                    output_folder: self.strategy.get_relative_output_dir(asset).ok(),
                    ..op
                }
            })
            .collect();

        Ok(operations)
    }
}

#[derive(new)]
pub struct SuffixSettingCopyOperationFactoryDecorator {
    inner: Box<dyn CopyOperationFactory>,
    suffix: String,
}
impl CopyOperationFactory for SuffixSettingCopyOperationFactoryDecorator {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String> {
        let operations = self.inner
            .build(asset)?
            .into_iter()
            .map(|op| {
                CopyOperation {
                    output_filename_suffix: Some(self.suffix.clone()),
                    ..op
                }
            })
            .collect();

        Ok(operations)
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