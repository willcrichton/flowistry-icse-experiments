FROM rust:1.63

WORKDIR /flowistry
RUN apt-get update && apt-get install -y git python3 python3-pip cloc nasm librust-alsa-sys-dev

COPY . ./
RUN pip3 install -r requirements.txt
RUN cd crates/eval && cargo install --path .
RUN cd crates/flowistry && cargo doc --lib && RUSTDOCFLAGS="--html-in-header scripts/katex-header.html" cargo doc --lib --no-deps
