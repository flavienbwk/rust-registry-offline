#!/bin/bash

# Copy test dir to tmp one because it is read only
mkdir -p /test-tmp
cp -r /test/* /test-tmp/

# Updated cargo configuration
cat <<EOT > "/test-tmp/.cargo/config"
[source.panamax]
registry = "http://172.17.0.1:8090/git/crates.io-index"
[source.panamax-sparse]
registry = "sparse+http://172.17.0.1:8090/index/"

[source.crates-io]
# To use sparse index, change "panamax" to "panamax-sparse".
replace-with = "panamax"
EOT

# Test build with custom cargo config
cd /test-tmp || exit
cargo build
