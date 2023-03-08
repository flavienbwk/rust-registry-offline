# renovate: datasource=github-releases depName=rust lookupName=rust-lang/rust
ARG RUST_VERSION=1.67.1

FROM rust:$RUST_VERSION

# renovate: datasource=crate depName=diesel_cli versioning=semver
ARG DIESEL_CLI_VERSION=2.0.1
ARG GIT_REPO_CRATESIO_URL=https://github.com/rust-lang/crates.io
ARG GIT_REPO_CRATESIO_COMMIT_ID=4f77e83728164c2c96f9ad79d5cd625484b1e995

RUN apt-get update \
    && apt-get install -y postgresql git \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install diesel_cli --version $DIESEL_CLI_VERSION --no-default-features --features postgres

WORKDIR /app

RUN git clone "${GIT_REPO_CRATESIO_URL}" crates_io \
    && cd crates_io \
    && git checkout "${GIT_REPO_CRATESIO_COMMIT_ID}" \
    && cd -
RUN cp -r /app/crates_io/* /app && rm -r ./crates_io

RUN cargo build --release

ENTRYPOINT ["/app/docker_entrypoint.sh"]
