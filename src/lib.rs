use pyo3::prelude::*;
use pyo3::types::PyBytes;
use rust_rocksdb::{
    self, DBCompressionType, DBWithThreadMode, MultiThreaded, WriteOptions,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[pyclass]
pub struct RocksDB {
    // `None` after `close()` has released the handle.  Not `Clone`: a single
    // owner guarantees `close()` actually drops the last reference and frees
    // the lock, rather than leaving a sibling clone holding the DB open.
    pub db: Option<Arc<DBWithThreadMode<MultiThreaded>>>,
    pub wo: Arc<WriteOptions>,
    pub read_only: bool,
}

// Rust-only helpers (not exposed to Python).
impl RocksDB {
    fn handle(&self) -> &DBWithThreadMode<MultiThreaded> {
        self.db
            .as_ref()
            .expect("RocksDB handle used after close()")
    }
}

#[pymethods]

impl RocksDB {
    #[new]
    #[pyo3(signature = (path, compression = None, read_only = None, bulk_load = None))]
    fn new(
        path: String,
        compression: Option<String>,
        read_only: Option<bool>,
        bulk_load: Option<bool>,
    ) -> Self {
        // create directory and all parent directory
        if !Path::new(&path).exists() {
            match fs::create_dir_all(&path) {
                Ok(_) => {}
                Err(_error) => panic!("Failed to create directory at {}.", path),
            };
        }
        let mut opts = rust_rocksdb::Options::default();
        opts.create_if_missing(true);
        // Sizes the background pools on the shared default Env: one set per
        // process, not per handle, and 24 asked for 31 threads in every process
        // holding a DB. 1 is RocksDB's floor -- GetBGJobLimits clamps to
        // max(1, ..) -- so an open still spawns one compaction and one flush
        // thread. Costs ~12% on prepare, the only large writer; the rest are
        // under 50 MB.
        opts.increase_parallelism(1);
        // Compression codec. Previously the `compression` argument was dead: the
        // match below was immediately overridden by an unconditional
        // set_compression_type(Zstd), so "snappy" silently did nothing.
        // The codec is recorded per SST block, so changing it only affects newly
        // written data -- existing DBs stay readable.
        let codec = match compression.as_deref() {
            Some("snappy") => DBCompressionType::Snappy,
            Some("lz4") => DBCompressionType::Lz4,
            Some("none") => DBCompressionType::None,
            Some("zstd") | None => DBCompressionType::Zstd,
            Some(other) => panic!(
                "unknown compression {:?} (expected zstd, lz4, snappy or none)",
                other
            ),
        };
        opts.set_compression_type(codec);
        // Bulk-load mode for write-once, rebuilt-each-run DBs (e.g. prepare's
        // nt store). Auto-compaction during the load reads freshly-flushed L0
        // SSTs back and re-compresses them L0->L1 (pure write amplification off
        // the critical path); disabling it leaves the data as L0 SSTs, which is
        // fine for a DB that is written once and then read a bounded number of
        // times. With compaction off, the default L0 slowdown/stop write
        // triggers (20/36) would throttle then freeze a large load, so raise
        // them out of the way. Durability is unchanged: close() still flushes
        // the memtable to SST.
        if bulk_load.unwrap_or(false) {
            opts.set_disable_auto_compactions(true);
            opts.set_level_zero_slowdown_writes_trigger(1 << 30);
            opts.set_level_zero_stop_writes_trigger(1 << 30);
        }
        opts.set_keep_log_file_num(1);
        let read_only = read_only.unwrap_or(false);
        let unopened_db = || {
            if read_only {
                DBWithThreadMode::open_for_read_only(&opts, &path, false)
            } else {
                DBWithThreadMode::open(&opts, &path)
            }
        };

        let database = match unopened_db() {
            Ok(r) => r,
            Err(e) => panic!("Unable to open RocksDB at {}, error: {}", &path, e),
        };
        let wo = WriteOptions::new();
        RocksDB {
            db: Some(Arc::new(database)),
            wo: Arc::new(wo),
            read_only: read_only,
        }
    }

    /// Release the underlying RocksDB handle.  Once closed, the database is
    /// flushed/closed and any further operation on this object will raise.
    fn close(&mut self) {
        if let Some(db) = self.db.as_ref() {
            // Best-effort flush before releasing the handle; ignore errors
            // (a read-only DB has nothing to flush).
            let _ = db.flush();
        }
        self.db = None;
    }

    fn disable_wal(&mut self) {
        let mut write_option = WriteOptions::new();
        write_option.disable_wal(true);
        self.wo = Arc::new(write_option);
    }
    fn put(&self, header: String, sequence: String) {
        if self.read_only {
            return;
        }
        self.handle()
            .put_opt(header.as_bytes(), sequence.as_bytes(), &self.wo)
            .unwrap();
    }

    fn get(&self, header: String) -> Option<String> {
        let sequence = match self.handle().get(header.as_bytes()) {
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
        self.handle().put_opt(key.as_bytes(), object, &self.wo).unwrap();
    }

    fn get_bytes(&self, py: Python, key: String) -> Py<PyAny> {
        match self.handle().get(key.as_bytes()) {
            Ok(Some(result)) => PyBytes::new(py, &result.as_slice()).into(),
            Ok(None) => return py.None().into(),
            _ => panic!("Received database error when trying to retrieve sequence"),
        }
    }

}

/// A Python module that wraps rocksdb's rust crate.
#[pymodule]
fn wrap_rocks(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RocksDB>()?;
    Ok(())
}
