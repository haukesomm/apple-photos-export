//! This module contains functions that modify export tasks based on the associated asset and
//! other metadata.
//!
//! These functions are used to customize the export tasks before they are executed and can be
//! used to filter out certain assets, change the destination path, or modify the metadata.
//!
//! The functions are designed to be used with the `ExportTask` struct and can be applied to
//! export tasks in a chain. Each function takes an `Asset`, a reference to a `HashMap` of
//! `Album`s, and an `ExportTask`, and returns an `Option<ExportTask>`. If the function
//! returns `None`, the task is filtered out and not executed. If it returns `Some(task)`,
//! the task is modified and executed.
//!
//! Apply these functions to the export tasks in the order you want them to be applied via
//! `TasksBuilder::add_modifier`.

use crate::export::{ExportAssetRelation, ExportTask};
use crate::model::album::Album;
use crate::model::Asset;
use chrono::Datelike;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Filters out hidden assets from the export tasks.
pub fn exclude_hidden(
    asset: &Asset,
    _: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    if asset.hidden {
        None
    } else {
        Some(task)
    }
}

/// Prefixes the destination path with "_hidden" if the asset is hidden.
pub fn prefix_hidden_assets(
    asset: &Asset,
    _: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    Some(if asset.hidden {
        ExportTask {
            destination: PathBuf::from("_hidden").join(&task.destination),
            ..task
        }
    } else {
        task
    })
}

/// Adds a suffix to the destination path's filename to be able to distinguish between
/// original and derivative files.
pub fn mark_originals_and_derivates(
    _: &Asset,
    _: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    let mut dest = task.destination;
    let ext = String::from(
        dest.extension()
            .unwrap_or(&OsStr::new(""))
            .to_string_lossy(),
    );

    dest.set_extension(if task.meta.derivate {
        format!("derivate.{}", ext)
    } else {
        format!("original.{}", ext)
    });

    Some(ExportTask {
        destination: dest,
        ..task
    })
}

/// Restores the original filename of the asset in the export task.
pub fn restore_original_filename(
    asset: &Asset,
    _: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    let original_extension = task.destination.extension().clone();

    let mut destination = PathBuf::from(&task.destination);

    destination.set_file_name(&asset.original_filename);
    // Restore original extension or remove it if the original destination did not have one
    destination.set_extension(&original_extension.unwrap_or(OsStr::new("")));

    Some(ExportTask {
        destination,
        ..task
    })
}

/// Adds a prefix to the destination path based on the album the asset is part of.
///
/// This modifier includes the full album path, starting from the root album.
/// Use `structure_by_album` to limit the depth of the album path to only include the first
/// level of albums.
pub fn structure_by_album_recursively(
    _: &Asset,
    albums: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    _structure_by_album(albums, task, 255)
}

/// Adds a prefix to the destination path based on the album the asset is part of.
///
/// This modifier only includes the first level of albums in the path.
/// Use `structure_by_album_recursively` to include the full album path.
pub fn structure_by_album(
    _: &Asset,
    albums: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    _structure_by_album(albums, task, 1)
}

pub fn _structure_by_album(
    albums: &HashMap<i32, Album>,
    task: ExportTask,
    max_depth: u8,
) -> Option<ExportTask> {
    if let ExportAssetRelation::AlbumMember { album_id, .. } = task.meta.relation {
        let album_path = _build_album_path_recursively(albums, &album_id, max_depth);
        Some(ExportTask {
            destination: PathBuf::from(album_path).join(&task.destination),
            ..task
        })
    } else {
        Some(task)
    }
}

fn _build_album_path_recursively(albums: &HashMap<i32, Album>, id: &i32, depth: u8) -> PathBuf {
    let album_optional = albums.get(&id);

    if depth == 0 || album_optional.is_none() || album_optional.unwrap().parent_id.is_none() {
        return PathBuf::new();
    }

    let album = album_optional.unwrap();
    let parent = _build_album_path_recursively(albums, &album.parent_id.unwrap(), depth - 1);

    parent.join(album.name.clone().unwrap_or("_unknown_".to_string()))
}

/// Prefixes the destination path with the asset's year and month.
pub fn prefix_with_asset_year_and_month(
    asset: &Asset,
    _: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    let mut prefix = PathBuf::new();
    prefix.push(asset.datetime.year().to_string());
    prefix.push(format!("{:>02}", asset.datetime.month()));

    Some(ExportTask {
        destination: PathBuf::from(prefix).join(&task.destination),
        ..task
    })
}

/// Prefixes the destination path with the album's year and month if the asset is part of an
/// album. If the asset is not part of an album, it prefixes with the asset's year and month.
pub fn prefix_with_album_year_and_month(
    asset: &Asset,
    albums: &HashMap<i32, Album>,
    task: ExportTask,
) -> Option<ExportTask> {
    match &task.meta.relation {
        ExportAssetRelation::None => prefix_with_asset_year_and_month(asset, albums, task),
        ExportAssetRelation::AlbumMember { album_id, .. } => {
            let album = albums.get(&album_id)?;

            let mut prefix = PathBuf::new();
            if let Some(date) = album.start_date {
                prefix.push(date.year().to_string());
                prefix.push(format!("{:>02}", date.month()))
            }

            Some(ExportTask {
                destination: PathBuf::from(prefix).join(&task.destination),
                ..task
            })
        }
    }
}

pub enum AlbumFilterMode {
    Include,
    Exclude,
}

/// Creates a modifier that either includes or excludes assets based on their album IDs.
/// 
/// __Note:__ If this modifier is used in combination with the album-based output structure,
/// directories and asset files for all albums will be created during the export, even if only one 
/// of them is included in the given IDs.
pub fn create_album_filtering_modifier(
    ids: Vec<i32>,
    filter: AlbumFilterMode
) -> impl Fn(&Asset, &HashMap<i32, Album>, ExportTask) -> Option<ExportTask> {
    move |asset: &Asset, _: &HashMap<i32, Album>, task: ExportTask| {
        let is_part_of_any = ids.iter().any(|i| asset.album_ids.contains(i));
        
        let include = match filter {
            AlbumFilterMode::Include => is_part_of_any,
            AlbumFilterMode::Exclude => !is_part_of_any,
        };
        
        if include {
            Some(task)
        } else {
            None
        }
    }
}
