use std::fs::copy;
use std::path::Path;

use colored::Colorize;

pub trait AssetCopyStrategy {

    fn copy_asset(&self, src: &Path, dest: &Path);

    fn on_finish(&self);
}


pub struct DryRunAssetCopyStrategy;

impl DryRunAssetCopyStrategy {
    pub fn new() -> DryRunAssetCopyStrategy {
        DryRunAssetCopyStrategy {}
    }
}

impl AssetCopyStrategy for DryRunAssetCopyStrategy {

    fn copy_asset(&self, _: &Path, _: &Path) {
        // do nothing - dry run
    }

    fn on_finish(&self) {
        println!("{}", "Done. This was a dry run - no files have been exported.".magenta())
    }
}


pub struct DefaultAssetCopyStrategy;

impl DefaultAssetCopyStrategy {
    pub fn new() -> DefaultAssetCopyStrategy {
        DefaultAssetCopyStrategy
    }
}

impl AssetCopyStrategy for DefaultAssetCopyStrategy {

    // TODO: Better error handling
    fn copy_asset(&self, src: &Path, dest: &Path) {
        match copy(src, dest) {
            Ok(_) => {}
            Err(e) => panic!("{}", e)
        }
    }

    fn on_finish(&self) {
        println!("{}", "Done.".green())
    }
}