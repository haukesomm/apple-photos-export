use crate::export::{ExportAssetMetadata, ExportAssetRelation, ExportTask};
use crate::model::album::Album;
use crate::model::{Asset, Library};
use std::collections::HashMap;
use std::path::PathBuf;


/// Configuration for the `TasksBuilder`, containing references to the library, assets, albums,
/// and the output directory.
pub struct TasksBuilderConfig<'a> {
    pub library: &'a Library,
    pub assets: &'a Vec<Asset>,
    pub albums: &'a HashMap<i32, Album>,
    pub output_dir: PathBuf,
}

impl<'a> TasksBuilderConfig<'a> {
    /// Creates a new `TasksBuilderConfig` with the specified library, assets, albums, and output
    /// directory.
    pub fn new<P: Into<PathBuf>>(
        library: &'a Library,
        assets: &'a Vec<Asset>,
        albums: &'a HashMap<i32, Album>,
        output_dir: P,
    ) -> Self {
        Self {
            library,
            assets,
            albums,
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
    albums: &'a HashMap<i32, Album>,
    tasks: Box<dyn Iterator<Item = (&'a Asset, ExportTask)> + 'a>,
    modifiers: Vec<Box<dyn Fn(&Asset, &HashMap<i32, Album>, ExportTask) -> Option<ExportTask>>>,
    output_dir: PathBuf,
}

impl<'a> TasksBuilder<'a> {
    
    /// Creates a new `ExportTasksBuilder` that includes all original assets but no derivatives.
    pub fn for_originals(config: TasksBuilderConfig<'a>) -> Self {
        Self {
            albums: config.albums,
            tasks: Box::from(Self::_originals_source(config.library, config.assets)),
            modifiers: vec![],
            output_dir: config.output_dir.into(),
        }
    }

    /// Creates a new `ExportTasksBuilder` that includes all derivatives but no originals.
    pub fn for_derivates(config: TasksBuilderConfig<'a>) -> Self {
        Self {
            albums: config.albums,
            tasks: Box::from(Self::_derivates_source(config.library, config.assets)),
            modifiers: vec![],
            output_dir: config.output_dir.into(),
        }
    }

    /// Creates a new `ExportTasksBuilder` that includes both original and derivative assets.
    pub fn for_originals_and_derivates(config: TasksBuilderConfig<'a>) -> Self {
        Self {
            albums: config.albums,
            tasks: Box::from(
                Self::_originals_source(config.library, config.assets)
                    .chain(Self::_derivates_source(config.library, config.assets)),
            ),
            modifiers: vec![],
            output_dir: config.output_dir.into(),
        }
    }

    fn _originals_source(
        lib: &'a Library,
        assets: &'a Vec<Asset>,
    ) -> impl Iterator<Item = (&'a Asset, ExportTask)> {
        assets.iter().map(move |a| {
            let task = ExportTask {
                source: lib.get_asset_original_path(a),
                destination: PathBuf::from(&a.filename),
                meta: ExportAssetMetadata {
                    asset_id: a.id,
                    derivate: false,
                    relation: ExportAssetRelation::None,
                },
            };
            (a, task)
        })
    }

    fn _derivates_source(
        lib: &'a Library,
        assets: &'a Vec<Asset>,
    ) -> impl Iterator<Item = (&'a Asset, ExportTask)> {
        assets.iter().filter_map(move |a| {
            let path = lib.get_asset_derivate_path(a)?;
            
            if !path.exists() {
                return None;
            }
            
            let mut output_filename = PathBuf::from(&a.filename);
            output_filename.set_extension(a.derivate_uti.ext);
            
            Some((
                a,
                ExportTask {
                    source: path,
                    destination: output_filename,
                    meta: ExportAssetMetadata {
                        asset_id: a.id,
                        derivate: true,
                        relation: ExportAssetRelation::None,
                    }
                },
            ))
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
        let decorated_iter = iter.flat_map(|(asset, task)| {
            if asset.album_ids.is_empty() {
                vec![(asset, task)]
            } else {
                asset
                    .album_ids
                    .iter()
                    .enumerate()
                    .map(|(index, album_id)| ExportTask {
                        meta: ExportAssetMetadata {
                            relation: ExportAssetRelation::AlbumMember {
                                album_id: album_id.clone(),
                                master: if index == 0 {
                                    None
                                } else {
                                    // unwrap may be called here as there is at least one album id
                                    Some(asset.album_ids.first().unwrap().clone())
                                },
                            },
                            ..task.meta
                        },
                        ..task.clone()
                    })
                    .map(|t| (asset, t))
                    .collect::<Vec<(&Asset, ExportTask)>>()
            }
        });

        self.tasks = Box::new(decorated_iter);
    }

    /// Adds a modifier to the list of modifiers to be applied to the export tasks.
    pub fn add_modifier<M>(&mut self, modifier: M)
    where
        M: Fn(&Asset, &HashMap<i32, Album>, ExportTask) -> Option<ExportTask> + 'static
    {
        self.modifiers.push(Box::new(modifier));
    }

    /// Builds the list of export tasks by creating an `ExportTask` for each asset and
    /// applying the specified modifiers.
    ///
    /// Once this method has been called, the builder cannot be used again.
    pub fn build(self) -> Vec<ExportTask> {
        let mut tasks: Vec<ExportTask> = self.tasks
            .filter_map(|(asset, task)| {
                self.modifiers.iter().fold(Some(task), |task, modify| {
                    task.map(|t| modify(asset, self.albums, t)).flatten()
                })
            })
            .map(|task| ExportTask {
                destination: self.output_dir.join(task.destination),
                ..task
            })
            .collect();
        
        tasks.sort_by_key(|t| t.meta.asset_id);
        
        tasks
    }
}
