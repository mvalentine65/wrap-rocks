use core::panic;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use rust_rocksdb::{
    self, DBCompressionType, DBWithThreadMode, MultiThreaded, WaitForCompactOptions, WriteOptions,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[pyclass]
#[derive(Clone)]
pub struct RocksDB {
    pub db: Arc<DBWithThreadMode<MultiThreaded>>,
    pub wo: Arc<WriteOptions>,
}

#[pymethods]
impl RocksDB {
    #[new]
    fn new(path: &str, compression: Option<&str>) -> Self {
        // create directory and all parent directory
        if !Path::new(path).exists() {
            match fs::create_dir_all(path) {
                Ok(_) => {}
                Err(_error) => panic!("Failed to create directory at {}.", path),
            };
        }
        let mut opts = rust_rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.increase_parallelism(24);
        match compression {
            Some("snappy") => opts.set_compression_type(DBCompressionType::Snappy),
            _ => opts.set_compression_type(DBCompressionType::Zstd),
        }
        opts.set_compression_type(DBCompressionType::Zstd);
        let database = match DBWithThreadMode::open(&opts, path) {
            Ok(r) => r,
            Err(e) => panic!("Unable to open RocksDB at {}, error: {}", path, e),
        };
        let wo = WriteOptions::new();
        RocksDB {
            db: Arc::new(database),
            wo: Arc::new(wo),
        }
    }

    fn disable_wal(&mut self) {
        let mut write_option = WriteOptions::new();
        write_option.disable_wal(true);
        self.wo = Arc::new(write_option);
    }
    fn enable_wal(&mut self) {
        let mut write_option = WriteOptions::new();
        write_option.disable_wal(false);
        self.wo = Arc::new(write_option);
    }

    fn put(&self, header: String, sequence: String) {
        self.db
            .put_opt(header.as_bytes(), sequence.as_bytes(), &self.wo)
            .unwrap();
    }

    fn get(&self, header: String) -> Option<String> {
        let sequence = match self.db.get(header.as_bytes()) {
            Ok(Some(r)) => String::from_utf8(r).unwrap(),
            Ok(None) => return None,
            Err(e) => panic!(
                "Received database error when trying to retrieve sequence, error: {}",
                e
            ),
        };

        Some(sequence)
    }

    fn put_bytes(&self, key: String, object: &[u8]) {
        self.db.put_opt(key.as_bytes(), object, &self.wo).unwrap();
    }

    fn get_bytes(&self, py: Python, key: String) -> PyObject {
        match self.db.get(key.as_bytes()) {
            Ok(Some(result)) => PyBytes::new(py, &result.as_slice()).into(),
            Ok(None) => return py.None().into(),
            _ => panic!("Received database error when trying to retrieve sequence"),
        }
    }

    fn delete(&self, header: String) -> bool {
        self.db.delete(header.as_bytes()).is_ok()
    }

    fn batch_put(&self, inserts: Vec<Vec<String>>) -> u64 {
        let mut batch = rust_rocksdb::WriteBatch::default();
        let mut counter: u64 = 0;
        for pair in inserts.iter() {
            batch.put(pair[0].as_bytes(), pair[1].as_bytes());
            counter += 1
        }
        match self.db.write_without_wal(batch) {
            Ok(_) => counter,
            Err(_) => 0,
        }
    }

    fn batch_get(&self, keys: Vec<String>) -> Vec<String> {
        let byte_keys: Vec<&[u8]> = keys.iter().map(|x| x.as_bytes()).collect();
        let packed_results = self.db.multi_get(byte_keys.iter());
        let mut unpacked_results: Vec<String> = Vec::with_capacity(keys.capacity());
        for pack in packed_results.iter() {
            match pack {
                Ok(Some(value)) => {
                    unpacked_results.push(String::from_utf8(value.to_vec()).unwrap())
                }
                Ok(None) => unpacked_results.push(String::from("")),
                Err(_) => unpacked_results.push(String::from("error")),
            }
        }
        return unpacked_results;
    }

    fn flush(&self) -> bool {
        match self.db.flush() {
            Ok(()) => {}
            Err(_) => return false,
        }
        match self.db.wait_for_compact(&WaitForCompactOptions::default()) {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

/// A Python module that wraps rocksdb's rust crate.
#[pymodule]
fn wrap_rocks(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RocksDB>()?;
    Ok(())
}
