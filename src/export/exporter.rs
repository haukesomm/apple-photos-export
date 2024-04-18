use std::path::{Path, PathBuf};

use colored::Colorize;
use derive_new::new;

use crate::db::repo::asset::{AssetRepository, LocalAvailability};
use crate::export::copying::{AssetCopyStrategy, CopyOperation, CopyOperationFactory, FinishState};
use crate::model::asset::ExportAsset;
use crate::model::FromDbModel;
use crate::util::confirmation::{Answer, confirmation_prompt};

#[derive(new)]
pub struct Exporter {
    repo: AssetRepository,
    library_path: PathBuf,
    output_path: PathBuf,
    copy_operation_factory: Box<dyn CopyOperationFactory>,
    copy_strategy: Box<dyn AssetCopyStrategy>,
}

impl Exporter {

    pub fn export(&self) -> Result<(), String> {
        let out_dir = self.output_path.clone();

        let visible_count = self.get_visible_count()?;
        let visible_offloaded_count = self.get_visible_offloaded_count()?;

        if visible_offloaded_count > 0 {
            if let Answer::No = missing_assets_prompt(visible_count, visible_offloaded_count) {
                return Ok(())
            }
        }

        let export_assets = self
            .get_exportable_assets()?
            .iter()
            .map(|a| self.copy_operation_factory.build(a))
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<CopyOperation>>();

        let export_assets_count = export_assets.len() as i64;

        if export_assets_count == 0 {
            no_matching_assets_prompt();
            return Ok(());
        }

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

    fn export_single_asset(&self, index: usize, total: i64, copy_operation: &CopyOperation) -> Result<(), String> {
        let source_path = self.library_path.join(&copy_operation.source_path);
        let output_path = self.output_path.join(&copy_operation.get_output_path());

        println!(
            "{} Exporting '{}' to '{}'",
            format!("({}/{})", index + 1, total).yellow(),
            source_path.to_string_lossy().dimmed(),
            output_path.to_str().unwrap().dimmed()
        );

        self.copy_strategy
            .copy_asset(source_path.as_path(), output_path.as_path())
            .map_err(|e| format!("{}: {}", source_path.to_string_lossy(), e))
    }

    fn get_visible_count(&self) -> Result<i64, String> {
        self.repo
            .get_visible_count(LocalAvailability::Any)
            .map_err(|e| e.to_string())
    }

    fn get_visible_offloaded_count(&self) -> Result<i64, String> {
        self.repo
            .get_visible_count(LocalAvailability::Offloaded)
            .map_err(|e| e.to_string())
    }

    fn get_exportable_assets(&self) -> Result<Vec<ExportAsset>, String> {
        self.repo
            .get_exportable()
            .map_err(|e| e.to_string())?
            .iter()
            .map(|a| {
                ExportAsset::from_db_model(a.clone())
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<ExportAsset>, String>>()
    }
}

fn missing_assets_prompt(total: i64, missing: i64) -> Answer {
    println!(
        "{} {} of {} assets in your library are not locally available and can not be exported.",
        "Warning:".yellow(),
        missing,
        total,
    );
    confirmation_prompt("Continue anyway?".to_string())
}

fn no_matching_assets_prompt() {
    println!("{} No available assets match the specified criteria!", "Warning:".yellow())
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