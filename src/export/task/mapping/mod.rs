pub mod mappers;

use crate::export::task::{AssetMapping, ExportTask};

/// The result of mapping an export task.
///
/// This enum represents the possible outcomes of mapping an `ExportTask`:
/// 1. `Remove`: The task should be removed and not exported.
/// 2. `Map(ExportTask)`: The task has been transformed into a new `ExportTask`.
/// 3. `Split(Vec<ExportTask>)`: The task has been split into multiple `ExportTask`s.
///
/// This allows for flexible handling of export tasks, enabling filtering, transformation,
/// and splitting of tasks as needed.
pub enum TaskMapperResult {
    Remove,
    Map(ExportTask),
    Split(Vec<ExportTask>),
}

/// A trait for mapping export tasks.
///
/// This trait is used to transform an `ExportTask` into another `ExportTask` or filter it out.
/// It is used in the export process to apply various transformations to the tasks before they are
/// executed, e.g. to group them by album, year, or month, or to exclude hidden assets.
///
/// Upon export, all registered mappers are called in the order they were registered.
pub trait MapExportTask {
    fn map_export_task(&self, task: ExportTask) -> TaskMapperResult;
}

/// A trait for mapping asset mappings.
///
/// This trait is used to transform an `AssetMapping` into another `AssetMapping`.
///
/// Since `ExportTask` can contain different variants (e.g., `Copy`, `Delete`), this trait
/// focuses solely on the `AssetMapping` part of the `ExportTask`.
///
/// All implementors of this trait automatically get an implementation of `MapExportTask`
/// that maps only the `Copy` variant of `ExportTask`.
pub trait MapAsset {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping;
}

impl<A: MapAsset> MapExportTask for A {
    fn map_export_task(&self, task: ExportTask) -> TaskMapperResult {
        TaskMapperResult::Map(match task {
            ExportTask::Copy(mapping) => ExportTask::Copy(self.map_asset(mapping)),
            ExportTask::Delete(_) => task,
        })
    }
}
