pub mod memory;
pub mod disk;

use std::{

    collections::HashMap,

    sync::OnceLock,

};

pub struct Dictionary {
    entries: Vec<String>,
    lookup_map: OnceLock<HashMap<String, u8>>,
}

// Useful getters and setters
impl Dictionary {

    pub fn entries(&self) -> &Vec<String> {
        &self.entries
    }

    pub fn set_entries(&mut self, entries: Vec<String>) {
        self.entries = entries;
    }

    pub fn lookup_map<F: FnOnce() -> HashMap<String, u8>>(&self, init_fn: F) -> &HashMap<String, u8> {

        self.lookup_map.get_or_init(init_fn)

    }

}
