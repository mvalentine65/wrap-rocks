use core::panic;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use rust_rocksdb::{
    self, DBCompressionType, DBWithThreadMode, MultiThreaded, WaitForCompactOptions, WriteOptions,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[pyclass]
#[derive(Clone)]
pub struct RocksDB {
    pub db: Arc<DBWithThreadMode<MultiThreaded>>,
    pub wo: Arc<WriteOptions>,
    pub read_only: bool,
    
}

#[pymethods]

impl RocksDB {
    #[new]
    #[pyo3(signature = (path, compression = None, read_only = None, max_log_count= None))]
    fn new(path: String, compression: Option<String>, read_only: Option<bool>, max_log_count: Option<usize>) -> Self {
        // create directory and all parent directory
        if !Path::new(&path).exists() {
            match fs::create_dir_all(&path) {
                Ok(_) => {}
                Err(_error) => panic!("Failed to create directory at {}.", path),
            };
        }
        let mut opts = rust_rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.increase_parallelism(24);
        match compression {
            Some(val) if val == "snappy".to_owned() => {
                opts.set_compression_type(DBCompressionType::Snappy)
            }
            _ => opts.set_compression_type(DBCompressionType::Zstd),
        }
        opts.set_compression_type(DBCompressionType::Zstd);
        opts.set_keep_log_file_num(max_log_count.unwrap_or(1));
        let read_only = read_only.unwrap_or(false);
        let unopened_db = || {
            if read_only {
                DBWithThreadMode::open_for_read_only(&opts, &path, false)
            } else {
                DBWithThreadMode::open(&opts, &path)
            }
        };
        //let database = unopened_db()
            //.unwrap_or_else(|e| {
                //panic!(
                    //"Unable to open RocksDB{} at {}, error: {}",
                    //if read_only { " read-only" } else { "" },
                    //&path,
                    //e
                //)
            //});

        //let database = match DBWithThreadMode::open(&opts, &path) {
        let database = match unopened_db() {
            Ok(r) => r,
            Err(e) => panic!("Unable to open RocksDB at {}, error: {}", &path, e),
        };
        let wo = WriteOptions::new();
        RocksDB {
            db: Arc::new(database),
            wo: Arc::new(wo),
            read_only: read_only,
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
        if self.read_only {
            return;
        }
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
        if self.read_only {
            return;
        }
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

    fn batch_put(&self, inserts: HashMap<String, String>) -> u64 {
        if self.read_only {
            return 0;
        }
        let mut batch = rust_rocksdb::WriteBatch::default();
        let mut counter: u64 = 0;
        for (key, value) in inserts.iter() {
            batch.put(key.as_bytes(), value.as_bytes());
            counter += 1;
        }
        match self.db.write_without_wal(batch) {
            Ok(_) => counter,
            Err(_) => 0,
        }
    }

    fn batch_put_bytes(&self, inserts: HashMap<Vec<u8>, Vec<u8>>) -> u64 {
        if self.read_only {
            return 0;
        }
        let mut batch = rust_rocksdb::WriteBatch::default();
        let mut counter: u64 = 0;
        for (key, value) in inserts.iter() {
            batch.put(key, value);
            counter += 1;
        }
        match self.db.write_without_wal(batch) {
            Ok(_) => counter,
            Err(_) => 0,
        }
    }

    fn batch_get<'py>(&self, py: Python<'py>, keys: Vec<String>) -> Bound<'py, PyDict> {
        let byte_keys: Vec<&[u8]> = keys.iter().map(|x| x.as_bytes()).collect();
        let packed_results = self.db.multi_get(&byte_keys);
        let dict = PyDict::new(py);
        for (key, pack) in keys.iter().zip(packed_results.iter()) {
            match pack {
                Ok(Some(value)) => {
                    dict.set_item(key, String::from_utf8(value.to_vec()).unwrap())
                        .unwrap();
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }
        dict
    }

    fn batch_get_bytes<'py>(&self, py: Python<'py>, keys: Vec<Vec<u8>>) -> Bound<'py, PyDict> {
        let byte_keys: Vec<&[u8]> = keys.iter().map(|x| x.as_slice()).collect();
        let packed_results = self.db.multi_get(&byte_keys);
        let dict = PyDict::new(py);
        for (key, pack) in keys.iter().zip(packed_results.iter()) {
            match pack {
                Ok(Some(value)) => {
                    dict.set_item(
                        PyBytes::new(py, key),
                        PyBytes::new(py, value.as_slice()),
                    )
                    .unwrap();
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }
        dict
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
