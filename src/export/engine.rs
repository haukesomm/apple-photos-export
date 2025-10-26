use crate::confirmation::{confirmation_prompt, Answer};
use crate::export::ExportTask;
use crate::result::Error;
use colored::Colorize;
use crate::export::copying::{CopyAsset, CopyAssetViaFs, PretendToCopyAsset};

/// Holds the metadata for the export process, including the total number of assets,
/// the number of exportable assets, and the number of export tasks.
pub struct ExportMetadata {
    pub total_asset_count: usize,
    pub exportable_asset_count: usize,
    pub export_task_count: usize
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
            copy_strategy: Box::new(CopyAssetViaFs::new()),
        }
    }
    
    /// Creates a new instance of the export engine that simulates the export process without
    /// actually copying any files.
    /// 
    /// Use the `new` method to create a real instance of the engine that performs the export.
    pub fn dry_run() -> Self {
        Self {
            copy_strategy: Box::new(PretendToCopyAsset::new()),
        }
    }
    
    
    /// Executes the export process using the provided tasks and metadata.
    /// 
    /// The method iterates over the tasks, copying each asset from the source to the destination.
    /// If any errors occur during the export, they are collected and returned as a result.
    pub fn run_export(&self, tasks: Vec<ExportTask>, meta: ExportMetadata) -> crate::Result<()> {
        if meta.total_asset_count != meta.exportable_asset_count {
            println!(
                "{} Out of {} assets in your library only {} are exportable",
                "Warning:".yellow(),
                meta.total_asset_count,
                meta.exportable_asset_count
            );
            println!("This may be because the missing assets have been offloaded to iCloud.");
            println!("Try downloading the entire library via the Photos app's settings to fix this.")
        }

        println!(
            "{} Assets that are part of multiple albums will be exported multiple times.",
            "Info:".blue()
        );

        if let Answer::No = confirmation_prompt(
            format!(
                "{}",
                format!(
                    "The export will consist of {} files. Start now?",
                    meta.export_task_count
                ).bright_green()
            )
        ) {
            return Ok(());
        };

        let (successes, failures): (i32, Vec<(String, String)>) = tasks
            .iter()
            .enumerate()
            .fold((0, vec![]), |(success_counter, failures), (index, task)| {
                match self.export_asset(task, index, meta.export_task_count) {
                    Ok(_) => (success_counter + 1, failures),
                    Err(msg) => {
                        let mut f = Vec::from(failures);
                        f.push((task.source.display().to_string(), msg));
                        (success_counter, f)
                    }
                }
            });

        if failures.is_empty() {
            self.copy_strategy.report_success(successes);
            Ok(())
        } else {
            Err(Error::Export(failures))
        }
    }
    
    fn export_asset(&self, task: &ExportTask, index: usize, total: usize) -> Result<(), String> {
        println!(
            "{}: {}",
            format!("[{}/{}]", index + 1, total).yellow(),
            task,
        );
        
        self.copy_strategy.copy(&task)
    }
}