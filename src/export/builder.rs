use crate::export::ExportTask;
use crate::model::{Asset, Library};
use std::path::PathBuf;
use crate::export::task_mapper::MapExportTask;

/// Configuration for the `TasksBuilder`, containing references to the library, assets, albums,
/// and the output directory.
pub struct TasksBuilderConfig<'a> {
    pub library: &'a Library,
    pub assets: Vec<Asset>,
    pub output_dir: PathBuf,
}

impl<'a> TasksBuilderConfig<'a> {
    
    /// Creates a new `TasksBuilderConfig` with the specified library, assets, albums, and output
    /// directory.
    pub fn new<P: Into<PathBuf>>(
        library: &'a Library,
        assets: Vec<Asset>,
        output_dir: P,
    ) -> Self {
        Self {
            library,
            assets,
            output_dir: output_dir.into(),
        }
    }
}


/// A builder for creating a list of export tasks from a list of assets.
///
/// This builder allows you to specify which assets to include in the export tasks and
/// apply modifiers to the export tasks.
///
/// Modifiers are functions that take an `Asset` and an `ExportTask` and return a modified
/// `ExportTask`. They are used to customize the export tasks based on the asset's properties
/// and CLI flags.
///
/// If no modifiers are specified, the assets will be exported as-is, with the internal file
/// names.
///
/// This builder is designed to work _lazily_, meaning that it does not create the export tasks
/// until the `build` method is called. Once `build` has been called, the builder cannot be
/// used again.
pub struct TasksBuilder<'a> {
    tasks: Box<dyn Iterator<Item = ExportTask> + 'a>,
    mappers: Vec<Box<dyn MapExportTask + 'a>>,
    output_dir: PathBuf,
}

impl<'a> TasksBuilder<'a> {
    
    /// Creates a new `ExportTasksBuilder` that includes all original assets but no derivatives.
    pub fn for_originals(config: TasksBuilderConfig<'a>) -> Self {
        Self {
            tasks: Box::from(Self::_originals_source(config.library, config.assets)),
            mappers: vec![],
            output_dir: config.output_dir.into(),
        }
    }

    /// Creates a new `ExportTasksBuilder` that includes all derivatives and falls back to the
    /// original version of the asset if no derivate exists.
    pub fn for_derivates_with_fallback(config: TasksBuilderConfig<'a>) -> Self {
        Self {
            tasks: Box::from(Self::_derivates_source_with_fallback(config.library, config.assets)),
            mappers: vec![],
            output_dir: config.output_dir.into(),
        }
    }

    /// Creates a new `ExportTasksBuilder` that includes both original and derivative assets.
    pub fn for_originals_and_derivates(config: TasksBuilderConfig<'a>) -> Self {
        Self {
            tasks: Box::from(
                Self::_originals_source(config.library, config.assets.clone())
                    .chain(Self::_derivates_source(config.library, config.assets)),
            ),
            mappers: vec![],
            output_dir: config.output_dir.into(),
        }
    }
    

    fn _originals_source(
        lib: &'a Library,
        assets: Vec<Asset>,
    ) -> impl Iterator<Item = ExportTask> + 'a {
        assets.into_iter().map(move |a| ExportTask::for_original(lib, a))
    }

    fn _derivates_source(
        lib: &'a Library,
        assets: Vec<Asset>,
    ) -> impl Iterator<Item = ExportTask> + 'a {
        assets.into_iter().filter_map(move |a| ExportTask::for_derivate(lib, a))
    }
    
    fn _derivates_source_with_fallback(
        lib: &'a Library,
        assets: Vec<Asset>,
    ) -> impl Iterator<Item = ExportTask> + 'a {
        assets.into_iter().filter_map(|a| {
            if a.has_adjustments {
                ExportTask::for_derivate(lib, a)
            } else {
                Some(ExportTask::for_original(lib, a))
            }
        })
    }
    

    /// Configures the builder to create a dedicated export task for each album an asset is part of.
    ///
    /// By default, the builder creates export tasks of type `ExportAssetType::Default`, which means
    /// exactly _one_ task is generated _for each_ asset variant included in the export.
    /// This means that assets which are part of multiple albums will still only be exported to one
    /// destination.
    ///
    /// In order to be able to produce album-oriented export structures, there also needs to be an
    /// export task for each album an asset is part of. If called, assets that are part of at least
    /// one album will instead result in a task of type `ExportAssetType::AlbumMember` with
    /// additional metadata, so they can correctly be handled at a later point.
    ///
    /// Since this behavior is only needed for the above-mentioned type of scenario and may
    /// otherwise introduce unwanted complexity or even unintended behavior, it is _turned off by
    /// default_.
    pub fn create_per_album_tasks(&mut self) {
        // Take ownership of the internally stored tasks iterator
        let iter = std::mem::replace(&mut self.tasks, Box::from(Vec::new().into_iter()));

        // Split up export tasks of assets that are part of one or more albums.
        // This way they can later be properly handled and for example be exported to multiple
        // destinations
        let decorated_iter = iter.flat_map(|task| {
            if task.asset.album_ids.is_empty() {
                vec![task]
            } else {
                task.asset
                    .album_ids
                    .iter()
                    .map(|album_id| ExportTask {
                        album_id: Some(album_id.clone()),
                        ..task.clone()
                    })
                    .collect::<Vec<ExportTask>>()
            }
        });

        self.tasks = Box::new(decorated_iter);
    }

    /// Adds a modifier to the list of modifiers to be applied to the export tasks.
    pub fn add_mapper<M: MapExportTask + 'a>(&mut self, mapper: M) {
        self.mappers.push(Box::new(mapper));
    }

    /// Builds the list of export tasks by creating an `ExportTask` for each asset and
    /// applying the specified modifiers.
    ///
    /// Once this method has been called, the builder cannot be used again.
    pub fn build(self) -> Vec<ExportTask> {
        let mut tasks: Vec<ExportTask> = self.tasks
            .filter_map(|task| {
                self.mappers.iter().fold(Some(task), |task, mapper| {
                    task.map(|t| mapper.map(t)).flatten()
                })
            })
            .map(|task| ExportTask {
                destination: self.output_dir.join(task.destination),
                ..task
            })
            .collect();
        
        tasks.sort_by_key(|t| t.asset.id);
        
        tasks
    }
}
