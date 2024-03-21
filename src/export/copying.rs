use std::fs::{copy, create_dir_all};
use std::path::Path;

use colored::Colorize;
use derive_new::new;

pub trait AssetCopyStrategy {

    fn copy_asset(&self, src: &Path, dest: &Path);

    fn on_finish(&self);
}


#[derive(new)]
pub struct DryRunAssetCopyStrategy;

impl AssetCopyStrategy for DryRunAssetCopyStrategy {

    fn copy_asset(&self, _: &Path, _: &Path) {
        // do nothing - dry run
    }

    fn on_finish(&self) {
        println!("{}", "Done. This was a dry run - no files have been exported.".magenta())
    }
}


#[derive(new)]
pub struct DefaultAssetCopyStrategy;

impl AssetCopyStrategy for DefaultAssetCopyStrategy {

    fn copy_asset(&self, src: &Path, dest: &Path) {
        if let Some(parent) = dest.parent() {
            create_dir_all(parent).expect("Cannot create parent directories");
        }
        if let Err(e) = copy(src, dest) {
            panic!("Error copying file: {}", e)
        }
    }

    fn on_finish(&self) {
        println!("{}", "Done.".green())
    }
}