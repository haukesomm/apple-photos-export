use std::path::Path;

use colored::Colorize;

use crate::confirmation::{Answer, confirmation_prompt};
use crate::export::copying::AssetCopyStrategy;
use crate::export::structure::OutputStructureStrategy;
use crate::repo::asset::AssetWithAlbumInfoRepo;

pub struct Exporter<'a> {
    repo: &'a AssetWithAlbumInfoRepo,
    output_strategy: &'a dyn OutputStructureStrategy,
    copy_strategy: &'a dyn AssetCopyStrategy
}

impl Exporter<'_> {

    pub fn new<'a>(
        repo: &'a AssetWithAlbumInfoRepo,
        output_strategy: &'a dyn OutputStructureStrategy,
        copy_strategy: &'a dyn AssetCopyStrategy
    ) -> Exporter<'a> {
        Exporter { repo, output_strategy, copy_strategy }
    }

    pub fn export(&self, asset_dir: &Path, output_dir: &Path, use_original_filenames: bool) {
        let assets = self.repo.get_all().unwrap();
        let asset_count = assets.len();

        match confirmation_prompt(
            format!("Export {} assets to {}?", &asset_count, output_dir.to_string_lossy())
        ) {
            Answer::Yes => println!(),
            Answer::No => return
        }

        assets.iter()
            .enumerate()
            .for_each(|(index, asset)| {
                let source_path = asset_dir.join(asset.get_path());
                let output_path = {
                    let filename = if use_original_filenames {
                        asset.original_filename.clone()
                    } else {
                        asset.filename.clone()
                    };
                    output_dir
                        .join(self.output_strategy.get_relative_output_dir(asset))
                        .join(filename)
                };

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