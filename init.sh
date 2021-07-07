#!/bin/bash

set -e

bold=`tput bold`
reset=`tput sgr0`

function msg() {
    echo "${bold}$1${reset}"
}

msg "WARNING: this could take up to 1-2 hours to run!"

rustup toolchain install nightly

pushd deps

pushd rust
pushd src/bootstrap
msg "Computing build triple"
BUILD_TRIPLE=$(python3 -c "import bootstrap; print(bootstrap.default_build_triple(False))")
popd # src/bootstrap

msg "Building Rust"
./x.py build
RUSTC_BUILD_DIR=$(pwd)/build/${BUILD_TRIPLE}/stage1
rustup toolchain link slicer ${RUSTC_BUILD_DIR}
popd # rust

pushd cargo
msg "Building Cargo"
rustup override set nightly
cargo build --release
ln -s $(pwd)/target/release/cargo ${RUSTC_BUILD_DIR}/bin/
popd # cargo

popd # deps

pushd eval
msg "Building evaluation"
rustup override set slicer
cargo install --path . --offline
popd # eval

exit

# Download dataset
mkdir -p data/repos
pushd data/repos

function co() {
    git clone $1 &&
    pushd $2 &&
    git checkout $3 &&
    popd
}

co https://github.com/SergioBenitez/Rocket Rocket 8d4d01106e2e10b08100805d40bfa19a7357e900 &&
co https://github.com/hyperium/hyper hyper ed2fdb7b6a2963cea7577df05ddc41c56fee7246 &&
co https://github.com/image-rs/image image 6e0cd31a5287dd589d2e78ae33c1f720c77a6863 &&
co https://github.com/dimforge/nalgebra nalgebra 984bb1a63943aa68b6f26ff4a6acf8f68b833b70 &&
co https://github.com/xiph/rav1e rav1e 1b6643324752785e7cd6ad0b19257f3c3a9b2c6a &&
co https://github.com/rayon-rs/rayon/ rayon c571f8ffb4f74c8c09b4e1e6d9979b71b4414d07 &&
co https://github.com/mrDIMAS/rg3d rg3d ca7b85f2b30e45b82caee0591ee1abf65bb3eb00 &&
co https://github.com/ctz/rustls rustls cdf1dada21a537e141d0c6dde9c5685bb43fbc0e &&
co https://github.com/mozilla/sccache sccache 3f318a8675e4c3de4f5e8ab2d086189f2ae5f5cf &&
co https://github.com/RustPython/RustPython RustPython c5d1f4afaaddc0b79b8be4cc923bf2e63791eece
popd

# Install native script dependencies
cargo install cargo-single-pyo3
pushd notebooks
cargo single-pyo3 rs_utils.rs --release
popd

# Install Python script dependencies (DO IT IN A VIRTUAL ENVIRONMENT!)
pip3 install -r requirements.txt
