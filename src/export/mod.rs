pub mod task_mapper;
pub mod factory;
mod engine;
pub mod copying;

use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use colored::Colorize;
pub use engine::{ExportMetadata, ExportEngine};
use crate::model::{Asset, Library};


/// Represents a task to export an asset from the library to a given destination.
/// 
/// Additionally, this struct also holds vector of `additional_destinations` that is populated with
/// other suitable export destinations of the asset.
/// 
/// If an asset is part of multiple albums, all but the first destinations will be part of the field
/// so the exporter can decide how to handle them appropriately.
/// 
/// The `destination` must be __relative__!
#[derive(Clone)]
pub struct ExportTask {
    pub asset: Asset,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub is_derivate: bool,
    pub album_id: Option<i32>,
}

impl ExportTask {
    
    pub fn for_original(lib: &Library, asset: Asset) -> Self {
        Self {
            asset: asset.clone(),
            source: lib.get_asset_original_path(&asset),
            destination: PathBuf::from(&asset.filename),
            is_derivate: false,
            album_id: None,
        }
    }

    pub fn for_derivate(lib: &Library, asset: Asset) -> Option<Self> {
        let path = lib.get_asset_derivate_path(&asset)?;

        if !path.exists() {
            return None;
        }

        let mut output_filename = PathBuf::from(&asset.filename);
        output_filename.set_extension(asset.derivate_uti.ext);

        Some(
            Self {
                asset: asset.clone(),
                source: path,
                destination: output_filename,
                is_derivate: true,
                album_id: None,
            }
        )
    }
}

impl Display for ExportTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        
        if self.is_derivate {
            write!(f, "{}", "derivate".cyan())?;
        } else {
            write!(f, "{}", "original".green())?;
        }
        
        if let Some(album_id) = self.album_id {
            write!(f, ", {}", format!("album #{}", album_id.to_string()).magenta())?;
        }
        
        write!(f, ") ")?;
        
        write!(
            f, 
            "{} => {}", 
            self.source.display().to_string().dimmed(),
            self.destination.display().to_string().dimmed(),
        )
    }
}