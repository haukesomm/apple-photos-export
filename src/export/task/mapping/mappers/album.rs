use crate::export::task::mapping::MapAsset;
use crate::export::task::AssetMapping;
use crate::model::album::Album;
use std::collections::HashMap;
use std::path::PathBuf;

/// A mapper that groups assets by album.
pub struct ByAlbum {
    albums: HashMap<i32, Album>,
    max_depth: usize,
}

impl ByAlbum {
    pub fn new(albums: HashMap<i32, Album>, max_depth: usize) -> Self {
        Self {
            albums,
            max_depth: std::cmp::min(max_depth, 255),
        }
    }

    fn build_album_path_recursively(&self, id: i32, depth: usize) -> PathBuf {
        let album_optional = self.albums.get(&id);

        if depth == 0 || album_optional.is_none() || album_optional.unwrap().parent_id.is_none() {
            return PathBuf::new();
        }

        let album = album_optional.unwrap();
        let parent = self.build_album_path_recursively(album.parent_id.unwrap(), depth - 1);

        parent.join(album.name.clone().unwrap_or("_unknown_".to_string()))
    }
}

impl<'a> MapAsset for ByAlbum {
    fn map_asset(&self, mapping: AssetMapping) -> AssetMapping {
        if let Some(album_id) = mapping.album_id {
            let album_path = self.build_album_path_recursively(album_id, self.max_depth);
            AssetMapping {
                destination_dir: PathBuf::from(album_path).join(&mapping.destination_dir),
                ..mapping
            }
        } else {
            mapping
        }
    }
}
