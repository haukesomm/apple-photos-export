use ::ascii_tree::write_tree;

use crate::album_list::ascii_tree::build_tree;
use crate::db::repo::album::AlbumRepository;
use crate::library::PhotosLibrary;
use crate::model::album::Album;
use crate::model::FromDbModel;

mod ascii_tree;

pub fn print_album_tree(library_path: String) {
    let library = PhotosLibrary::new(library_path);
    let album_repository = AlbumRepository::new(library.db_path());

    let db_albums = album_repository.get_all().unwrap_or(Vec::new());

    let album_result: Result<Vec<_>, _> = db_albums
        .iter()
        .map(|a| {
            Album::from_db_model(a.clone())
        })
        .collect();

    if album_result.is_err() {
        eprintln!("At least one album could not be converted from DB model");
        return;
    }
    let albums = album_result.unwrap();

    let tree = build_tree(&albums);

    let mut ascii_tree = String::new();
    let _ = write_tree(&mut ascii_tree, &tree);

    println!("{}", ascii_tree);
}