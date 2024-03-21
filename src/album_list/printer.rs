use ascii_tree::write_tree;
use derive_new::new;

use crate::album_list::ascii_tree::build_tree;
use crate::repo::album::AlbumRepository;

#[derive(new)]
pub struct AlbumListPrinter {
    repo: AlbumRepository
}

impl AlbumListPrinter {

    pub fn print_album_tree(&self) {
        let albums = self.repo.get_all().unwrap();
        let tree = build_tree(&albums);

        let mut ascii_tree = String::new();
        let _ = write_tree(&mut ascii_tree, &tree);

        println!("{}", ascii_tree);
    }
}