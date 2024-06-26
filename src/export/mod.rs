use std::fs::File;
use std::io::Write;

use colored::Colorize;
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::db::repo::asset::AssetRepository;
use crate::export::copying::{AssetCopyStrategy, CopyOperationFactory};
use crate::export::exporter::Exporter;
use crate::result::{PhotosExportError, PhotosExportResult};

pub mod structure;
pub mod exporter;
pub mod copying;

pub fn export_assets(
    asset_repo: AssetRepository,
    copy_operation_factory: Box<dyn CopyOperationFactory>,
    copy_strategy: Box<dyn AssetCopyStrategy>,
) -> PhotosExportResult<()> {

    let exporter = Exporter::new(
        asset_repo,
        copy_operation_factory,
        copy_strategy,
    );

    exporter.export()
        .map(|count| {
            println!("{}", format!("\nAll {} assets have successfully been exported.", count).green());
            ()
        })
        .map_err(|export| {
            eprintln!(
                "{}",
                format!("\nThe export produced a total of {} errors.", &export.messages.len()).red()
            );
            match write_error_log(&export.messages) {
                Ok(_) => PhotosExportError::empty(),
                Err(e) => PhotosExportError { messages: vec![e] }
            }
        })
}

fn write_error_log(messages: &Vec<String>) -> Result<(), String> {
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let filename = format!("apple-photos-export-{}.log", random_suffix);

    let mut report = File::create(&filename)
        .map_err(|e| format!("Unable to create error log: {}", e))?;

    report.write_all(messages.join("\n").as_bytes())
        .map_err(|e| format!("Unable to write to error log: {}", e))?;

    eprintln!("Error log written to '{}'", &filename.dimmed());

    Ok(())
}