# wrap-rocks

**Minimal Python bindings for RocksDB**\
Built with [PyO3](https://pyo3.rs/), powered by [RocksDB](https://github.com/facebook/rocksdb), and packaged via [maturin](https://github.com/PyO3/maturin).

## 🚀 Features

- Fast key-value store backed by RocksDB.
- Pythonic API with support for both strings and bytes.
- Optional compression: `zstd` (default) or `snappy`.
- Toggleable WAL (Write-Ahead Log) behavior.
- Lightweight, no-dependency interface for read/write/delete operations.
- Wheels for Python 3.8–3.13 (manylinux-compatible).

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
The container should support every python version from 3.8 to 3.13.

Option 1: Build your own image
```bash
sudo docker build -t wrap-rocks -f docker/Dockerfile .
sudo docker run --rm -v $(pwd):/io wrap-rocks python3.13 -m maturin -i python3.13 build --release
```

Option 2: Use the prebuilt image
```bash
sudo docker pull saferq/wrap-rocks
sudo docker run --rm -v $(pwd):/io saferq/wrap-rocks python3.13 -m maturin -i python3.13 build --release
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

# Remove a key
db.delete("seq1")

# Flush and compact
db.flush()
```

---

## ⚖️ API Overview

### `RocksDB(path: str, compression: Optional[str] = None)`

- Initializes a RocksDB database at the given path.
- Creates directories automatically if missing.
- Compression options:
  - `"snappy"`: fast, lightweight
  - Default is `"zstd"`: higher compression ratio

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

### `delete(key: str) -> bool`

Removes a key (and its value). Returns `True` on success.

---

### `flush() -> bool`

Flushes in-memory writes to disk and triggers compaction.

Use this before backups or to minimize storage bloat after bulk writes.

---

### `enable_wal()` and `disable_wal()`

Toggles Write-Ahead Logging. By default, WAL is **enabled** for durability.

> Disabling WAL can improve write performance or help avoid data duplication after breaking changes in schema or logic. Use with care—data may be lost on crash.

---

## ⚙️ License

Licensed under the Apache License, Version 2.0 ([LICENSE](./LICENSE)).

---

## 🔮 Why wrap RocksDB?

This crate was built to expose a **minimal and predictable interface** to RocksDB for Python projects—particularly for use cases like:

- Storing FASTA-style `header:sequence` pairs.
- Staging byte-encoded ML or bioinformatics data.
- Quickly dumping + retrieving structured data with compression.