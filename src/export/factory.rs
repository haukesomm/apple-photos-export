use crate::export::task::{AssetMapping, ExportTask};
use crate::export::task::mapping::{MapExportTask, TaskMapperResult};
use crate::model::{Asset, Library};

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
    mappers: Vec<Box<dyn MapExportTask + 'a>>
}

impl<'a> ExportTaskFactory<'a> {
    
    /// Creates a new factory that generates export tasks for the original assets only.
    pub fn new_for_originals(library: Library) -> Self {
        Self::new(
            library,
            |lib, asset| vec![ExportTask::Copy(AssetMapping::for_original(lib, asset))]
        )
    }

    /// Creates a new factory that generates export tasks for the derivative assets,
    /// falling back to the original asset if no derivative exists.
    pub fn new_for_derivates_with_fallback(library: Library) -> Self {
        Self::new(
            library,
            |lib, asset| Self::create_derivate_task_with_fallback(lib, asset, false)
        )
    }

    /// Creates a new factory that generates export tasks for both derivative and original assets.
    /// Derivates are included if they exist, and originals are always included as well.
    pub fn new_for_originals_and_derivates(library: Library) -> Self {
        Self::new(
            library,
            |lib, asset| Self::create_derivate_task_with_fallback(lib, asset, true)
        )
    }

    fn new(library: Library, factory: impl Fn(&Library, Asset) -> Vec<ExportTask> + 'a) -> Self {
        Self {
            library,
            factory: Box::new(factory),
            mappers: vec![]
        }
    }
    
    fn create_derivate_task_with_fallback(library: &Library, asset: Asset, always_include_fallback: bool) -> Vec<ExportTask> {
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
        self.mappers
            .iter()
            .fold(vec![task], |vec, mapper| {
                vec.iter()
                    .flat_map(|task| self.recursively_apply_mapper(mapper.as_ref(), task.clone()))
                    .collect()
            })
    }
    
    fn recursively_apply_mapper(&self, mapper: &dyn MapExportTask, task: ExportTask) -> Vec<ExportTask> {
        let result = mapper.map_export_task(task);
        match result {
            TaskMapperResult::Remove => vec![],
            TaskMapperResult::Map(task) => vec![task],
            TaskMapperResult::Split(additional) => {
                additional
                    .into_iter()
                    .flat_map(|task| self.recursively_apply_mapper(mapper, task))
                    .collect()
            }
        }
    }
}