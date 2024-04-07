use ::ascii_tree::write_tree;

use crate::album_list::ascii_tree::build_tree;
use crate::db::repo::album::AlbumRepository;
use crate::library::PhotosLibrary;

mod ascii_tree;

pub fn print_album_tree(library_path: String) {
    let library = PhotosLibrary::new(library_path);
    let album_repository = AlbumRepository::new(library.db_path());

    let albums = album_repository.get_all().unwrap();
    let tree = build_tree(&albums);

    let mut ascii_tree = String::new();
    let _ = write_tree(&mut ascii_tree, &tree);

    println!("{}", ascii_tree);
}