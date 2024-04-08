use std::collections::HashMap;
use std::fmt::Display;

use ascii_tree::Tree;
use ascii_tree::Tree::{Leaf, Node};
use colored::Colorize;

use crate::model::album::Album;
use crate::model::album::Kind;

pub fn build_tree(albums: &Vec<Album>) -> Tree {
    let root = match albums.iter().find(|a| a.kind == Kind::Root) {
        None => panic!("Library does not contain a root album!"),
        Some(album) => album
    };

    let mut albums_by_parent: HashMap<i32, Vec<&Album>> = HashMap::new();
    albums.iter().for_each(|a| {
        albums_by_parent
            .entry(a.parent_id.unwrap_or(-1))
            .or_insert_with(Vec::new)
            .push(a);
    });

    build_tree_recursively(root, &albums_by_parent)
}

fn build_tree_recursively(album: &Album, albums_by_parent: &HashMap<i32, Vec<&Album>>) -> Tree {
    let children = match albums_by_parent.get(&album.id) {
        None => return Leaf(vec![format!("{album}")]),
        Some(c) => c
    };

    let child_nodes = children
        .iter()
        .map(|a| build_tree_recursively(a, albums_by_parent))
        .collect();

    Node(format!("{album}"), child_nodes)
}

impl Display for Album {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = format!("({})", self.id).yellow();

        let date = format!(
            "{}:",
            match self.start_date {
                None => "<no date>".to_string(),
                Some(d) => d.to_string()
            }
        ).dimmed();

        let name = if self.kind == Kind::Root {
            "<root>".magenta().to_string()
        } else {
            self.name.clone().unwrap_or(String::from("<no name>"))
        };

        write!(f, "{}", format!("{} {} {}", id, date, name))
    }
}