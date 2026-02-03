use crate::confirmation::{confirmation_prompt, Answer};
use crate::export::copying::{CopyAsset, CopyAssetViaFs, PretendToCopyAsset};
use crate::export::task::{AssetMapping, ExportTask};
use crate::result::Error;
use colored::Colorize;
use log::{info, warn};
use std::fmt::{Display, Formatter};

/// Holds the metadata for the export process, including the total number of assets,
/// the number of exportable assets, and the number of export tasks.
pub struct ExportMetadata {
    pub total_asset_count: usize,
    pub exportable_asset_count: usize,
    pub export_task_count: usize,
}

struct ExportStep {
    task: ExportTask,
    index: usize,
    total: usize,
}

impl Display for ExportStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { task, index, total } = self;

        write!(f, "{}", format!("[{}/{}] ", index + 1, total).yellow())?;

        match task {
            ExportTask::Copy(mapping @ AssetMapping { skip: false, .. }) => {
                write!(f, "{} {}", "Exporting".bright_green(), mapping)?;
            }
            ExportTask::Copy(mapping @ AssetMapping { skip: true, .. }) => {
                write!(f, "{} {}", "Skipping".bright_blue(), mapping)?;
            }
            ExportTask::Delete(path) => {
                write!(
                    f,
                    "{} {}",
                    "Deleting file at".bright_red(),
                    format!("{}", path.display()).dimmed()
                )?;
            }
        };

        Ok(())
    }
}

/// Represents the export engine responsible for executing the export tasks.
///
/// The export engine takes care of copying files from the source to the destination and reports
/// the results of the user.
///
/// The engine can be configured to run in a dry-run mode, where it simulates the export process
/// without actually copying any files by creating a new instance of the engine with the
/// `dry_run` method instead of the `new` method.
pub struct ExportEngine {
    copy_strategy: Box<dyn CopyAsset>,
}

impl ExportEngine {
    /// Creates a new instance of the export engine.
    ///
    /// The engine is configured to copy files from the source to the destination using the
    /// `std::fs::copy` function.
    ///
    /// Use the `dry_run` method to create a dry-run instance of the engine.
    pub fn new() -> Self {
        Self {
            copy_strategy: Box::new(CopyAssetViaFs),
        }
    }

    /// Creates a new instance of the export engine that simulates the export process without
    /// actually copying any files.
    ///
    /// Use the `new` method to create a real instance of the engine that performs the export.
    pub fn dry_run() -> Self {
        Self {
            copy_strategy: Box::new(PretendToCopyAsset),
        }
    }

    /// Executes the export process using the provided tasks and metadata.
    ///
    /// The method iterates over the tasks, copying each asset from the source to the destination.
    /// If any errors occur during the export, they are collected and returned as a result.
    pub fn run_export(&self, tasks: Vec<ExportTask>, meta: ExportMetadata) -> crate::Result<()> {
        if meta.total_asset_count != meta.exportable_asset_count {
            warn!(
                "Out of {} assets in your library only {} are exportable. This may be because the \
                missing assets have been offloaded to iCloud. Try downloading the entire library \
                via the Photos app's settings to fix this.",
                meta.total_asset_count, meta.exportable_asset_count
            );
        }

        info!(
            "The export will consist of {} files. Assets that are part of multiple albums will be \
            exported multiple times.",
            meta.export_task_count
        );

        if let Answer::No = confirmation_prompt("Start the export now?") {
            return Ok(());
        };

        let (successes, failures): (i32, Vec<String>) = tasks.into_iter().enumerate().fold(
            (0, vec![]),
            |(success_counter, failures), (index, task)| match self.export_asset(ExportStep {
                task,
                index,
                total: meta.export_task_count,
            }) {
                Ok(_) => (success_counter + 1, failures),
                Err(msg) => {
                    let mut f = Vec::from(failures);
                    f.push(msg);
                    (success_counter, f)
                }
            },
        );

        if failures.is_empty() {
            self.copy_strategy.report_success(successes);
            Ok(())
        } else {
            Err(Error::Export(failures))
        }
    }

    fn export_asset(&self, step: ExportStep) -> Result<(), String> {
        info!("{}", step);

        match step.task {
            ExportTask::Copy(mapping @ AssetMapping { skip: false, .. }) => {
                self.copy_strategy.copy(&mapping)
            }
            ExportTask::Delete(path) => self.copy_strategy.delete(&path),
            _ => Ok(()),
        }
    }
}
