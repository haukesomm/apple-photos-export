use std::path::{Path, PathBuf};

use colored::Colorize;
use derive_new::new;

use crate::db::repo::exportable_assets::ExportableAssetsRepository;
use crate::export::copying::{AssetCopyStrategy, FinishState};
use crate::export::structure::OutputStructureStrategy;
use crate::model::asset::ExportAsset;
use crate::model::FromDbModel;
use crate::util::confirmation::{Answer, confirmation_prompt};

#[derive(new)]
pub struct Exporter {
    repo: ExportableAssetsRepository,
    output_strategy: Box<dyn OutputStructureStrategy>,
    copy_strategy: Box<dyn AssetCopyStrategy>,
    use_original_filenames: bool,
}

impl Exporter {

    pub fn export(&self, asset_dir: &Path, out_dir: &Path) -> Result<(), String> {
        let total_count = self.repo
            .get_total_count()
            .map_err(|e| e.to_string())?;

        let offloaded_count = self.repo
            .get_offloaded_count()
            .map_err(|e| e.to_string())?;

        if offloaded_count > 0 {
            if let Answer::No = self.missing_assets_prompt(total_count, offloaded_count) {
                return Ok(())
            }
        }

        let exportable_assets: Vec<ExportAsset> = self.repo
            .get_exportable_assets()
            .map_err(|e| e.to_string())?
            .iter()
            .map(|a| {
                ExportAsset::from_db_model(a.clone())
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<ExportAsset>, String>>()?;

        let exportable_count = exportable_assets.len() as i64;

        if let Answer::No = self.start_export_prompt(exportable_count, out_dir) {
            return Ok(());
        }

        type ResultType = Vec<Result<(), String>>;
        let (exported, errors): (ResultType, ResultType) = exportable_assets.iter()
            .enumerate()
            .map(|(index, asset)| {
                self.export_single_asset(asset, asset_dir, out_dir, index as i64, exportable_count)
            })
            .partition(Result::is_ok);

        self.copy_strategy.on_finish(
            if errors.is_empty() {
                FinishState::Success(exported.len())
            } else {
                FinishState::Failure(errors.into_iter().map(Result::unwrap_err).collect())
            }
        );

        Ok(())
    }

    fn missing_assets_prompt(&self, total: i64, missing: i64) -> Answer {
        println!(
            "{} {} of {} assets are not locally available and cannot be exported!",
            "Warning:".yellow(),
            missing,
            total,
        );
        confirmation_prompt(
            format!(
                "Continue with {} available assets?",
                total - missing
            )
        )
    }

    fn start_export_prompt(&self, total: i64, out_dir: &Path) -> Answer {
        println!(
            "{} Some assets may be part of multiple albums and will be exported multiple times. \
            Thus, the number of exported assets may be higher than the number of assets in the \
            database.",
            "Note:".blue()
        );
        confirmation_prompt(
            format!(
                "Export {} assets to {}?",
                &total,
                &out_dir.to_string_lossy().to_string()
            )
        )
    }

    fn export_single_asset(
        &self,
        asset: &ExportAsset,
        asset_dir: &Path,
        out_dir: &Path,
        index: i64,
        total: i64
    ) -> Result<(), String> {
        let source_path = self.get_source_path(&asset_dir, asset);
        let output_path = out_dir.join(
            self.get_output_path(asset)
        );

        println!(
            "{} Exporting '{}' to '{}'",
            format!("({}/{})", index + 1, total).yellow(),
            asset.filename.italic(),
            output_path.to_str().unwrap().italic()
        );

        self.copy_strategy.copy_asset(source_path.as_path(), output_path.as_path())
    }

    fn get_source_path(&self, asset_dir: &Path, asset: &ExportAsset) -> PathBuf {
        asset_dir
            .join(asset.dir.clone())
            .join(asset.filename.clone())
    }

    fn get_output_path(&self, asset: &ExportAsset) -> PathBuf {

        let filename = if self.use_original_filenames {
            asset.original_filename.clone()
        } else {
            asset.filename.clone()
        };

        self.output_strategy
            .get_relative_output_dir(asset)
            .join(filename)
    }
}