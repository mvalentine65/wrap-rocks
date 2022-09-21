# use manylinux2014 as base
FROM quay.io/pypa/manylinux2014_x86_64:latest
#RUN yum install epel-release -y
#RUN yum update && yum upgrade
#RUN yum -y install llvm-toolset-7
#COPY entrypoint.sh /usr/bin/entrypoint.sh
#RUN chmod +x /usr/bin/entrypoint.sh
#ENTRYPOINT [ "/usr/bin/entrypoint.sh" ]
#ENV LIBCLANG_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64/ \
 # LIBCLANG_STATIC_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64/ \
  #CLANG_PATH=/opt/rh/llvm-toolset-7/root/usr/bin/clang
#RUN scl enable llvm-toolset-7 bash
#RUN scl_source enable /opt/rh/llvm-toolset-7/usr/bin
#/opt/rh/llvm-toolset-7/
ENV PATH /root/.cargo/bin:$PATH
ENV USER root
# install default rust toolchain
RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH /root/.cargo/bin:$PATH
#RUN source $HOME/.cargo/env
RUN yum -y install yum-plugin-copr
RUN yum -y copr enable alonid/llvm-3.9.0
RUN yum -y install clang-3.9.0
ENV PATH opt/llvm-3.9.0/bin:$PATH
ENV PATH /opt/python/cp35-cp35m/bin/:/opt/python/cp36-cp36m/bin/:/opt/python/cp37-cp37m/bin/:$PATH
#RUN python3.7 -m pip install maturin
#RUN python3.10 -m pip install maturin
ENV LIBCLANG_PATH="/opt/llvm-3.9.0/lib64/"
#COPY wrap-rocks-master /root
ADD ./maturin /maturin
WORKDIR /maturin
RUN rustc -C linkargs
    && mv /maturin/target/debug/maturin /usr/bin/maturin \
    && rm -rf /maturin
WORKDIR /io
#ENTRYPOINT ["/usr/bin/maturin"]
