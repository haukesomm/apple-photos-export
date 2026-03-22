use colored::Colorize;
use std::path::Path;

/// This trait defines a strategy to execute export-related file operations such as copying or
/// deleting a file and printing a resulting success message to the screen.
///
/// Additionally, this trait also defines how to report the number of successful copy operations
/// to the user.
pub trait ExecuteFileOperation {
    fn copy(&self, source: &Path, destination: &Path) -> crate::Result<()>;

    fn delete(&self, path: &Path) -> crate::Result<()>;

    fn report_success(&self, count: usize);
}

/// Represents a strategy that actually copies Asset using the `std::fs` module.
pub struct ActualExport;

impl ExecuteFileOperation for ActualExport {
    fn copy(&self, source: &Path, destination: &Path) -> crate::Result<()> {
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(&source, &destination)
            .map(|_| ())
            .map_err(|inner_message| {
                format!(
                    "Could not copy file: {} to {}: {}",
                    &source.to_string_lossy(),
                    &destination.to_string_lossy(),
                    inner_message
                )
            })?;

        Ok(())
    }

    fn delete(&self, path: &Path) -> crate::Result<()> {
        std::fs::remove_file(path)
            .map(|_| ())
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn report_success(&self, count: usize) {
        println!(
            "{}",
            format!("{} actions have successfully been executed.", count).bright_green()
        )
    }
}

/// Defines a `dry-run` strategy that does not actually copy any data.
pub struct DryRun;

impl ExecuteFileOperation for DryRun {
    fn copy(&self, _: &Path, _: &Path) -> crate::Result<()> {
        Ok(())
    }

    fn delete(&self, _: &Path) -> crate::Result<()> {
        Ok(())
    }

    fn report_success(&self, count: usize) {
        println!(
            "{}",
            format!("Dry-run: {} actions would have been executed.", count).magenta()
        )
    }
}
