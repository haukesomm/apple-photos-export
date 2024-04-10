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
    library_path: PathBuf,
    output_path: PathBuf
}

impl Exporter {

    pub fn export(&self) -> Result<(), String> {
        let out_dir = self.output_path.clone();

        let total_count = self.get_total_count()?;
        let offloaded_count = self.get_missing_count()?;

        if offloaded_count > 0 {
            if let Answer::No = missing_assets_prompt(total_count, offloaded_count) {
                return Ok(())
            }
        }

        let export_assets: Vec<ExportAsset> = self.get_exportable_assets()?;
        let export_assets_count = export_assets.len() as i64;

        if let Answer::No = start_export_prompt(export_assets_count, out_dir.as_path()) {
            return Ok(());
        }

        type ResultType = Vec<Result<(), String>>;
        let (exported, errors): (ResultType, ResultType) = export_assets
            .iter()
            .enumerate()
            .map(|(index, asset)| {
                self.export_single_asset(index, export_assets_count, asset)
            })
            .partition(Result::is_ok);

        self.copy_strategy.on_finish(
            if errors.is_empty() {
                FinishState::Success(exported.len())
            } else {
                FinishState::Failure(
                    export_assets_count,
                    errors.into_iter().map(Result::unwrap_err).collect()
                )
            }
        );

        Ok(())
    }

    fn export_single_asset(&self, index: usize, total: i64, asset: &ExportAsset) -> Result<(), String> {
        let source_path = self.library_path.join(asset.path());
        let output_path = self.get_absolute_output_path(asset)?;

        println!(
            "{} Exporting '{}' to '{}'",
            format!("({}/{})", index + 1, total).yellow(),
            asset.filename.italic(),
            output_path.to_str().unwrap().italic()
        );

        self.copy_strategy
            .copy_asset(source_path.as_path(), output_path.as_path())
            .map_err(|e| format!("{}: {}", source_path.to_string_lossy(), e))
    }

    fn get_total_count(&self) -> Result<i64, String> {
        self.repo
            .get_total_count()
            .map_err(|e| e.to_string())
    }

    fn get_missing_count(&self) -> Result<i64, String> {
        self.repo
            .get_offloaded_count()
            .map_err(|e| e.to_string())
    }

    fn get_exportable_assets(&self) -> Result<Vec<ExportAsset>, String> {
        self.repo
            .get_exportable_assets()
            .map_err(|e| e.to_string())?
            .iter()
            .map(|a| {
                ExportAsset::from_db_model(a.clone())
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<ExportAsset>, String>>()
    }

    fn get_absolute_output_path(&self, asset: &ExportAsset) -> Result<PathBuf, String> {
        let filename = if self.use_original_filenames {
            asset.original_filename.clone()
        } else {
            asset.filename.clone()
        };

        let relative_path = self.output_strategy.get_relative_output_dir(asset)?;
        let absolute_path = self.output_path.join(relative_path).join(filename);

        Ok(absolute_path)
    }
}

fn missing_assets_prompt(total: i64, missing: i64) -> Answer {
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

fn start_export_prompt(total: i64, out_dir: &Path) -> Answer {
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