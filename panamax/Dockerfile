FROM rust:1.67.1-bullseye AS builder

WORKDIR /app

RUN apt update && apt install -y git
RUN git clone https://github.com/panamax-rs/panamax panamax
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
