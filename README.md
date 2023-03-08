# Rust Registry offline

Run your own crate registry offline for Rust. Get instructions to download and push crates. Perfect for long plane trips ! :airplane:

## Init the server (online)

1. Build the server from [Panamax's repo](https://github.com/panamax-rs/panamax)

    ```bash
    docker-compose build
    ```

2. Init the server

    ```bash
    docker-compose -f init.docker-compose.yml run registry_init
    ```

3. Replace default values

    ```bash
    export SERVER_BASE_URL=http://172.17.0.1:8080/crates
    sed -i "s|^base_url = .*|base_url = \"$SERVER_BASE_URL\"|g" ./mirror/mirror.toml
    sed -i "s|^keep_latest_nightlies = .*|keep_latest_nightlies = 0|g" ./mirror/mirror.toml
    sed -i "s|^keep_latest_betas = .*|keep_latest_betas = 0|g" ./mirror/mirror.toml
    sed -i '/^\[rustup\]/a platforms_unix = ["x86_64-unknown-linux-gnu","x86_64-unknown-linux-musl"]' ./mirror/mirror.toml
    sed -i '/^\[rustup\]/a platforms_windows = []' ./mirror/mirror.toml
    ```

4. Run server sync to retrieve base crates

    ```bash
    docker-compose -f sync.docker-compose.yml run registry_sync
    ```

    :warning: This takes around 20+ minutes to download and uses ~XXGo of storage

## Download crates (online)

Pre-requisite : [rust and cargo are installed on your computer](https://www.rust-lang.org/tools/install)

Let's say you want to download [Huggingface's text-generation-inference](https://github.com/huggingface/text-generation-inference) crates.

1. Go to the download directory

    ```bash
    cd ./download
    ```

2. Clone the project including the `Cargo.toml` and `Cargo.lock` files

    ```bash
    git clone https://github.com/huggingface/text-generation-inference
    cd text-generation-inference
    ```

3. Run the cargo vendor command

    ```bash
    cargo vendor
    ```

## Setup the offline server (offline)

## Push the crates (offline)
