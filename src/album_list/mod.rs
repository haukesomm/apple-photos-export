use ::ascii_tree::write_tree;

use crate::album_list::ascii_tree::build_tree;
use crate::db::repo::album::AlbumRepository;
use crate::model::album::Album;
use crate::model::FromDbModel;
use crate::result::PhotosExportResult;

mod ascii_tree;

pub fn print_album_tree(db_path: String) -> PhotosExportResult<()> {
    let album_repository = AlbumRepository::new(db_path);

    let db_albums = album_repository.get_all()?;

    let albums: Vec<Album> = db_albums
        .iter()
        .map(|a| {
            Album::from_db_model(&a)
        })
        .collect::<Result<Vec<Album>, String>>()?;

    let tree = build_tree(&albums);

    let mut ascii_tree = String::new();
    let _ = write_tree(&mut ascii_tree, &tree);

    println!("{}", ascii_tree);

    Ok(())
}