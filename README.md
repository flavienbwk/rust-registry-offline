# Rust Registry offline

Run your own crate registry offline for Rust. Get instructions to download and push crates. Perfect for long plane trips ! :airplane:

![Homepage of Panamax](./homepage.png)

Pre-requisite :

- Docker
- Docker-compose

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

    :information_source: Before running this command, you might want to check `./mirror/mirror.toml`'s content

    ```bash
    cp ./mirror.example.toml ./mirror/mirror.toml
    ```

4. Run server sync to retrieve base crates

    ```bash
    docker-compose -f sync.docker-compose.yml run registry_sync
    ```

    :warning: This takes around 1 hour to download on a 30MB/s connection and uses ~XXGo of storage

5. Now export server's docker image and zip this project to copy it on your offline computer

    ```bash
    docker save -o panamax_registry:latest panamax_registry.tar
    zip -r rust-registry-offline.zip ./*
    ```

## Setup the offline server (offline)

1. Load server's docker image and unzip the project

    ```bash
    docker load -i panamax_registry.tar
    unzip -d rust-registry-offline rust-registry-offline.zip
    cd rust-registry-offline
    ```

2. Start the server

    ```bash
    docker-compose up -d
    ```

    Check server is running browsing `http://localhost:8080`

## Download crates (online)

Pre-requisite : [rust and cargo are installed on your computer](https://www.rust-lang.org/tools/install)

Let's say you want to download [Huggingface's text-generation-inference](https://github.com/huggingface/text-generation-inference) crates.


1. Clone the project including the `Cargo.toml` and `Cargo.lock` files

    ```bash
    git clone https://github.com/huggingface/text-generation-inference && cd text-generation-inference
    ```

2. Run the cargo vendor command

    ```bash
    cargo vendor
    ```

3. Zip vendor crates to copy them to your offline computer

    ```bash
    zip -r "crates_$(date +%s).zip" ./vendor/*
    ```

## Push the crates (offline)

1. Unzip vendor crates into the `vendor/` directory

    ```bash
    export CRATE_DIR=$(date +%s)
    zip -d "./vendor/$CRATE_DIR" "crates_XXXXXXXXX.zip"
    ```

2. Push packages

    ```bash
    CRATE_DIR=$CRATE_DIR docker-compose -f push.docker-compose.yml run push
    ```
