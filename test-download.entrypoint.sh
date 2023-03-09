#!/bin/bash

# Copy test dir to tmp one because it is read only
mkdir -p /test-tmp
cp -r /test/* /test-tmp/
cd /test-tmp || exit

# Install zerus
mkdir -p /root/zerus
wget https://github.com/wcampbell0x2a/zerus/releases/download/v0.4.0/zerus-x86_64-unknown-linux-musl.tar.gz
tar -xvf zerus-x86_64-unknown-linux-musl.tar.gz -C /root/zerus/
export PATH="$PATH:/root/zerus"

echo "Downloading packages..."
zerus package-mirror ./Cargo.toml

CRATES_PATH=/crates/$(date +%s)
echo "Copy binary packages to $CRATES_PATH..."
mkdir -p "$CRATES_PATH"
cp -r ./package-mirror/crates/* "$CRATES_PATH/"

echo "Finished download under $CRATES_PATH."
