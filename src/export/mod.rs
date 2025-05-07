pub mod task_mapper;
pub mod builder;
mod engine;

use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use colored::Colorize;
pub use engine::{ExportMetadata, ExportEngine};
use crate::model::{Asset, Library};

/// Represents a special relation an asset may have to another model during the export.
/// 
/// Currently, the only relation is that the asset is a member of an album, in which case addtional
/// destinations may be created.
/// 
/// If no such relation exists, the `None` variant should be used.
#[derive(Clone)]
pub enum ExportAssetRelation {
    /// No special relation exists.
    None,
    
    /// This relation is used to indicate that the asset is a member of an album.
    /// 
    /// There may be cases where an asset is part of multiple albums, in which case one album is
    /// considered the _master_ album and the others are considered _additional_ albums. If an
    /// asset is _not_ the master, the master album's id is stored in the `master` field.
    /// 
    /// This is done in order to avoid exporting multiple copies of the same asset to multiple
    /// directories if the album-based export-structure is used.
    AlbumMember { album_id: i32, master: Option<i32> },
}


/// Holds the metadata for an asset that is being exported.
/// 
/// The metadata may be used to display additional information about the asset during the export
/// or to determine special steps needed during the export.
#[derive(Clone)]
pub struct ExportAssetMetadata {
    pub derivate: bool,
    pub relation: ExportAssetRelation
}

impl Display for ExportAssetMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.derivate {
            write!(f, "{}", "derivate".cyan())?;
        } else {
            write!(f, "{}", "original".green())?;
        }

        match self.relation {
            ExportAssetRelation::AlbumMember { album_id, master: None } =>
                write!(
                    f,
                    ", {}",
                    format!("album {} (primary destination)", album_id).magenta()
                ),
            ExportAssetRelation::AlbumMember { album_id, master: Some(_) } =>
                write!(
                    f,
                    ", {}",
                    format!(
                        "album {} (additional destination)",
                        album_id
                    ).bright_magenta()
                ),
            _ => Ok(())
        }
    }
}


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
    pub meta: ExportAssetMetadata,
}

impl ExportTask {
    
    pub fn for_original_from(lib: &Library, asset: Asset) -> Self {
        Self {
            asset: asset.clone(),
            source: lib.get_asset_original_path(&asset),
            destination: PathBuf::from(&asset.filename),
            meta: ExportAssetMetadata {
                derivate: false,
                relation: ExportAssetRelation::None,
            },
        }
    }

    pub fn for_derivate_from(lib: &Library, asset: Asset) -> Option<Self> {
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
                meta: ExportAssetMetadata {
                    derivate: true,
                    relation: ExportAssetRelation::None,
                }
            }
        )
    }
}