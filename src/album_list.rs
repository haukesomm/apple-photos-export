use crate::model::album::Album;
use colored::Colorize;
use std::collections::HashMap;

/// Prints the given list of albums as a tree structure to the console.
pub fn print_album_tree(albums: &Vec<Album>) -> crate::Result<()> {
    let tree = build_tree(albums)?;

    let mut buffer = String::new();
    ascii_tree::write_tree(&mut buffer, &tree)?;
    println!("{}", buffer);

    Ok(())
}

fn build_tree(albums: &Vec<Album>) -> crate::Result<ascii_tree::Tree> {
    let root = albums
        .iter()
        .find(|a| a.is_root_album())
        .ok_or("Library does not contain a root album!")?;

    let mut albums_by_parent: HashMap<i32, Vec<&Album>> = HashMap::new();
    albums.iter().for_each(|a| {
        albums_by_parent
            .entry(a.parent_id.unwrap_or(-1))
            .or_insert_with(Vec::new)
            .push(a);
    });

    Ok(build_tree_recursively(root, &albums_by_parent))
}

fn build_tree_recursively(
    album: &Album,
    albums_by_parent: &HashMap<i32, Vec<&Album>>,
) -> ascii_tree::Tree {
    let album_label = format_album(album);

    let children = match albums_by_parent.get(&album.id) {
        None => return ascii_tree::Tree::Leaf(vec![album_label]),
        Some(child_albums) => child_albums,
    };

    let child_nodes = children
        .iter()
        .map(|a| build_tree_recursively(a, albums_by_parent))
        .collect();

    ascii_tree::Tree::Node(album_label, child_nodes)
}

fn format_album(album: &Album) -> String {
    let id = format!("({})", album.id).yellow();

    let date = format!(
        "{}:",
        match album.start_date {
            None => "<no date>".to_string(),
            Some(d) => d.to_string(),
        }
    )
    .dimmed();

    let name = if album.is_root_album() {
        "<root>".magenta().to_string()
    } else {
        album.name.clone().unwrap_or(String::from("<no name>"))
    };

    format!("{} {} {}", id, date, name)
}
