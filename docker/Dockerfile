FROM quay.io/pypa/manylinux_2_28_x86_64

RUN dnf install -y clang rocksdb-devel

RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH=/opt/python/cp38-cp38/bin:/opt/python/cp39-cp39/bin:/opt/python/cp310-cp310/bin:/opt/python/cp311-cp311/bin:/opt/python/cp312-cp312/bin:/opt/python/cp313-cp313/bin:/root/.cargo/bin:$PATH

RUN python3.8 -m pip install maturin && \
  python3.9 -m pip install maturin && \
  python3.10 -m pip install maturin && \
  python3.11 -m pip install maturin && \
  python3.12 -m pip install maturin && \
  python3.13 -m pip install maturin

WORKDIR /io

