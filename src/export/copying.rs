use std::fs::{copy, create_dir_all};
use std::path::PathBuf;

use derive_new::new;

use crate::export::structure::OutputStrategy;
use crate::model::asset::ExportAsset;
use crate::model::uti::Uti;

#[derive(new)]
pub struct CopyOperation {
    pub source_path: PathBuf,
    pub uti: &'static Uti,
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
                    self.uti.extension
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
pub struct AbsolutePathBuildingCopyOperationFactoryDecorator {
    library_path: PathBuf,
    output_folder: PathBuf,
    inner: Box<dyn CopyOperationFactory>,
}
impl CopyOperationFactory for AbsolutePathBuildingCopyOperationFactoryDecorator {
    fn build(&self, asset: &ExportAsset) -> Result<Vec<CopyOperation>, String> {
        let operations = self.inner
            .build(asset)?
            .into_iter()
            .map(|op| {
                CopyOperation {
                    source_path: self.library_path.join(&op.source_path),
                    output_folder: Some(
                        self.output_folder.clone()
                            .join(&op.output_folder.unwrap_or(PathBuf::new()))
                    ),
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


pub trait AssetCopyStrategy {

    fn copy_asset(&self, copy_operation: &CopyOperation) -> Result<u64, std::io::Error>;
}

#[derive(new)]
pub struct DryRunAssetCopyStrategy;
impl AssetCopyStrategy for DryRunAssetCopyStrategy {

    fn copy_asset(&self, _: &CopyOperation) -> Result<u64, std::io::Error> {
        // do nothing - dry run
        Ok(0)
    }
}

#[derive(new)]
pub struct DefaultAssetCopyStrategy;
impl AssetCopyStrategy for DefaultAssetCopyStrategy {

    fn copy_asset(&self, copy_operation: &CopyOperation) -> Result<u64, std::io::Error> {
        let dest = copy_operation.get_output_path();

        if let Some(parent) = dest.parent() {
            create_dir_all(parent)?
        }
        copy(&copy_operation.source_path, &dest)
    }
}