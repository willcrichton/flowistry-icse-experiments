FROM rust:1.59

WORKDIR /flowistry
COPY . ./
RUN apt-get update && apt-get install -y git python3 python3-pip cloc nasm librust-alsa-sys-dev
RUN pip3 install -r requirements.txt
RUN cd notebooks && cargo install cargo-single-pyo3 && cargo single-pyo3 rs_utils.rs --release
RUN cd crates/eval && cargo install --path .
RUN cd crates/flowistry && cargo doc --lib && RUSTDOCFLAGS="--html-in-header scripts/katex-header.html" cargo doc --lib --no-deps
