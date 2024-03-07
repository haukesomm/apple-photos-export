use ascii_tree::write_tree;
use crate::album_list::ascii_tree::build_tree;
use crate::repo::album::AlbumRepository;


pub struct AlbumListPrinter<'a> {
    repo: &'a AlbumRepository
}

impl AlbumListPrinter<'_> {

    pub fn new(repo: &AlbumRepository) -> AlbumListPrinter {
        AlbumListPrinter { repo }
    }

    pub fn print_album_tree(&self) {
        let albums = self.repo.get_all().unwrap();
        let tree = build_tree(&albums);

        let mut ascii_tree = String::new();
        let _ = write_tree(&mut ascii_tree, &tree);

        println!("{}", ascii_tree);
    }
}