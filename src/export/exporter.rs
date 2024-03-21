use std::path::{Path, PathBuf};

use colored::Colorize;
use derive_new::new;

use crate::export::copying::AssetCopyStrategy;
use crate::export::structure::OutputStructureStrategy;
use crate::model::asset::AssetWithAlbumInfo;
use crate::repo::asset::AssetRepository;
use crate::util::confirmation::{Answer, confirmation_prompt};

#[derive(new)]
pub struct Exporter {
    repo: Box<dyn AssetRepository>,
    output_strategy: Box<dyn OutputStructureStrategy>,
    copy_strategy: Box<dyn AssetCopyStrategy>,
}

impl Exporter {

    fn get_source_path(&self, asset_dir: &Path, asset: &AssetWithAlbumInfo) -> PathBuf {
        asset_dir
            .join(asset.dir.clone())
            .join(asset.filename.clone())
    }

    fn get_output_path(&self, output_dir: &Path, asset: &AssetWithAlbumInfo,
                       use_original_filenames: bool) -> PathBuf {

        let filename = if use_original_filenames {
            asset.original_filename.clone()
        } else {
            asset.filename.clone()
        };

        let output_path = self.output_strategy.get_relative_output_dir(asset);
        output_dir.join(output_path).join(filename)
    }

    pub fn export(&self, asset_dir: &Path, output_dir: &Path, use_original_filenames: bool) {
        let assets = self.repo.get_all().unwrap();
        let asset_count = assets.len();

        if let Answer::No = confirmation_prompt(
            format!("Export {} assets to {}?", &asset_count, output_dir.to_string_lossy())
        ) {
            return;
        }

        assets.iter()
            .enumerate()
            .for_each(|(index, asset)| {
                let source_path = self.get_source_path(asset_dir, asset);
                let output_path = self.get_output_path(output_dir, asset, use_original_filenames);

                println!(
                    "{} Exporting '{}' to '{}'",
                    format!("({}/{})", index + 1, asset_count).yellow(),
                    asset.filename.italic(),
                    output_path.to_str().unwrap().italic()
                );

                self.copy_strategy.copy_asset(source_path.as_path(), output_path.as_path())
            });

        self.copy_strategy.on_finish();
    }
}