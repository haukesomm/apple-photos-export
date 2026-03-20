pub mod mapping;

use crate::model::asset::Asset;
use crate::model::Library;
use colored::Colorize;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Clone)]
pub enum ExportTask {
    Copy(AssetMapping),
    Delete(PathBuf),
}

// TODO Impl Default for AssetMapping
// TODO Improve error handling
#[derive(Clone)]
pub struct AssetMapping {
    pub asset: Asset,
    pub source: PathBuf,
    pub destination_dir: PathBuf,
    pub filename_components: Vec<String>,
    pub file_extension: String,
    pub is_derivate: bool,
    pub album_id: Option<i32>,
    pub skip: bool,
    pub is_part_of_raw_pair: bool,
}

impl AssetMapping {
    pub fn for_original(lib: &Library, asset: Asset) -> Self {
        let filename_path = PathBuf::from(&asset.filename);

        Self {
            asset: asset.clone(),
            source: lib.get_asset_original_path(&asset),
            destination_dir: PathBuf::new(),
            filename_components: vec![filename_path
                .file_stem()
                .expect("Fatal: Encountered an internal asset without file extension!")
                .to_string_lossy()
                .to_string()],
            file_extension: filename_path
                .extension()
                .expect("Fatal: Encountered an internal asset without file extension!")
                .to_string_lossy()
                .to_string(),
            is_derivate: false,
            album_id: None,
            skip: false,
            is_part_of_raw_pair: false,
        }
    }

    pub fn for_derivate(lib: &Library, asset: Asset) -> Option<Self> {
        let path = lib.get_asset_derivate_path(&asset)?;

        if !path.exists() {
            return None;
        }

        Some(Self {
            asset: asset.clone(),
            source: path,
            destination_dir: PathBuf::new(),
            filename_components: vec![PathBuf::from(&asset.filename)
                .file_stem()
                .expect("Fatal: Encountered an internal asset without file extension!")
                .to_string_lossy()
                .to_string()],
            file_extension: asset.derivate_uti.ext.to_string(),
            is_derivate: true,
            album_id: None,
            skip: false,
            is_part_of_raw_pair: false,
        })
    }

    pub fn destination_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.destination_dir).join(format!(
            "{}.{}",
            self.filename_components.join("."),
            self.file_extension.as_str()
        ));

        path
    }
}

impl Display for AssetMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        if self.is_derivate {
            write!(f, "{}", "derivate".cyan())?;
        } else {
            write!(f, "{}", "original".blue())?;
        }

        if self.is_part_of_raw_pair {
            write!(f, ", ")?;
            write!(f, "{}", "RAW pair".bright_magenta())?;
        }

        if let Some(album_id) = self.album_id {
            write!(
                f,
                ", {}",
                format!("album #{}", album_id.to_string()).magenta()
            )?;
        }

        write!(f, ") ")?;

        write!(
            f,
            "{} => {}",
            self.source.display().to_string().dimmed(),
            self.destination_path().display().to_string().dimmed(),
        )
    }
}
