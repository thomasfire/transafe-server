extern crate chrono;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use chrono::Local;

#[derive(Clone, Debug)]
pub enum StorageError {
    UnknownError,
    NoSuchKeyError,
    InsertError,
}

#[derive(Clone, Debug)]
pub struct LogItem {
    pub ip: String,
    pub time: String,
    pub data: String,
}

#[derive(Clone, Debug)]
pub struct CollectedItems {
    current_index: usize,
    max_size: usize,
    data: Vec<LogItem>,
}

#[derive(Clone, Debug)]
pub struct Storage {
    max_size: usize,
    recent_data: Arc<Mutex<BTreeMap<String, CollectedItems>>>,
}

impl LogItem {
    pub fn new(ip: &str, data: &str) -> Self {
        LogItem { ip: ip.to_string(), time: Local::now().to_string(), data: data.to_string() }
    }
}


impl CollectedItems {
    pub fn new(maxsize: usize) -> Self {
        let mut bufvec: Vec<LogItem> = vec![];
        bufvec.resize(maxsize, LogItem { ip: "".to_string(), time: "".to_string(), data: "".to_string() });
        CollectedItems { current_index: 0, max_size: maxsize, data: bufvec }
    }

    pub fn push(&mut self, item: LogItem) {
        self.current_index += 1;
        self.current_index %= self.max_size;
        self.data[self.current_index] = item;
    }

    pub fn get(&self) -> LogItem {
        self.data[self.current_index].clone()
    }

    pub fn get_all(&self) -> Vec<LogItem> {
        let mut buffer: Vec<LogItem> = vec![];
        buffer.reserve(self.max_size);
        for x in self.current_index..self.current_index + self.max_size {
            buffer.push(self.data[x % self.max_size].clone());
        }
        buffer
    }
}


impl Storage {
    pub fn new(maxsize: usize) -> Self {
        let buffer: BTreeMap<String, CollectedItems> = BTreeMap::new();
        Storage { recent_data: Arc::new(Mutex::new(buffer)), max_size: maxsize }
    }

    pub fn get_all(&self, key: &str) -> Option<Vec<LogItem>> {
        let items = self.recent_data.lock().unwrap().get(key)?.get_all();
        Some(items)
    }

    pub fn get(&self, key: &str) -> Option<LogItem> {
        let items = self.recent_data.lock().unwrap().get(key)?.get();
        Some(items)
    }

    pub fn push(&mut self, key: &str, item: LogItem) -> Result<(), StorageError> {
        self.recent_data.lock().map(|mut x| {
            let itemo = x.entry(key.to_string()).or_insert(CollectedItems::new(self.max_size));
            itemo.push(item);
            ()
        }).map_err(|_| StorageError::UnknownError)
    }

    pub fn erase(&mut self) {
        self.recent_data.lock().unwrap().clear();
    }
}