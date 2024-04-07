use std::path::{Path, PathBuf};

use colored::Colorize;
use derive_new::new;

use crate::db::model::album::Album;
use crate::db::model::asset::{Asset, AssetAttributes};
use crate::db::repo::exportable_assets::ExportableAssetsRepository;
use crate::export::copying::AssetCopyStrategy;
use crate::export::structure::OutputStructureStrategy;
use crate::util::confirmation::{Answer, confirmation_prompt};

#[derive(new)]
pub struct Exporter {
    repo: ExportableAssetsRepository,
    output_strategy: Box<dyn OutputStructureStrategy>,
    copy_strategy: Box<dyn AssetCopyStrategy>,
}

impl Exporter {

    pub fn export(&self, asset_dir: &Path, output_dir: &Path, use_original_filenames: bool) {
        let total_asset_count = self.repo.get_total_count();

        let offloaded_count = self.repo.get_offloaded_count();
        if offloaded_count > 0 {
            println!(
                "{} {} of {} assets are not locally available and cannot be exported!",
                "Warning:".yellow(),
                offloaded_count,
                total_asset_count,
            );
            if let Answer::No = confirmation_prompt(
                format!(
                    "Continue with {} available assets?",
                    total_asset_count - offloaded_count
                )
            ) {
                return;
            }
        }

        let exportable_assets = self.repo.get_exportable_assets();
        let exportable_assets_count = exportable_assets.len();

        println!(
            "{} Some assets may be part of multiple albums and will be exported multiple times. \
            Thus, the number of exported assets may be higher than the number of assets in the \
            database.",
            "Note:".blue()
        );
        if let Answer::No = confirmation_prompt(
            format!("Export {} assets to {}?", &exportable_assets_count, output_dir.to_string_lossy())
        ) {
            return;
        }

        exportable_assets.iter()
            .enumerate()
            .for_each(|(index, result)| {
                let (asset, asset_attribs, album) = result;

                let source_path = self.get_source_path(asset_dir, asset);
                let output_path = output_dir.join(
                    self.get_output_path(asset, asset_attribs, album, use_original_filenames)
                );

                println!(
                    "{} Exporting '{}' to '{}'",
                    format!("({}/{})", index + 1, exportable_assets_count).yellow(),
                    asset.filename.italic(),
                    output_path.to_str().unwrap().italic()
                );

                self.copy_strategy.copy_asset(source_path.as_path(), output_path.as_path())
            });

        self.copy_strategy.on_finish();
    }

    fn get_source_path(&self, asset_dir: &Path, asset: &Asset) -> PathBuf {
        asset_dir
            .join(asset.dir.clone())
            .join(asset.filename.clone())
    }

    fn get_output_path(&self, asset: &Asset, attribs: &AssetAttributes,
                       album: &Option<Album>, use_original_filenames: bool) -> PathBuf {

        let filename = if use_original_filenames {
            attribs.original_filename.clone()
        } else {
            asset.filename.clone()
        };

        self.output_strategy
            .get_relative_output_dir(asset, album)
            .join(filename)
    }
}