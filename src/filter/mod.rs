use std::{collections::HashSet, sync::{Arc, Mutex}};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BLACKLIST: Arc<Blacklist> = Blacklist::new();
}

const VENDOR_PATH: &'static str = "./vendor.list";
const BLACKLIST_PATH: &'static str = "./black.list";

pub struct BLContent {
    cache: HashSet<String>,
    vendors: HashSet<String>,
}

impl BLContent {
    pub fn new() -> Self {
        let mut blacklist = Self {
            cache: HashSet::new(),
            vendors: HashSet::new(),
        };
        blacklist.update();

        blacklist
    }

    pub fn is_blocked(&self, value: &String) -> bool {
        self.cache.contains(value)
    }

    pub fn update(&mut self) {
        let vendors: Vec<String> = std::fs::read_to_string(VENDOR_PATH).expect("Failed to load vendor file.").split("\n").map(String::from).collect();
        let blacklist: Vec<String> = std::fs::read_to_string(BLACKLIST_PATH).expect("Failed to load blacklist file.").split("\n").map(String::from).collect();

        self.vendors = HashSet::from_iter(vendors.iter().map(String::to_string));
        self.cache = HashSet::from_iter(blacklist.iter().map(String::to_string));
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