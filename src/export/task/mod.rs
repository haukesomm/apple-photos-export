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

#[derive(Clone)]
pub struct AssetMapping {
    pub asset: Asset,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub is_derivate: bool,
    pub album_id: Option<i32>,
    pub skip: bool,
}

impl AssetMapping {
    pub fn for_original(lib: &Library, asset: Asset) -> Self {
        Self {
            asset: asset.clone(),
            source: lib.get_asset_original_path(&asset),
            destination: PathBuf::from(&asset.filename),
            is_derivate: false,
            album_id: None,
            skip: false,
        }
    }

    pub fn for_derivate(lib: &Library, asset: Asset) -> Option<Self> {
        let path = lib.get_asset_derivate_path(&asset)?;

        if !path.exists() {
            return None;
        }

        let mut output_filename = PathBuf::from(&asset.filename);
        output_filename.set_extension(asset.derivate_uti.ext);

        Some(Self {
            asset: asset.clone(),
            source: path,
            destination: output_filename,
            is_derivate: true,
            album_id: None,
            skip: false,
        })
    }
}

impl Display for AssetMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        if self.is_derivate {
            write!(f, "{}", "derivate".bright_magenta())?;
        } else {
            write!(f, "{}", "original".cyan())?;
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
            self.destination.display().to_string().dimmed(),
        )
    }
}

pub fn create_delete_tasks<P, I>(paths: I) -> Vec<ExportTask>
where
    P: Into<PathBuf>,
    I: IntoIterator<Item = P>,
{
    paths
        .into_iter()
        .map(|p| ExportTask::Delete(p.into()))
        .collect()
}
