//! Deduplication of export tasks.

use crate::export::task::ExportTask;
use std::collections::HashSet;
use std::path::PathBuf;

/// Removes duplicate `ExportTask::Copy` tasks that would copy the same source
/// file to the same destination path.
///
/// This can happen when `OneTaskPerAlbum` splits an asset into one mapping per
/// album it belongs to (so that album filters can discriminate between them)
/// but no album-based grouping strategy is selected. In that case the surviving
/// per-album copies all share the same flat destination, and exporting the same
/// file to the same path more than once is wasteful (and, with `--skip-existing`,
/// results in redundant skip checks).
///
/// Only tasks with an identical `(source, destination_path)` pair are considered
/// duplicates, so distinct assets that merely happen to share a destination
/// filename are left untouched. `ExportTask::Delete` tasks are passed through
/// unchanged. The first occurrence of each duplicate is kept, preserving the
/// original task order.
pub fn deduplicate_copy_tasks(tasks: Vec<ExportTask>) -> Vec<ExportTask> {
    let mut seen: HashSet<(PathBuf, PathBuf)> = HashSet::new();
    tasks
        .into_iter()
        .filter(|task| match task {
            ExportTask::Copy(mapping) => {
                let key = (mapping.source.clone(), mapping.destination_path());
                seen.insert(key)
            }
            ExportTask::Delete(_) => true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::task::AssetMapping;
    use crate::model::asset::Asset;
    use crate::uti::Uti;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn copy_task(asset_id: i32, album_id: Option<i32>, source: &str, dest_dir: &str) -> ExportTask {
        ExportTask::Copy(AssetMapping {
            asset: Asset {
                id: asset_id,
                uuid: format!("uuid-{asset_id}"),
                dir: "directory".to_string(),
                filename: "image.jpeg".to_string(),
                derivate_uti: Uti::JPEG,
                datetime: chrono::DateTime::UNIX_EPOCH.naive_utc(),
                hidden: false,
                original_filename: "image.jpeg".to_string(),
                has_adjustments: false,
                data_store_subtypes: vec![],
                album_ids: HashSet::new(),
            },
            source: PathBuf::from(source),
            destination_dir: PathBuf::from(dest_dir),
            filename_components: vec!["image".to_string()],
            file_extension: "jpeg".to_string(),
            is_derivate: false,
            album_id,
            skip: false,
            is_part_of_raw_pair: false,
        })
    }

    #[test]
    fn collapses_same_source_same_destination() {
        let tasks = vec![
            copy_task(1, Some(7), "src/image.jpeg", "out"),
            copy_task(1, Some(8), "src/image.jpeg", "out"),
        ];

        let result = deduplicate_copy_tasks(tasks);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn keeps_first_occurrence() {
        let tasks = vec![
            copy_task(1, Some(7), "src/image.jpeg", "out"),
            copy_task(1, Some(8), "src/image.jpeg", "out"),
        ];

        let result = deduplicate_copy_tasks(tasks);
        match &result[0] {
            ExportTask::Copy(m) => assert_eq!(m.album_id, Some(7)),
            _ => panic!("expected a copy task"),
        }
    }

    #[test]
    fn keeps_copies_with_different_destinations() {
        // Album-based grouping puts each copy in its own album folder.
        let tasks = vec![
            copy_task(1, Some(7), "src/image.jpeg", "out/album7"),
            copy_task(1, Some(8), "src/image.jpeg", "out/album8"),
        ];

        let result = deduplicate_copy_tasks(tasks);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn does_not_merge_distinct_assets_sharing_a_destination() {
        // Two different source files resolving to the same destination filename
        // is a filename collision, not a duplicate export, and must be preserved
        // so the user can decide how to handle it (e.g. via --include-asset-ids).
        let tasks = vec![
            copy_task(1, None, "src/a/image.jpeg", "out"),
            copy_task(2, None, "src/b/image.jpeg", "out"),
        ];

        let result = deduplicate_copy_tasks(tasks);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn preserves_delete_tasks() {
        let tasks = vec![
            ExportTask::Delete(PathBuf::from("out/old.jpeg")),
            copy_task(1, Some(7), "src/image.jpeg", "out"),
            ExportTask::Delete(PathBuf::from("out/other.jpeg")),
        ];

        let result = deduplicate_copy_tasks(tasks);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn preserves_order() {
        let tasks = vec![
            copy_task(1, Some(7), "src/one.jpeg", "out"),
            copy_task(2, Some(8), "src/two.jpeg", "out"),
            copy_task(1, Some(9), "src/one.jpeg", "out"), // duplicate of first
            copy_task(3, Some(10), "src/three.jpeg", "out"),
        ];

        let result = deduplicate_copy_tasks(tasks);
        let sources: Vec<PathBuf> = result
            .iter()
            .map(|t| match t {
                ExportTask::Copy(m) => m.source.clone(),
                _ => PathBuf::new(),
            })
            .collect();

        assert_eq!(
            sources,
            vec![
                PathBuf::from("src/one.jpeg"),
                PathBuf::from("src/two.jpeg"),
                PathBuf::from("src/three.jpeg"),
            ]
        );
    }
}
