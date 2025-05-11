use colored::Colorize;
use derive_new::new;
use crate::export::ExportTask;


/// Implementors of this trait are able to copy an Asset from an ExportTasks source to the 
/// associated destination.
/// 
/// Additionally, this trait also defines how to report the number of successful copy operations
/// to the user.
pub trait CopyAsset {
    fn copy(&self, task: &ExportTask) -> Result<(), String>;
    fn report_success(&self, count: i32);
}


/// Represents a strategy that actually copies Asset using the `std::fs` module.
#[derive(new)]
pub struct CopyAssetViaFs;

impl CopyAsset for CopyAssetViaFs {
    fn copy(&self, task: &ExportTask) -> Result<(), String> {
        let stem = task.destination
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .ok_or(
                format!(
                    "Original file name has no stem - source: {}, original filename: {}",
                    task.source.display(),
                    task.destination.display()
                )
            )?;
        
        let ext = task.destination
            .extension()
            .ok_or(
                format!(
                    "Original file name has no extension - source: {}, original filename: {}", 
                    task.source.display(), 
                    task.destination.display()
                )
            )?;

        let mut dest = task.destination.to_owned();
        let mut counter = 0;
        
        while dest.exists() {
            dest.set_file_name(format!("{} ({})", &stem, counter));
            dest.set_extension(&ext);
            
            counter = counter + 1;

            if counter > 10 {
                return Err(String::from(format!("{}: Too many files with the same name", &stem)));
            }
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create output folders: {}", e))?
        }

        std::fs::copy(&task.source, &task.destination)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn report_success(&self, count: i32) {
        println!("{}", format!("{} files have successfully been copied.", count).bright_green())
    }
}


/// Defines a `dry-run` strategy that does not actually copy any data. 
#[derive(new)]
pub struct PretendToCopyAsset;

impl CopyAsset for PretendToCopyAsset {
    fn copy(&self, _: &ExportTask) -> Result<(), String> {
        Ok(())
    }

    fn report_success(&self, count: i32) {
        println!("{}", format!("Dry-run: {} files would have been copied.", count).magenta())
    }
}