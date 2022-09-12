use std::collections::HashMap;

pub struct Lagerung {
    kv: HashMap<String, String>,
}

impl Lagerung {
    pub fn has(&mut self, key: &str) -> bool {
        self.kv.contains_key(&key.to_string)
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.kv.insert(key.to_string, value.to_string());
    }

    pub fn remove(&mut self, key: &str) {
        self.kv.remove(&key.to_string);
    }
}
