use crate::export::task::mapping::{MapExportTask, TaskMapperResult};
use crate::export::task::{AssetMapping, ExportTask};
use crate::model::Library;
use crate::model::asset::Asset;

/// A factory to create export tasks for a given set of assets.
///
/// The factory works like a pipeline:
/// 1. It uses a factory function to create initial export tasks for each asset.
/// 2. It applies a series of mappers to each task to transform or filter them. This is where the
///   main customization happens (e.g., grouping by album, excluding hidden assets, etc.).
/// 3. Finally, it converts the destination paths of the tasks to absolute paths based on the
///   provided output directory.
pub struct ExportTaskFactory<'a> {
    library: Library,
    factory: Box<dyn (Fn(&Library, Asset) -> Vec<ExportTask>) + 'a>,
    mappers: Vec<Box<dyn MapExportTask + 'a>>,
}

impl<'a> ExportTaskFactory<'a> {
    /// Creates a new factory that generates export tasks for the original assets only.
    pub fn new_for_originals(library: Library) -> Self {
        Self::new(library, |lib, asset| {
            vec![ExportTask::Copy(AssetMapping::for_original(lib, asset))]
        })
    }

    /// Creates a new factory that generates export tasks for the derivative assets,
    /// falling back to the original asset if no derivative exists.
    pub fn new_for_derivates_with_fallback(library: Library) -> Self {
        Self::new(library, |lib, asset| {
            Self::create_derivate_task_with_fallback(lib, asset, false)
        })
    }

    /// Creates a new factory that generates export tasks for both derivative and original assets.
    /// Derivates are included if they exist, and originals are always included as well.
    pub fn new_for_originals_and_derivates(library: Library) -> Self {
        Self::new(library, |lib, asset| {
            Self::create_derivate_task_with_fallback(lib, asset, true)
        })
    }

    fn new(library: Library, factory: impl Fn(&Library, Asset) -> Vec<ExportTask> + 'a) -> Self {
        Self {
            library,
            factory: Box::new(factory),
            mappers: vec![],
        }
    }

    fn create_derivate_task_with_fallback(
        library: &Library,
        asset: Asset,
        always_include_fallback: bool,
    ) -> Vec<ExportTask> {
        let mut vec: Vec<ExportTask> = vec![];

        if asset.has_adjustments {
            if let Some(mapping) = AssetMapping::for_derivate(library, asset.clone()) {
                vec.push(ExportTask::Copy(mapping));
            }
        }

        if !asset.has_adjustments || always_include_fallback {
            vec.push(ExportTask::Copy(AssetMapping::for_original(library, asset)));
        }

        vec
    }

    /// Adds a mapper to the factory's pipeline.
    pub fn add_mapper(&mut self, mapper: impl MapExportTask + 'a) {
        self.mappers.push(Box::new(mapper));
    }

    /// Builds the export tasks for the given assets by applying the factory function and
    /// mappers in sequence.
    pub fn build(self, assets: Vec<Asset>) -> Vec<ExportTask> {
        assets
            .into_iter()
            .flat_map(|asset| (self.factory)(&self.library, asset))
            .flat_map(|task| self.apply_mappers(task))
            .collect()
    }

    fn apply_mappers(&self, task: ExportTask) -> Vec<ExportTask> {
        self.mappers.iter().fold(vec![task], |vec, mapper| {
            vec.iter()
                .flat_map(|task| self.recursively_apply_mapper(mapper.as_ref(), task.clone()))
                .collect()
        })
    }

    fn recursively_apply_mapper(
        &self,
        mapper: &dyn MapExportTask,
        task: ExportTask,
    ) -> Vec<ExportTask> {
        let result = mapper.map_export_task(task);
        match result {
            TaskMapperResult::Remove => vec![],
            TaskMapperResult::Map(task) => vec![task],
            TaskMapperResult::Split(additional) => additional
                .into_iter()
                .flat_map(|task| self.recursively_apply_mapper(mapper, task))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::task::mapping::mappers;
    use crate::model::Library;
    use crate::model::asset::Asset;
    use crate::uti::Uti;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn asset(id: i32, album_ids: &[i32]) -> Asset {
        Asset {
            id,
            uuid: format!("uuid-{id}"),
            dir: format!("dir-{id}"),
            filename: format!("image-{id}.jpeg"),
            derivate_uti: Uti::JPEG,
            datetime: chrono::DateTime::UNIX_EPOCH.naive_utc(),
            hidden: false,
            original_filename: format!("image-{id}.jpeg"),
            has_adjustments: false,
            data_store_subtypes: vec![],
            album_ids: HashSet::from_iter(album_ids.iter().copied()),
        }
    }

    /// Demo use case: `--include-by-album 42` without any album-based grouping
    /// strategy. Before the fix, `OneTaskPerAlbum` was never added to the
    /// pipeline in this configuration, so every `AssetMapping` had
    /// `album_id: None` and `FilterByAlbumId` matched nothing — the filter was
    /// a silent no-op. With `OneTaskPerAlbum` added, each asset is split into
    /// one mapping per album and the filter keeps only the copies whose album
    /// id matches.
    #[test]
    fn include_by_album_filters_without_album_grouping() {
        let library = Library::new(PathBuf::from("/library"));
        let mut builder = ExportTaskFactory::new_for_originals(library);

        // Mirror main.rs: no grouping strategy selected, but an album filter is
        // active, so OneTaskPerAlbum has to run before the filter.
        builder.add_mapper(mappers::OneTaskPerAlbum);
        builder.add_mapper(mappers::filter::FilterByAlbumId::new(
            vec![42],
            mappers::filter::AlbumFilterMode::Include,
        ));

        // Asset 1 is in albums 7 and 42 -> only the album-42 copy survives.
        // Asset 2 is only in album 7 -> dropped entirely.
        // Asset 3 is in no album -> passes through with album_id None.
        let tasks = builder.build(vec![asset(1, &[7, 42]), asset(2, &[7]), asset(3, &[])]);

        let album_ids: Vec<Option<i32>> = tasks
            .iter()
            .map(|t| match t {
                ExportTask::Copy(m) => m.album_id,
                ExportTask::Delete(_) => None,
            })
            .collect();

        assert_eq!(album_ids, vec![Some(42), None]);
    }

    /// Demo use case: `--exclude-by-album 42` without album-based grouping. An
    /// asset that lives in both a matching and a non-matching album keeps its
    /// non-matching copy and drops the matching one.
    #[test]
    fn exclude_by_album_filters_without_album_grouping() {
        let library = Library::new(PathBuf::from("/library"));
        let mut builder = ExportTaskFactory::new_for_originals(library);

        builder.add_mapper(mappers::OneTaskPerAlbum);
        builder.add_mapper(mappers::filter::FilterByAlbumId::new(
            vec![42],
            mappers::filter::AlbumFilterMode::Exclude,
        ));

        let tasks = builder.build(vec![asset(1, &[7, 42])]);

        let album_ids: Vec<Option<i32>> = tasks
            .iter()
            .map(|t| match t {
                ExportTask::Copy(m) => m.album_id,
                ExportTask::Delete(_) => None,
            })
            .collect();

        assert_eq!(album_ids, vec![Some(7)]);
    }

    /// Demo use case for the dedup fix: `--exclude-by-album 42` without
    /// album-based grouping, applied to an asset that lives in albums 7, 8 and
    /// 42. `OneTaskPerAlbum` splits the asset into three per-album copies so the
    /// filter can drop the album-42 copy, leaving copies for albums 7 and 8.
    /// Without album-based grouping both survivors resolve to the same flat
    /// destination, so the asset would be exported twice to the same path.
    /// `deduplicate_copy_tasks` collapses them into a single export.
    #[test]
    fn exclude_by_album_without_grouping_yields_single_destination() {
        let library = Library::new(PathBuf::from("/library"));
        let mut builder = ExportTaskFactory::new_for_originals(library);

        builder.add_mapper(mappers::OneTaskPerAlbum);
        builder.add_mapper(mappers::filter::FilterByAlbumId::new(
            vec![42],
            mappers::filter::AlbumFilterMode::Exclude,
        ));

        let tasks = builder.build(vec![asset(1, &[7, 8, 42])]);

        // Before dedup: two copies (album 7 and album 8) sharing one destination.
        let destinations: Vec<std::path::PathBuf> = tasks
            .iter()
            .map(|t| match t {
                ExportTask::Copy(m) => m.destination_path(),
                ExportTask::Delete(_) => std::path::PathBuf::new(),
            })
            .collect();
        assert_eq!(destinations.len(), 2);
        assert_eq!(destinations[0], destinations[1]);

        // After dedup: a single export task remains.
        let deduped = crate::export::task::dedup::deduplicate_copy_tasks(tasks);
        assert_eq!(deduped.len(), 1);
    }
}
