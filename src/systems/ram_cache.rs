use std::collections::HashMap;

// Each HashMap is a key-value store with time-to-live (TTL) in seconds in value
#[derive(Clone, Debug)]
pub struct RamCache {
    pub images: HashMap<String, (Vec<u8>, usize)>,
    pub texts: HashMap<String, (String, usize)>,
}

impl RamCache {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            texts: HashMap::new(),
        }
    }

    pub fn calc_ttl(ttl_in_seconds: usize) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        now + ttl_in_seconds
    }

    pub fn get_image(&self, key: &str) -> Option<&Vec<u8>> {
        if let Some((value, _)) = self.images.get(key) {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&String> {
        if let Some((value, _)) = self.texts.get(key) {
            Some(value)
        } else {
            None
        }
    }

    pub fn set_image(&mut self, key: &str, value: &Vec<u8>, ttl: usize) {
        self.images
            .insert(key.to_string(), (value.to_vec(), RamCache::calc_ttl(ttl)));
    }

    pub fn set_str(&mut self, key: &str, value: String, ttl: usize) {
        self.texts
            .insert(key.to_string(), (value, RamCache::calc_ttl(ttl)));
    }

    pub fn gc(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        self.images.retain(|_, (_, ttl)| *ttl > now);
        self.texts.retain(|_, (_, ttl)| *ttl > now);
    }
}
