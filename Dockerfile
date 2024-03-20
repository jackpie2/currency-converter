FROM rust:1.76.0 as builder

WORKDIR /usr/src/currency-converter
COPY . .

RUN apt-get update && apt-get -y install openssl pkg-config libssl-dev libssl3 && rm -rf /var/lib/apt/lists/*
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get -y install ca-certificates && update-ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/currency-converter /usr/local/bin/currency-converter

VOLUME [ "/data" ]
WORKDIR /data
ENTRYPOINT [ "currency-converter" ]