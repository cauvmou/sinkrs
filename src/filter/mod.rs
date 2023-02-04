use std::{collections::{HashSet}, sync::{Arc, Mutex}};
use lazy_static::lazy_static;

use self::{tree::{Node, NodeTree}};

mod tree;
mod hasher;

lazy_static! {
    pub static ref BLACKLIST: Arc<Blacklist> = Blacklist::new();
}

const VENDOR_PATH: &'static str = "./vendor.list";
const BLACKLIST_PATH: &'static str = "./black.list";


pub struct BLContent {
    nodes: HashSet<Arc<Node>>,
    tree: NodeTree,
    vendors: HashSet<String>,
}

impl BLContent {
    pub fn new() -> Self {
        let mut blacklist = Self {
            nodes: HashSet::new(),
            tree: NodeTree::new(),
            vendors: HashSet::new(),
        };
        blacklist.update();
        blacklist
    }

    pub fn is_blocked(&self, value: &String) -> bool {
        let result = self.tree.is_branch_blocking(value.split(".").map(String::from).collect::<Vec<String>>());
        result
    }

    pub fn update(&mut self) {
        let vendors: Vec<String> = std::fs::read_to_string(VENDOR_PATH).expect("Failed to load vendor file.").split("\n").map(String::from).collect();
        let blacklist: Vec<String> = std::fs::read_to_string(BLACKLIST_PATH).expect("Failed to load blacklist file.").split("\n").map(String::from).collect();

        self.vendors = HashSet::from_iter(vendors.iter().map(String::to_string));
        self.cache(blacklist);
    }

    pub fn cache(&mut self, lines: Vec<String>) {
        lines.iter().filter(|s| !s.starts_with("#")).for_each(|s| {
            let segments = s.split(".").map(String::from).collect::<Vec<String>>();
            let nodes = segments.iter().enumerate().map(|(i, s)| {
                if s.as_str() == "*" {
                    Node::Wildcard
                } else {
                    Node::Domain { name: s.to_owned(), blocking: i == 0 }
                }
            }).map(|node| {
                let arc = Arc::new(node);
                self.nodes.insert(arc.clone());
                arc
            }).collect::<Vec<Arc<Node>>>();
            self.tree = self.tree.clone().branch(nodes);
        });
    }
}

pub struct Blacklist(Mutex<BLContent>);

impl Blacklist {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(Mutex::new(BLContent::new())))
    }

    pub fn is_blocked(&self, value: &String) -> bool {
        self.0.lock().expect("Failed to grab lock.").is_blocked(value)
    }

    pub fn update(&self) {
        self.0.lock().expect("Failed to grab lock.").update()
    }
}