FROM rust:1.67.1-bullseye AS builder

ARG GIT_REPO_PANAMAX_URL=https://github.com/panamax-rs/panamax
ARG GIT_REPO_PANAMAX_COMMIT_ID=7cd1ae613547ee2aeb05fa05a42ebe2be9d74467

WORKDIR /app

RUN apt update && apt install -y git

RUN git clone "${GIT_REPO_PANAMAX_URL}" panamax
WORKDIR /app/panamax
RUN git checkout "${GIT_REPO_PANAMAX_COMMIT_ID}"
RUN mkdir /app/bpanamax && cp -r /app/panamax/* /app/bpanamax/

WORKDIR /app/bpanamax
ARG CARGO_BUILD_EXTRA
RUN cargo build --release $CARGO_BUILD_EXTRA

FROM debian:bullseye-20230227

COPY --from=builder /app/bpanamax/target/release/panamax /usr/local/bin

RUN apt update \
    && apt install -y \
        ca-certificates \
        git \
        libssl1.1 \
    && git config --global --add safe.directory '*'

ENTRYPOINT [ "/usr/local/bin/panamax" ]
CMD ["--help"]
