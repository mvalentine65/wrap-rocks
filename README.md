# wrap-rocks

**Minimal Python bindings for RocksDB**\
Built with [PyO3](https://pyo3.rs/), powered by [RocksDB](https://github.com/facebook/rocksdb), and packaged via [maturin](https://github.com/PyO3/maturin).

## 🚀 Features

- Fast key-value store backed by RocksDB.
- Pythonic API with support for both strings and bytes.
- Optional compression: `zstd` (default), `lz4`, `snappy`, or `none`.
- WAL (Write-Ahead Log) can be disabled for rebuild-from-scratch databases.
- Lightweight, no-dependency interface for read/write operations.
- Wheels for Python 3.10–3.14 (manylinux-compatible).

---

## 📦 Installation

```bash
pip install wrap-rocks
```

Or, from source (requires Rust and maturin):

```bash
maturin develop
```
## 🐳 Building with Docker

You can build wheels locally using the provided Dockerfile, or use the prebuilt image on Docker Hub.
The container should support every python version from 3.10 to 3.14.

Option 1: Build your own image
```bash
sudo docker build -t wrap-rocks -f docker/Dockerfile .
sudo docker run --rm -v $(pwd):/io wrap-rocks python3.13 -m maturin -i python3.13 build --release
```

Option 2: Use the prebuilt image
```bash
sudo docker pull saferq/wrap-rocks:local
sudo docker run --rm -v $(pwd):/io saferq/wrap-rocks:local python3.13 -m maturin -i python3.13 build --release
```

---

## 🧪 Example

```python
from wrap_rocks import RocksDB

# Open or create the database
db = RocksDB("mydb", compression="snappy")

# Store string data
db.put("seq1", "AGCT")
print(db.get("seq1"))  # "AGCT"

# Store binary data
db.put_bytes("meta", b"\x00\x01")
print(db.get_bytes("meta"))  # b"\x00\x01"

# Release the handle (flushes the memtable to SST)
db.close()
```

---

## ⚖️ API Overview

### `RocksDB(path, compression=None, read_only=None, bulk_load=None)`

- Initializes a RocksDB database at the given path.
- Creates directories automatically if missing.
- `compression`: `"zstd"` (default, best ratio), `"lz4"`, `"snappy"` (fast,
  lightweight), or `"none"`. Recorded per SST block, so changing it affects
  only newly written data -- existing databases stay readable.
- `read_only`: open without taking the write lock. Writes become no-ops.
- `bulk_load`: disable auto-compaction for write-once databases that are
  rebuilt every run. Avoids re-compressing freshly flushed L0 SSTs.

---

### `put(key: str, value: str)`

Stores a string value under a string key.

### `get(key: str) -> Optional[str]`

Retrieves a string value by key. Returns `None` if missing.

---

### `put_bytes(key: str, value: bytes)`

Stores arbitrary binary data under a string key.

### `get_bytes(key: str) -> Optional[bytes]`

Retrieves binary data as `bytes`. Returns `None` if missing.

---

### `close()`

Flushes the memtable to SST and releases the underlying handle, freeing the
database lock. Any further operation on the object raises.

---

### `disable_wal()`

Turns off Write-Ahead Logging for subsequent writes. WAL is **enabled** by
default for durability.

> Skipping the WAL makes writes roughly 3x faster, which is worth it for a
> database rebuilt from scratch on every run. `close()` still flushes the
> memtable to SST, so a completed run is durable. Use with care---data written
> since the last flush may be lost on crash.

---

## ⚙️ License

Licensed under the Apache License, Version 2.0 ([LICENSE](./LICENSE)).

---

## 🔮 Why wrap RocksDB?

This crate was built to expose a **minimal and predictable interface** to RocksDB for Python projects—particularly for use cases like:

- Storing FASTA-style `header:sequence` pairs.
- Staging byte-encoded ML or bioinformatics data.
- Quickly dumping + retrieving structured data with compression.
