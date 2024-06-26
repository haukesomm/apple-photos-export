use colored::Colorize;
use derive_new::new;

use crate::db::repo::asset::{AssetRepository, LocalAvailabilityFilter};
use crate::export::copying::{AssetCopyStrategy, CopyOperation, CopyOperationFactory};
use crate::model::asset::ExportAsset;
use crate::model::FromDbModel;
use crate::result::{PhotosExportError, PhotosExportResult};
use crate::util::confirmation::{Answer, confirmation_prompt};

#[derive(new)]
pub struct Exporter {
    repo: AssetRepository,
    copy_operation_factory: Box<dyn CopyOperationFactory>,
    copy_strategy: Box<dyn AssetCopyStrategy>,
}

impl Exporter {

    pub fn export(&self) -> PhotosExportResult<u64> {
        let visible_count = self.get_visible_count()?;
        let visible_offloaded_count = self.get_visible_offloaded_count()?;

        if visible_offloaded_count > 0 {
            if let Answer::No = self.missing_assets_prompt(visible_count, visible_offloaded_count) {
                return Ok(0)
            }
        }

        let export_assets = self.get_copy_operations()?;
        let export_assets_count = export_assets.len() as i64;

        if export_assets_count == 0 {
            self.no_matching_assets_warning();
            return Ok(0);
        }

        if let Answer::No = self.start_export_prompt(export_assets_count) {
            return Ok(0);
        }

        let (export_count, error_messages) = export_assets
            .iter()
            .enumerate()
            .fold((0, Vec::<String>::new()), |(cnt, msgs), (index, op)| {
                let result = self.export_single_asset(index, export_assets_count, op);
                match result {
                    Ok(_) => (cnt + 1, msgs),
                    Err(e) => (cnt, [msgs, vec![e.to_string()]].concat())
                }
            });

        if error_messages.is_empty() {
            Ok(export_count)
        } else {
            Err(PhotosExportError { messages: error_messages })
        }
    }


    fn export_single_asset(&self, index: usize, total: i64, copy_operation: &CopyOperation) -> Result<(), String> {
        let source_path = copy_operation.source_path.to_string_lossy().to_string();
        let output_path = copy_operation.get_output_path().to_string_lossy().to_string();

        println!(
            "{} Exporting '{}' to '{}'",
            format!("({}/{})", index + 1, total).yellow(),
            source_path.dimmed(),
            output_path.dimmed()
        );

        self.copy_strategy.copy_asset(copy_operation)
            .map(|_| ())
            .map_err(|e| {
                // Short error message to print to the console
                eprintln!("{} {}", "Error:".red(), e.to_string());
                // Long, more detailed error message to include in the error log
                format!(
                    "Error exporting '{}' to '{}': {}",
                    source_path,
                    output_path,
                    e.to_string()
                )
            })
    }


    fn get_visible_count(&self) -> Result<i64, String> {
        self.repo
            .get_visible_count(LocalAvailabilityFilter::Any)
            .map_err(|e| e.to_string())
    }

    fn get_visible_offloaded_count(&self) -> Result<i64, String> {
        self.repo
            .get_visible_count(LocalAvailabilityFilter::Offloaded)
            .map_err(|e| e.to_string())
    }

    fn get_copy_operations(&self) -> Result<Vec<CopyOperation>, String> {
        let operations = self
            .get_exportable_assets()?
            .iter()
            .map(|a| self.copy_operation_factory.build(a))
            .collect::<Result<Vec<Vec<_>>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<CopyOperation>>();

        Ok(operations)
    }

    fn get_exportable_assets(&self) -> Result<Vec<ExportAsset>, String> {
        self.repo
            .get_exportable()
            .map_err(|e| e.to_string())?
            .iter()
            .map(|a| {
                ExportAsset::from_db_model(a)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<ExportAsset>, String>>()
    }


    fn missing_assets_prompt(&self, total: i64, missing: i64) -> Answer {
        println!(
            "{} {} of {} assets in your library are not locally available and can not be exported.",
            "Warning:".yellow(),
            missing,
            total,
        );
        confirmation_prompt("Continue anyway?".to_string())
    }

    fn start_export_prompt(&self, total: i64) -> Answer {
        println!(
            "{} Some assets may be part of multiple albums and will be exported multiple times. \
            Thus, the number of exported assets may be higher than the number of assets in the \
            database.",
            "Note:".blue()
        );
        confirmation_prompt(
            format!(
                "Export {} assets?",
                &total,
            )
        )
    }

    fn no_matching_assets_warning(&self) {
        println!("{} No available assets match the specified criteria!", "Warning:".yellow())
    }
}